use serde::{Deserialize, Serialize};
use std::{
    panic,
    process,
    io::Write,
    path::PathBuf,
    fs::{self, OpenOptions},
    time::{SystemTime, UNIX_EPOCH},
};
use chrono::Local;

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


/// 安装全局 Rust Panic Hook
///
/// 日志目录：
/// %APPDATA%\com.cyear.nuctool\
///
/// 日志文件：
/// crash_YYYYMMDD_HHMMSS_mmm.log
///
/// 日志数量 >= 30 时：
/// 删除最旧的 10 个日志。
pub fn install_panic_hook() {
    panic::set_hook(Box::new(|info| {
        // =========================================================
        // 时间
        // =========================================================

        let now = Local::now();

        // 用于文件名
        let timestamp = now.format("%Y%m%d_%H%M%S_%3f");

        // 用于日志内容
        let readable_time = now.format("%Y-%m-%d %H:%M:%S%.3f");

        // =========================================================
        // 创建日志内容
        // =========================================================

        let mut log = String::new();

        log.push_str("============================================================\n");
        log.push_str("                    NUCtool CRASH REPORT\n");
        log.push_str("============================================================\n");

        // =========================================================
        // 基本信息
        // =========================================================

        log.push_str("\n[Crash]\n");

        log.push_str(&format!(
            "Time:           {}\n",
            readable_time
        ));

        log.push_str(&format!(
            "PID:            {}\n",
            process::id()
        ));

        if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
            log.push_str(&format!(
                "Unix Time:      {}\n",
                duration.as_secs()
            ));

            log.push_str(&format!(
                "Unix Millis:    {}\n",
                duration.as_millis()
            ));
        }

        // =========================================================
        // Application
        // =========================================================

        log.push_str("\n[Application]\n");

        log.push_str(&format!(
            "Name:           NUCtool\n"
        ));

        log.push_str(&format!(
            "Package:        {}\n",
            env!("CARGO_PKG_NAME")
        ));

        log.push_str(&format!(
            "Version:        {}\n",
            env!("CARGO_PKG_VERSION")
        ));

        log.push_str(&format!(
            "Architecture:   {}\n",
            std::env::consts::ARCH
        ));

        log.push_str(&format!(
            "OS:             {}\n",
            std::env::consts::OS
        ));

        log.push_str(&format!(
            "OS Family:      {}\n",
            std::env::consts::FAMILY
        ));

        // =========================================================
        // Thread
        // =========================================================

        log.push_str("\n[Thread]\n");

        let current_thread = std::thread::current();

        log.push_str(&format!(
            "Name:           {}\n",
            current_thread.name().unwrap_or("<unnamed>")
        ));

        log.push_str(&format!(
            "ID:             {:?}\n",
            current_thread.id()
        ));

        // =========================================================
        // Panic
        // =========================================================

        log.push_str("\n[Panic]\n");

        if let Some(location) = info.location() {
            log.push_str(&format!(
                "File:           {}\n",
                location.file()
            ));

            log.push_str(&format!(
                "Line:           {}\n",
                location.line()
            ));

            log.push_str(&format!(
                "Column:         {}\n",
                location.column()
            ));

            log.push_str(&format!(
                "Location:       {}:{}:{}\n",
                location.file(),
                location.line(),
                location.column()
            ));
        } else {
            log.push_str("Location:       <unknown>\n");
        }

        if let Some(message) = info.payload().downcast_ref::<&str>() {
            log.push_str(&format!(
                "Message:        {}\n",
                message
            ));
        } else if let Some(message) = info.payload().downcast_ref::<String>() {
            log.push_str(&format!(
                "Message:        {}\n",
                message
            ));
        } else {
            log.push_str("Message:        <unknown>\n");
        }

        // =========================================================
        // Environment
        // =========================================================

        log.push_str("\n[Environment]\n");

        let current_dir = std::env::current_dir()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| "<unknown>".to_string());

        log.push_str(&format!(
            "Current Dir:    {}\n",
            current_dir
        ));

        let config = config_dir();

        log.push_str(&format!(
            "Config Dir:     {}\n",
            config.display()
        ));

        log.push_str(&format!(
            "RUST_BACKTRACE: {}\n",
            std::env::var("RUST_BACKTRACE")
                .unwrap_or_else(|_| "<not set>".to_string())
        ));

        // =========================================================
        // 结束
        // =========================================================

        log.push_str("\n============================================================\n");
        log.push_str("                    END CRASH REPORT\n");
        log.push_str("============================================================\n");

        // =========================================================
        // 控制台输出
        // =========================================================

        eprintln!("{}", log);

        // =========================================================
        // 获取配置目录
        // =========================================================

        let dir = config_dir();

        if let Err(error) = fs::create_dir_all(&dir) {
            eprintln!(
                "NUCtool: failed to create crash log directory '{}': {}",
                dir.display(),
                error
            );

            return;
        }

        // =========================================================
        // 清理旧日志
        //
        // 如果已有 >= 30 个：
        //     删除最旧的 10 个
        //
        // 然后再创建本次日志。
        // =========================================================

        cleanup_old_crash_logs(&dir);

        // =========================================================
        // 创建本次 Crash Log
        // =========================================================

        let filename = format!(
            "crash_{}.log",
            timestamp
        );

        let path = dir.join(filename);

        match OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
        {
            Ok(mut file) => {
                if let Err(error) = file.write_all(log.as_bytes()) {
                    eprintln!(
                        "NUCtool: failed to write crash log '{}': {}",
                        path.display(),
                        error
                    );
                }

                if let Err(error) = file.flush() {
                    eprintln!(
                        "NUCtool: failed to flush crash log '{}': {}",
                        path.display(),
                        error
                    );
                }
            }

            Err(error) => {
                eprintln!(
                    "NUCtool: failed to create crash log '{}': {}",
                    path.display(),
                    error
                );
            }
        }
    }));
}

/// 清理旧的 Crash Log。
///
/// 规则：
///
///     >= 30 个
///         ↓
///     删除最旧的 10 个
///
/// 文件名格式：
///
///     crash_YYYYMMDD_HHMMSS_mmm.log
///
/// 因此按文件名排序即可得到时间顺序。
fn cleanup_old_crash_logs(dir: &std::path::Path) {
    let mut logs = Vec::new();

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,

        Err(error) => {
            eprintln!(
                "NUCtool: failed to read crash log directory '{}': {}",
                dir.display(),
                error
            );

            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // 只处理：
        //
        // crash_*.log
        //
        let is_crash_log = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| {
                name.starts_with("crash_")
                    && name.ends_with(".log")
            })
            .unwrap_or(false);

        if is_crash_log {
            logs.push(path);
        }
    }

    // 数量没有达到 30，不需要清理。
    if logs.len() < 30 {
        return;
    }

    // =========================================================
    // 按文件名排序
    //
    // crash_20261008_070000_001.log
    // crash_20261008_071000_002.log
    // crash_20261008_072000_003.log
    //
    // 越靠前越旧。
    // =========================================================

    logs.sort_by(|a, b| {
        a.file_name()
            .cmp(&b.file_name())
    });

    // =========================================================
    // 删除最旧的 10 个
    // =========================================================

    let delete_count = 10.min(logs.len());

    for path in logs.into_iter().take(delete_count) {
        if let Err(error) = fs::remove_file(&path) {
            eprintln!(
                "NUCtool: failed to delete old crash log '{}': {}",
                path.display(),
                error
            );
        }
    }
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
