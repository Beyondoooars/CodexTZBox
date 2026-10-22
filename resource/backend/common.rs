use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, io::Write, path::Path};

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
pub fn is_valid_iana_zone(zone: &str) -> bool {
    let zone = zone.trim();
    if zone.is_empty() || zone.len() > 64 { return false; }
    if zone == "UTC" || zone == "GMT" { return true; }
    let parts: Vec<&str> = zone.split('/').collect();
    if parts.len() < 2 { return false; }
    parts.iter().all(|part| {
        !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '+')
    })
}
pub fn tz(s: &Settings) -> Result<String, String> {
    match s.mode.as_str() {
        "offset" if (-12..=14).contains(&s.offset) => Ok(if s.offset == 0 { "Etc/UTC".into() } else { format!("Etc/GMT{}{offset}", if s.offset > 0 { "-" } else { "+" }, offset = s.offset.abs()) }),
        "zone" => {
            let zone = s.zone_id.trim();
            if zones().as_array().unwrap().iter().any(|z| z["id"] == zone) || is_valid_iana_zone(zone) {
                Ok(zone.to_string())
            } else {
                Err("请选择有效时区或 -12 至 +14 小时的 UTC 偏移。".into())
            }
        },
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
    tz(s)?;
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let serialized = serde_json::to_vec_pretty(s).map_err(|e| e.to_string())?;
    let target = dir.join("settings.json");

    let mut last_err = None;
    for attempt in 0..3 {
        if attempt > 0 {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        let res = (|| -> Result<(), String> {
            let mut temp = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
            temp.write_all(&serialized).map_err(|e| e.to_string())?;
            temp.as_file().sync_all().map_err(|e| e.to_string())?;
            temp.persist(&target).map_err(|e| e.to_string())?;
            Ok(())
        })();
        match res {
            Ok(_) => return Ok(()),
            Err(e) => last_err = Some(e),
        }
    }
    fs::write(&target, &serialized).map_err(|e| format!("配置保存失败：{e} (前置错误: {last_err:?})"))?;
    Ok(())
}
pub fn settings(payload: &Value, dir: &Path) -> Result<Settings, String> {
    match payload.get("settings").filter(|s| !s.is_null()) {
        Some(s) => { let s = serde_json::from_value(s.clone()).map_err(|e| e.to_string())?; tz(&s)?; Ok(s) },
        None => load(dir),
    }
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

    #[test]
    fn custom_iana_zone_support() {
        let dir = tempfile::tempdir().unwrap();
        let s = Settings { mode: "zone".into(), zone_id: "Asia/Taipei".into(), offset: 8, executable: "".into() };
        assert_eq!(tz(&s).unwrap(), "Asia/Taipei");
        save(dir.path(), &s).unwrap();
        assert_eq!(load(dir.path()).unwrap().zone_id, "Asia/Taipei");

        let invalid = Settings { mode: "zone".into(), zone_id: "invalid zone name".into(), offset: 8, executable: "".into() };
        assert!(tz(&invalid).is_err());
    }
}
