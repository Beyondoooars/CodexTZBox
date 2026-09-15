use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{env, fs, io::Write, path::Path, process::Command};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    #[serde(alias = "Mode")]
    pub mode: String,
    #[serde(alias = "ZoneId")]
    pub zone_id: String,
    #[serde(alias = "Offset")]
    pub offset: i32,
    #[serde(alias = "Executable")]
    pub executable: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self { mode: "zone".into(), zone_id: "Asia/Shanghai".into(), offset: 8, executable: String::new() }
    }
}
pub fn zones() -> Value { serde_json::from_str(include_str!("zones.json")).expect("valid bundled zones") }
pub fn tz(s: &Settings) -> Result<String, String> {
    match s.mode.as_str() {
        "offset" if (-12..=14).contains(&s.offset) => Ok(if s.offset == 0 { "Etc/UTC".into() } else { format!("Etc/GMT{}{offset}", if s.offset > 0 { "-" } else { "+" }, offset = s.offset.abs()) }),
        "zone" if zones().as_array().unwrap().iter().any(|z| z["id"] == s.zone_id) => Ok(s.zone_id.clone()),
        _ => Err("请选择有效时区或 -12 至 +14 小时的 UTC 偏移。".into()),
    }
}
pub fn load(dir: &Path) -> Result<Settings, String> {
    let path = dir.join("settings.json");
    if !path.exists() { return Ok(Settings::default()); }
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let s: Settings = serde_json::from_slice(bytes.strip_prefix(&[0xef,0xbb,0xbf]).unwrap_or(&bytes)).map_err(|e| format!("设置文件无效：{e}"))?;
    tz(&s)?; Ok(s)
}
pub fn save(dir: &Path, s: &Settings) -> Result<(), String> {
    tz(s)?; fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let mut temp = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
    temp.write_all(&serde_json::to_vec_pretty(s).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    temp.as_file().sync_all().map_err(|e| e.to_string())?;
    temp.persist(dir.join("settings.json")).map_err(|e| e.to_string())?;
    Ok(())
}
pub fn settings(payload: &Value, dir: &Path) -> Result<Settings, String> {
    match payload.get("settings").filter(|s| !s.is_null()) {
        Some(s) => { let s = serde_json::from_value(s.clone()).map_err(|e| e.to_string())?; tz(&s)?; Ok(s) },
        None => load(dir),
    }
}

fn public_ip(use_proxy: bool) -> Result<Value, String> {
    let mut command = Command::new("curl.exe");
    command.args(["--fail", "--silent", "--show-error", "--max-time", "10", "https://ipinfo.io/json"]);
    if !use_proxy { command.env_remove("HTTP_PROXY").env_remove("HTTPS_PROXY").env_remove("ALL_PROXY").arg("--noproxy").arg("*"); }
    let output = command.output().map_err(|e| format!("无法查询公网 IP：{e}"))?;
    if !output.status.success() { return Err(format!("公网 IP 查询失败：{}", String::from_utf8_lossy(&output.stderr).trim())); }
    let value: Value = serde_json::from_slice(&output.stdout).map_err(|e| format!("公网 IP 服务返回无效数据：{e}"))?;
    let ip = value["ip"].as_str().filter(|s| !s.trim().is_empty()).ok_or("公网 IP 服务未返回地址。")?;
    let zone = value["timezone"].as_str().filter(|s| !s.trim().is_empty()).ok_or("公网 IP 服务未返回时区。")?;
    if !zones().as_array().unwrap().iter().any(|z| z["id"] == zone) { return Err(format!("公网 IP 返回了未收录的时区：{zone}")); }
    Ok(serde_json::json!({"ip":ip,"timezone":zone,"city":value["city"],"country":value["country_name"],"viaProxy":use_proxy}))
}
pub fn network_info() -> Result<Value, String> {
    let has_proxy = ["HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy", "ALL_PROXY", "all_proxy"].iter().any(|key| env::var_os(key).is_some_and(|v| !v.is_empty()));
    let proxy = if has_proxy { public_ip(true).ok() } else { None };
    let direct = public_ip(false).ok();
    if proxy.is_none() && direct.is_none() { return Err("无法获取公网 IP，请检查网络连接。".into()); }
    Ok(serde_json::json!({"proxy":proxy,"direct":direct,"proxyConfigured":has_proxy}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_settings_and_atomic_replace() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("settings.json"), "\u{feff}{\"Mode\":\"offset\",\"Offset\":-5}").unwrap();
        let mut s = load(dir.path()).unwrap();
        assert_eq!(tz(&s).unwrap(), "Etc/GMT+5");
        s.offset = 9;
        save(dir.path(), &s).unwrap();
        assert_eq!(load(dir.path()).unwrap().offset, 9);
        s.offset = 99;
        assert!(save(dir.path(), &s).is_err());
        assert_eq!(load(dir.path()).unwrap().offset, 9);
    }
}
