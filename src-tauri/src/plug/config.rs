use std::{
    io::Read,
    fs::{self, File},
    path::PathBuf
};

use crate::plug::struct_set::FanData;
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

#[cfg(unix)]
pub fn find_hwmon_with_name() -> PathBuf {
    let hwmon_dir = "/sys/class/hwmon";
    for entry in fs::read_dir(hwmon_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() && path.file_name().map(
            |name| name.to_str().unwrap_or("").starts_with("hwmon")
        ).unwrap_or(false) {
            let name_path = path.join("name");
            if name_path.exists() {
                let mut name_file = File::open(name_path).unwrap();
                let mut content = String::new();
                name_file.read_to_string(&mut content).unwrap();
                if content.trim() == "uniwill" {
                    return path;
                }
            }
        }
    }
    panic!("未找到匹配的设备");
}

#[tauri::command]
pub async fn save_fan_config(fan_data: FanData) -> Result<(), String> {
    let config_path = get_config_file_path()?;
    let json_data = serde_json::to_string_pretty(&fan_data).map_err(|e| e.to_string())?;
    fs::write(config_path, json_data).map_err(|e| e.to_string())?;
    println!("风扇配置已保存");
    Ok(())
}

#[tauri::command]
pub async fn load_fan_config() -> FanData {
    let config_path = get_config_file_path().unwrap();
    if !config_path.exists() {
        println!("配置文件不存在");
    }
    let json_data = fs::read_to_string(config_path)
        .map_err(|e| e.to_string())
        .unwrap();
    let fan_data: FanData = serde_json::from_str(&json_data)
        .map_err(|e| e.to_string())
        .unwrap();
    println!("风扇配置已加载");
    fan_data
}
