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
