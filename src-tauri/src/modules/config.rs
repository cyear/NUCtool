use std::fs;
use std::path::PathBuf;

pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap()
        .join("com.nuc.x15.fan.cyear.app")
}
pub fn get_config_file_path() -> Result<PathBuf, String> {
    // 获取应用的配置目录
    let config_dir = get_config_dir();
    // 确保配置目录存在
    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    let config_tdp = config_dir.join("debug.config");
    if !config_tdp.exists() {
        println!("debug.config 配置文件不存在");
        fs::write(&config_tdp, "0").map_err(|e| e.to_string())?;
    }
    println!("{:?}", &config_tdp);
    // 配置文件名
    let config_file = config_dir.join("fan_config.json");
    println!("{:?}", &config_file);
    Ok(config_file)
}
