use crate::common::{load, save, settings, tz, zones, Settings};
use quick_xml::events::Event;
use serde_json::{json, Value};
use std::{fs, mem::size_of, path::{Path, PathBuf}, process::{Command, Stdio}, time::Duration};
use ::windows::{
    core::{BOOL, GUID, HSTRING, PWSTR},
    Management::Deployment::PackageManager,
    Win32::{
        Foundation::{CloseHandle, HANDLE, HWND, LPARAM, WPARAM},
        System::{
            Com::CoTaskMemFree,
            Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS},
            RemoteDesktop::ProcessIdToSessionId,
            Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION},
            WinRT::{RoInitialize, RoUninitialize, RO_INIT_SINGLETHREADED},
        },
        UI::{
            Shell::{FOLDERID_LocalAppData, KF_FLAG_DEFAULT, SHGetKnownFolderPath},
            WindowsAndMessaging::{EnumWindows, GetWindowTextLengthW, GetWindowThreadProcessId, IsWindowVisible, PostMessageW, WM_CLOSE},
        },
    },
};

struct Runtime {
    initialized: bool,
}
impl Runtime {
    fn new() -> Result<Self, String> {
        match unsafe { RoInitialize(RO_INIT_SINGLETHREADED) } {
            Ok(_) => Ok(Self { initialized: true }),
            Err(err) if err.code().0 == -2147417850 /* RPC_E_CHANGED_MODE: 0x80010106 */ => {
                Ok(Self { initialized: false })
            }
            Err(err) => Err(format!("无法初始化系统运行时：{err}")),
        }
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        if self.initialized {
            unsafe { RoUninitialize(); }
        }
    }
}
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
fn clean_path(path: &Path) -> String {
    path.to_string_lossy().trim_start_matches(r"\\?\").to_string()
}
fn data_directory() -> Result<PathBuf, String> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let portable_data = parent.join("data");
            if fs::create_dir_all(&portable_data).is_ok() {
                let test_file = portable_data.join(".write_test");
                if fs::write(&test_file, b"ok").is_ok() {
                    let _ = fs::remove_file(test_file);
                    return Ok(portable_data);
                }
            }
        }
    }
    if let Ok(local) = known_folder(&FOLDERID_LocalAppData) {
        let fallback_data = local.join("CodexTZBox").join("data");
        let _ = fs::create_dir_all(&fallback_data);
        return Ok(fallback_data);
    }
    Err("无法确定可写的数据存储目录。".into())
}
fn load_migrated(dir: &Path) -> Result<Settings, String> {
    let s = if dir.join("settings.json").exists() {
        load(dir)?
    } else {
        let local = known_folder(&FOLDERID_LocalAppData)?;
        let legacy = [
            local.join("CodexTimeZoneSwitcher").join("data"),
            local.join("ChatGPTTimeZoneLauncher"),
        ]
        .into_iter()
        .find(|legacy_dir| legacy_dir.join("settings.json").exists());
        match legacy.and_then(|legacy_dir| load(&legacy_dir).ok()) {
            Some(legacy_settings) => {
                save(dir, &legacy_settings)?;
                legacy_settings
            }
            None => Settings::default(),
        }
    };
    Ok(s)
}
fn validate(path: &str) -> Result<PathBuf, String> {
    let path = fs::canonicalize(path.trim().trim_matches('"')).map_err(|_| "客户端路径不存在，请选择 Codex 桌面程序。")?;
    let name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
    if !path.is_file() || !["codex.exe", "chatgpt.exe"].contains(&name.as_str()) {
        return Err("请选择 Codex 桌面安装目录中的 Codex.exe / ChatGPT.exe，不要选择命令行程序。".into());
    }
    let parent = path.parent().ok_or("客户端目录无效")?;
    let has_resources = parent.join("icudtl.dat").is_file()
        || parent.join("resources").join("icudtl.dat").is_file()
        || parent.join("resources").join("app.asar").is_file()
        || parent.join("resources").is_dir();
    if !has_resources {
        return Err("请选择 Codex 桌面安装目录中的 Codex.exe / ChatGPT.exe，不要选择命令行程序。".into());
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
            Event::Start(element) | Event::Empty(element) if element.local_name().as_ref() == "Application" => {
                let mut is_app = false;
                let mut executable = None;
                for attribute in element.attributes().flatten() {
                    match attribute.key.local_name().as_ref() {
                        "Id" => {
                            let val = attribute.value.as_ref();
                            is_app = val == "App" || val.eq_ignore_ascii_case("OpenAI.Codex") || val.eq_ignore_ascii_case("OpenAICodex");
                        }
                        "Executable" => executable = Some(attribute.value.into_owned()),
                        _ => {}
                    }
                }
                if is_app && executable.is_some() {
                    executable_name = executable;
                    break;
                }
                if executable_name.is_none() {
                    executable_name = executable;
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
fn running_client_pids(executable: &Path) -> Result<Vec<u32>, String> {
    let mut pids = Vec::new();
    unsafe {
        let mut session = 0;
        ProcessIdToSessionId(std::process::id(), &mut session).map_err(|e| e.to_string())?;
        let snapshot = Handle(CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).map_err(|e| e.to_string())?);
        let mut entry = PROCESSENTRY32W { dwSize: size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        if Process32FirstW(snapshot.0, &mut entry).is_err() { return Ok(pids); }
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
                        let matched = same_path(executable, &proc_path) || (
                            executable.parent().is_some() &&
                            proc_path.parent().is_some() &&
                            same_path(executable.parent().unwrap(), proc_path.parent().unwrap())
                        );
                        if matched { pids.push(entry.th32ProcessID); }
                    }
                }
            }
            if Process32NextW(snapshot.0, &mut entry).is_err() { break; }
        }
    }
    Ok(pids)
}
fn is_running(executable: &Path) -> Result<bool, String> {
    Ok(!running_client_pids(executable)?.is_empty())
}
fn request_graceful_exit(pids: &[u32]) {
    struct State {
        pids: Vec<u32>,
        preferred: Vec<HWND>,
        fallback: Vec<HWND>,
    }
    unsafe extern "system" fn collect(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let state = unsafe { &mut *(lparam.0 as *mut State) };
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)); }
        if state.pids.contains(&pid) {
            let visible = unsafe { IsWindowVisible(hwnd) }.as_bool();
            let titled = unsafe { GetWindowTextLengthW(hwnd) } > 0;
            if visible && titled { state.preferred.push(hwnd); } else { state.fallback.push(hwnd); }
        }
        BOOL(1)
    }
    let mut state = State { pids: pids.to_vec(), preferred: Vec::new(), fallback: Vec::new() };
    unsafe {
        let _ = EnumWindows(Some(collect), LPARAM(&mut state as *mut State as isize));
    }
    let targets = if state.preferred.is_empty() { &state.fallback } else { &state.preferred };
    for hwnd in targets {
        unsafe {
            let _ = PostMessageW(Some(*hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }
}
fn wait_for_client_exit(executable: &Path, timeout: Duration) -> Result<bool, String> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if running_client_pids(executable)?.is_empty() { return Ok(true); }
        if std::time::Instant::now() >= deadline { return Ok(false); }
        std::thread::sleep(Duration::from_millis(400));
    }
}
pub fn launch(dir: &Path, s: Settings, exit_running: bool) -> Result<Value, String> {
    tz(&s)?;
    let path = validate(&if s.executable.trim().is_empty() { discover() } else { s.executable.clone() })?;
    if is_running(&path)? {
        if !exit_running {
            return Ok(json!({
                "launched": false,
                "needsExitConfirm": true,
                "message": "检测到 Codex 正在运行。需要请求其正常退出后重新启动吗？"
            }));
        }
        let pids = running_client_pids(&path)?;
        request_graceful_exit(&pids);
        if !wait_for_client_exit(&path, Duration::from_secs(15))? {
            return Err("Codex 未能在 15 秒内退出，请先在客户端中保存工作并手动完全退出后重试。".into());
        }
    }
    let mut cmd = Command::new(&path);
    cmd.env("TZ", tz(&s)?)
        .env_remove("ELECTRON_RUN_AS_NODE")
        .current_dir(path.parent().ok_or("客户端目录无效")?)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

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
            let direct_cli = local.join("OpenAI").join("Codex").join("bin").join("codex.exe");
            if direct_cli.is_file() {
                cmd.env("CODEX_CLI_PATH", direct_cli);
            } else {
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
    }

    save(dir, &s)?;
    let mut child = cmd.spawn().map_err(|e| format!("无法启动客户端：{e}"))?;
    std::thread::sleep(Duration::from_millis(800));
    if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
        if !status.success() {
            let mut err_msg = String::new();
            if let Some(mut err_pipe) = child.stderr.take() {
                use std::io::Read;
                let mut buf = Vec::new();
                let _ = err_pipe.read_to_end(&mut buf);
                let text = String::from_utf8_lossy(&buf).trim().to_string();
                if !text.is_empty() {
                    err_msg = format!("：{text}");
                }
            }
            return Err(format!("客户端启动后异常退出 ({status}){err_msg}"));
        }
    }
    Ok(json!({"launched":true,"message":"Codex 已按所选时区启动。"}))
}
pub fn execute(command: &str, payload: Value) -> Result<Value, String> {
    let _runtime = Runtime::new()?;
    let dir = data_directory()?;
    match command {
        // 已保存客户端路径时跳过整机应用包扫描，缩短启动到可用的时间；未配置或路径为空时再探测。
        "bootstrap" => {
            let settings = load_migrated(&dir)?;
            let detected = if settings.executable.trim().is_empty() { discover() } else { String::new() };
            Ok(json!({"settings":settings,"zones":zones(),"localZone":"UTC","detected":detected,"settingsPath":dir.join("settings.json")}))
        },
        "discover" => Ok(json!({"path":discover()})),
        "validate" => Ok(json!({"path":clean_path(&validate(payload["path"].as_str().unwrap_or_default())?)})),
        "save" => { save(&dir, &settings(&payload, &dir)?)?; Ok(json!({"path":dir.join("settings.json"),"message":format!("设置已保存到 {}", dir.join("settings.json").display())})) },
        "launch" => launch(&dir, settings(&payload, &dir)?, payload["exitRunning"].as_bool().unwrap_or(false)),
        _ => Err(format!("未知命令：{command}")),
    }
}

