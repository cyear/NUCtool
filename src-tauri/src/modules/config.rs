use crate::modules::struct_set::FanData;
use std::{fs, path::PathBuf};

pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap()
        .join("com.nuc.x15.fan.cyear.app")
}
pub fn get_config_file_path() -> Result<PathBuf, String> {
    // 获取应用的配置目录
    let config_dir = get_config_dir();
    // 配置文件名
    let config_file = config_dir.join("fan_config.json");
    println!("{:?}", &config_file);
    Ok(config_file)
}

#[tauri::command]
pub async fn save_fan_config(fan_data: FanData) -> Result<(), String> {
    // 获取配置文件路径
    let config_path = get_config_file_path()?;

    // 将 fan_data 序列化为 JSON
    let json_data = serde_json::to_string_pretty(&fan_data).map_err(|e| e.to_string())?;

    // 写入配置文件
    fs::write(config_path, json_data).map_err(|e| e.to_string())?;

    println!("风扇配置已保存");
    Ok(())
}

#[tauri::command]
pub async fn load_fan_config() -> FanData {
    // 获取配置文件路径
    let config_path = get_config_file_path().unwrap();
    // 检查配置文件是否存在
    if !config_path.exists() {
        println!("配置文件不存在");
    }
    // 读取配置文件
    let json_data = fs::read_to_string(config_path)
        .map_err(|e| e.to_string())
        .unwrap();
    // 反序列化为 FanData
    let fan_data: FanData = serde_json::from_str(&json_data)
        .map_err(|e| e.to_string())
        .unwrap();
    println!("风扇配置已加载");
    fan_data
}
