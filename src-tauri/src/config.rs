use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
};

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

pub fn load_auto_fan_mode() -> Result<u8, String> {
    let path = config_dir().join("autofan.conf");
    match fs::read_to_string(path) {
        Ok(value) => match value.trim() {
            "0" => Ok(0),
            "1" => Ok(1),
            "2" => Ok(2),
            _ => {
                eprintln!("autofan.conf 的值无效，风扇自启动按关闭处理");
                Ok(0)
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            save_auto_fan_mode(0)?;
            Ok(0)
        }
        Err(e) => Err(format!("读取 autofan.conf 失败: {e}")),
    }
}

pub fn save_auto_fan_mode(mode: u8) -> Result<(), String> {
    if mode > 2 {
        return Err("风扇自启动模式只能是 0、1 或 2".into());
    }

    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("创建配置目录失败: {e}"))?;
    fs::write(dir.join("autofan.conf"), mode.to_string())
        .map_err(|e| format!("写入 autofan.conf 失败: {e}"))
}

pub fn load_log_enabled() -> Result<bool, String> {
    let path = config_dir().join("log.conf");
    match fs::read_to_string(&path) {
        Ok(value) => match value.trim() {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err("log.conf 的值只能是 0 或 1".into()),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            save_log_enabled(false)?;
            Ok(false)
        }
        Err(error) => Err(format!("读取 log.conf 失败: {error}")),
    }
}

pub fn save_log_enabled(enabled: bool) -> Result<(), String> {
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("创建配置目录失败: {e}"))?;
    fs::write(dir.join("log.conf"), if enabled { "1" } else { "0" })
        .map_err(|e| format!("写入 log.conf 失败: {e}"))
}

pub fn dll_dir() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("获取程序路径失败：{e}"))?;

    let install_dir = exe_path
        .parent()
        .ok_or_else(|| "无法获取程序安装目录".to_string())?;

    let dll_path = install_dir.join("dll");

    if !dll_path.is_dir() {
        return Err(format!("DLL 目录不存在：{}", dll_path.display()));
    }

    Ok(dll_path)
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
