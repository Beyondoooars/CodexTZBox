use crate::common::{load, network_info, save, settings, tz, zones, Settings};
use quick_xml::events::Event;
use serde_json::{json, Value};
use std::{fs, mem::size_of, os::windows::ffi::OsStrExt, path::{Path, PathBuf}, process::{Command, Stdio}, time::Duration};
use ::windows::{
    core::{w, GUID, HSTRING, Interface, PCWSTR, PWSTR},
    Management::Deployment::PackageManager,
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        System::{
            Com::{CoCreateInstance, CoTaskMemFree, IPersistFile, CLSCTX_INPROC_SERVER},
            Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS},
            RemoteDesktop::ProcessIdToSessionId,
            Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION},
            WinRT::{RoInitialize, RoUninitialize, RO_INIT_SINGLETHREADED},
        },
        UI::Shell::{FOLDERID_Desktop, FOLDERID_LocalAppData, IShellLinkW, KF_FLAG_DEFAULT, SHGetKnownFolderPath, ShellLink},
    },
};

struct Runtime;
impl Runtime {
    fn new() -> Result<Self, String> {
        unsafe { RoInitialize(RO_INIT_SINGLETHREADED) }.map_err(|e| e.to_string())?;
        Ok(Self)
    }
}
impl Drop for Runtime { fn drop(&mut self) { unsafe { RoUninitialize(); } } }
struct Handle(HANDLE);
impl Drop for Handle { fn drop(&mut self) { unsafe { let _ = CloseHandle(self.0); } } }
fn known_folder(id: &GUID) -> Result<PathBuf, String> {
    unsafe {
        let pointer = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None).map_err(|e| e.to_string())?;
        let value = pointer.to_string().map_err(|e| e.to_string());
        CoTaskMemFree(Some(pointer.0.cast()));
        value.map(PathBuf::from)
    }
}
fn wide(path: &Path) -> Vec<u16> { path.as_os_str().encode_wide().chain(Some(0)).collect() }
fn clean_path(path: &Path) -> String {
    path.to_string_lossy().trim_start_matches(r"\\?\").to_string()
}
fn data_directory() -> Result<PathBuf, String> {
    Ok(std::env::current_exe().map_err(|e| e.to_string())?.parent().ok_or("无法确定程序目录。")?.join("data"))
}
fn load_migrated(dir: &Path) -> Result<Settings, String> {
    let mut s = if dir.join("settings.json").exists() {
        load(dir)?
    } else {
        let legacy = known_folder(&FOLDERID_LocalAppData)?.join("ChatGPTTimeZoneLauncher");
        if legacy.join("settings.json").exists() {
            if let Ok(legacy_settings) = load(&legacy) {
                save(dir, &legacy_settings)?;
                legacy_settings
            } else {
                Settings::default()
            }
        } else {
            Settings::default()
        }
    };
    if !s.executable.trim().is_empty() && validate(&s.executable).is_err() {
        s.executable.clear();
        let _ = save(dir, &s);
    }
    Ok(s)
}
fn validate(path: &str) -> Result<PathBuf, String> {
    let mut path = fs::canonicalize(path.trim().trim_matches('"')).map_err(|_| "客户端路径不存在，请选择 Codex 桌面程序。")?;
    let name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
    if !path.is_file() || !["codex.exe", "chatgpt.exe"].contains(&name.as_str()) || !path.parent().ok_or("客户端目录无效")?.join("icudtl.dat").is_file() {
        return Err("请选择 Codex 桌面安装目录中的 Codex.exe / ChatGPT.exe，不要选择命令行程序。".into());
    }
    if name == "codex.exe" {
        if let Some(parent) = path.parent() {
            let chatgpt = parent.join("ChatGPT.exe");
            if chatgpt.is_file() {
                path = chatgpt;
            }
        }
    }
    Ok(path)
}
fn manifest_executable(root: &Path) -> Option<PathBuf> {
    let root = fs::canonicalize(root).ok()?;
    let text = fs::read_to_string(root.join("AppxManifest.xml")).ok()?;
    let mut reader = quick_xml::Reader::from_str(&text);
    reader.config_mut().trim_text(true);
    let mut executable_name = None;
    loop {
        match reader.read_event().ok()? {
            Event::Start(element) if element.local_name().as_ref() == "Application" => {
                let mut is_app = false;
                let mut executable = None;
                for attribute in element.attributes().flatten() {
                    match attribute.key.local_name().as_ref() {
                        "Id" => is_app = attribute.value.as_ref() == "App",
                        "Executable" => executable = Some(attribute.value.into_owned()),
                        _ => {}
                    }
                }
                if is_app {
                    executable_name = executable;
                    break;
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    let executable = fs::canonicalize(root.join(executable_name?.replace('/', "\\"))).ok()?;
    if !executable.starts_with(&root) { return None; }
    validate(&executable.to_string_lossy()).ok()
}
fn discover() -> String {
    let mut candidates = Vec::new();
    if let Ok(manager) = PackageManager::new() {
        if let Ok(packages) = manager.FindPackagesByUserSecurityId(&HSTRING::new()) {
            for package in packages {
                let found = || -> ::windows::core::Result<_> {
                    let id = package.Id()?;
                    let name = id.Name()?.to_string_lossy();
                    let v = id.Version()?;
                    let root = package.InstalledLocation()?.Path()?.to_string_lossy();
                    Ok((name, (v.Major, v.Minor, v.Build, v.Revision), PathBuf::from(root)))
                };
                if let Ok((name, version, root)) = found() {
                    if name.eq_ignore_ascii_case("OpenAI.Codex") { candidates.push((version, root)); }
                }
            }
        }
    }
    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    for (_, root) in candidates {
        if let Some(path) = manifest_executable(&root) { return clean_path(&path); }
    }
    if let Ok(local) = known_folder(&FOLDERID_LocalAppData) {
        for relative in [r"Programs\Codex\Codex.exe", r"Codex\Codex.exe", r"Programs\OpenAI\Codex\Codex.exe"] {
            if let Ok(path) = validate(&local.join(relative).to_string_lossy()) { return clean_path(&path); }
        }
    }
    String::new()
}
fn same_path(a: &Path, b: &Path) -> bool {
    let normalize = |p: &Path| fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()).to_string_lossy().trim_start_matches(r"\\?\").to_lowercase();
    normalize(a) == normalize(b)
}
fn is_running(executable: &Path) -> Result<bool, String> {
    unsafe {
        let mut session = 0;
        ProcessIdToSessionId(std::process::id(), &mut session).map_err(|e| e.to_string())?;
        let snapshot = Handle(CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).map_err(|e| e.to_string())?);
        let mut entry = PROCESSENTRY32W { dwSize: size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        if Process32FirstW(snapshot.0, &mut entry).is_err() { return Ok(false); }
        loop {
            let end = entry.szExeFile.iter().position(|c| *c == 0).unwrap_or(entry.szExeFile.len());
            let name = String::from_utf16_lossy(&entry.szExeFile[..end]).to_lowercase();
            if ["codex.exe", "chatgpt.exe"].contains(&name.as_str()) {
                let mut candidate_session = 0;
                if ProcessIdToSessionId(entry.th32ProcessID, &mut candidate_session).is_ok() && session == candidate_session {
                    let process = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, entry.th32ProcessID) {
                        Ok(p) => Handle(p),
                        Err(_) => {
                            if Process32NextW(snapshot.0, &mut entry).is_err() { break; }
                            continue;
                        }
                    };
                    let mut buffer = vec![0u16; 32768];
                    let mut length = buffer.len() as u32;
                    if QueryFullProcessImageNameW(process.0, PROCESS_NAME_WIN32, PWSTR(buffer.as_mut_ptr()), &mut length).is_ok() {
                        let proc_path = PathBuf::from(String::from_utf16_lossy(&buffer[..length as usize]));
                        if same_path(executable, &proc_path) {
                            return Ok(true);
                        }
                    }
                }
            }
            if Process32NextW(snapshot.0, &mut entry).is_err() { break; }
        }
    }
    Ok(false)
}
fn create_shortcut() -> Result<PathBuf, String> {
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let desktop = known_folder(&FOLDERID_Desktop)?;
    let path = desktop.join("Codex 时区启动器.lnk");
    // Stage the COM-created shortcut before replacing the destination atomically.
    let staging = tempfile::tempdir_in(&desktop).map_err(|e| e.to_string())?;
    let staged = staging.path().join("launcher.lnk");
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).map_err(|e| e.to_string())?;
        link.SetPath(PCWSTR(wide(&executable).as_ptr())).map_err(|e| e.to_string())?;
        link.SetWorkingDirectory(PCWSTR(wide(executable.parent().ok_or("程序目录无效")?).as_ptr())).map_err(|e| e.to_string())?;
        link.SetDescription(w!("选择时区并启动 Codex")).map_err(|e| e.to_string())?;
        link.SetIconLocation(PCWSTR(wide(&executable).as_ptr()), 0).map_err(|e| e.to_string())?;
        let persist: IPersistFile = link.cast().map_err(|e| e.to_string())?;
        persist.Save(PCWSTR(wide(&staged).as_ptr()), true).map_err(|e| e.to_string())?;
    }
    let mut file = tempfile::NamedTempFile::new_in(&desktop).map_err(|e| e.to_string())?;
    use std::io::Write;
    file.write_all(&fs::read(staged).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    file.persist(&path).map_err(|e| e.to_string())?;
    Ok(path)
}
pub fn launch(dir: &Path, s: Settings) -> Result<Value, String> {
    tz(&s)?;
    let path = validate(&if s.executable.trim().is_empty() { discover() } else { s.executable.clone() })?;
    if is_running(&path)? { return Err("客户端正在运行，请保存工作并完全退出 Codex 后重试。".into()); }
    let mut cmd = Command::new(&path);
    cmd.env("TZ", tz(&s)?)
        .env_remove("ELECTRON_RUN_AS_NODE")
        .current_dir(path.parent().ok_or("客户端目录无效")?)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // OpenAI Codex 26.924+ compatibility:
    // When launched outside the MSIX sandbox container without package identity,
    // Codex will throw APPMODEL_ERROR_NO_PACKAGE ("该进程没有程序包标识符")
    // during updater initialization and core runtime resolution.
    // 1) Set CODEX_SPARKLE_ENABLED="false" to disable store updater init.
    // 2) Set CODEX_CLI_PATH to the bundled or local codex.exe CLI tool, which causes
    //    Codex to skip package identity check and directly use the core CLI runtime.
    cmd.env("CODEX_SPARKLE_ENABLED", "false");
    if let Some(parent) = path.parent() {
        let bundled_cli = parent.join("resources").join("codex.exe");
        if bundled_cli.is_file() {
            cmd.env("CODEX_CLI_PATH", bundled_cli);
        } else if let Ok(local) = known_folder(&FOLDERID_LocalAppData) {
            let user_cli_dir = local.join("OpenAI").join("Codex").join("bin");
            if let Ok(entries) = fs::read_dir(&user_cli_dir) {
                for entry in entries.flatten() {
                    let candidate = entry.path().join("codex.exe");
                    if candidate.is_file() {
                        cmd.env("CODEX_CLI_PATH", candidate);
                        break;
                    }
                }
            }
        }
    }

    save(dir, &s)?;
    let mut child = cmd.spawn().map_err(|e| format!("无法启动客户端：{e}"))?;
    std::thread::sleep(Duration::from_millis(800));
    if let Some(status) = child.try_wait().map_err(|e| e.to_string())? { return Err(format!("客户端启动后提前退出：{status}。")); }
    std::thread::spawn(move || { let _ = child.wait(); });
    Ok(json!({"launched":true,"message":"Codex 已按所选时区启动。"}))
}
pub fn execute(command: &str, payload: Value) -> Result<Value, String> {
    let _runtime = Runtime::new()?;
    let dir = data_directory()?;
    match command {
        "network_info" => network_info(),
        "bootstrap" => Ok(json!({"settings":load_migrated(&dir)?,"zones":zones(),"localZone":"UTC","detected":discover(),"platform":"windows","settingsPath":dir.join("settings.json")})),
        "discover" => Ok(json!({"path":discover()})),
        "validate" => Ok(json!({"path":clean_path(&validate(payload["path"].as_str().unwrap_or_default())?)})),
        "save" => { save(&dir, &settings(&payload, &dir)?)?; Ok(json!({"path":dir.join("settings.json"),"message":format!("设置已保存到 {}", dir.join("settings.json").display())})) },
        "launch" => launch(&dir, settings(&payload, &dir)?),
        "create_shortcut" => Ok(json!({"created":true,"path":create_shortcut()?})),
        _ => Err(format!("未知命令：{command}")),
    }
}

