use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct FanPoint {
    pub temperature: u8,
    pub speed: u8,
}

/// 左右风扇曲线
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FanData {
    pub left_fan: Vec<FanPoint>,
    pub right_fan: Vec<FanPoint>,
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.cyear.nuctool")
}

use std::{
    fs::OpenOptions,
    io::Write,
    panic,
};

pub fn install_panic_hook() {
    panic::set_hook(Box::new(|info| {
        let mut log = String::new();

        log.push_str("\n========== NUCtool RUST PANIC ==========\n");

        if let Some(location) = info.location() {
            log.push_str(&format!(
                "Location: {}:{}:{}\n",
                location.file(),
                location.line(),
                location.column()
            ));
        } else {
            log.push_str("Location: <unknown>\n");
        }

        if let Some(message) = info.payload().downcast_ref::<&str>() {
            log.push_str(&format!("Message: {}\n", message));
        } else if let Some(message) = info.payload().downcast_ref::<String>() {
            log.push_str(&format!("Message: {}\n", message));
        } else {
            log.push_str("Message: <unknown>\n");
        }

        log.push_str("========================================\n");

        eprintln!("{}", log);

        let dir = config_dir();

        if std::fs::create_dir_all(&dir).is_ok() {
            let path = dir.join("crash.log");

            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let _ = file.write_all(log.as_bytes());
                let _ = file.flush();
            }
        }
    }));
}

fn fan_config_path() -> PathBuf {
    config_dir().join("fan_config.json")
}

pub fn save(data: &FanData) -> Result<(), String> {
    fs::create_dir_all(config_dir()).map_err(|e| format!("创建配置目录失败: {e}"))?;
    let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    let path = fan_config_path();
    fs::write(&path, json).map_err(|e| format!("写入配置失败: {e}"))?;
    println!("风扇配置已保存: {:?}", path);
    Ok(())
}

fn fan_mode_config_path() -> PathBuf {
    config_dir().join("fan_mode.txt")
}

pub fn load_fan_mode() -> Result<i32, String> {

    let path = fan_mode_config_path();

    if !path.exists() {
        println!("风扇模式配置不存在返回: 1");
        return Ok(1);
    }

    let value = fs::read_to_string(path)
        .map_err(|e| e.to_string())?;
    println!("读取风扇模式配置: {}", value);

    value
        .trim()
        .parse::<i32>()
        .map_err(|e| e.to_string())
}

pub fn save_fan_mode(mode: i32) -> Result<(), String> {
    println!("写入风扇模式配置: {}", mode);
    fs::write(
        fan_mode_config_path(),
        mode.to_string(),
    )
    .map_err(|e| e.to_string())
}

pub fn load() -> Result<FanData, String> {
    let path = fan_config_path();
    if !path.exists() {
        return Err("配置文件不存在, 请先调整曲线并保存配置".into());
    }
    let json = fs::read_to_string(&path).map_err(|e| format!("读取配置失败: {e}"))?;
    let data: FanData = serde_json::from_str(&json).map_err(|e| format!("配置解析失败: {e}"))?;
    println!("风扇配置已加载: {:?}", path);
    Ok(data)
}
