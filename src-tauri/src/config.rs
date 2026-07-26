//! 配置文件读写
//!
//! 旧版 `load_fan_config` 在文件缺失/损坏时直接 `unwrap()`,
//! 配合 release 的 `panic = "abort"` 会导致点击"加载配置"闪退;
//! 现在全部以 `Result` 返回给前端处理。

use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

/// 风扇曲线点(前端图表拖动可能产生小数, 用 f64 兼容新旧配置文件)
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct FanPoint {
    pub temperature: f64,
    pub speed: f64,
}

/// 左右风扇曲线
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FanData {
    pub left_fan: Vec<FanPoint>,
    pub right_fan: Vec<FanPoint>,
}

/// 配置目录: `%AppData%\com.nuc.x15.fan.cyear.app`
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.nuc.x15.fan.cyear.app")
}

fn fan_config_path() -> PathBuf {
    config_dir().join("fan_config.json")
}

pub fn save(data: &FanData) -> Result<(), String> {
    fs::create_dir_all(config_dir()).map_err(|e| format!("创建配置目录失败: {e}"))?;
    let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    fs::write(fan_config_path(), json).map_err(|e| format!("写入配置失败: {e}"))?;
    println!("风扇配置已保存");
    Ok(())
}

pub fn load() -> Result<FanData, String> {
    let path = fan_config_path();
    if !path.exists() {
        return Err("配置文件不存在, 请先调整曲线并保存配置".into());
    }
    let json = fs::read_to_string(&path).map_err(|e| format!("读取配置失败: {e}"))?;
    let data: FanData = serde_json::from_str(&json).map_err(|e| format!("配置解析失败: {e}"))?;
    println!("风扇配置已加载");
    Ok(data)
}

/// 自启动标记(`beta.config` 内容为 "1" 表示开启)。
/// 文件不存在时创建并写入默认值 "0"; 内容做 trim 处理(旧版没有 trim,
/// 手动编辑带换行会导致启动失败)。
pub fn autostart_flag() -> bool {
    let dir = config_dir();
    let _ = fs::create_dir_all(&dir);
    let path = dir.join("beta.config");
    if !path.exists() {
        println!("beta.config 不存在, 写入默认值 0");
        let _ = fs::write(&path, "0");
        return false;
    }
    fs::read_to_string(&path)
        .map(|s| s.trim() == "1")
        .unwrap_or(false)
}
