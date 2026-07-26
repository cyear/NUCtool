//! Tauri 命令层
//!
//! 全部命令改为异步执行 — 旧版同步命令会在**主线程**上执行,
//! `set_tdp` 内部 2 秒的写入间隔曾直接冻结整个界面
//! (对应 分析.md 中"程序界面卡死无响应")。

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use colored::Colorize;
use tauri::{Emitter, State, WebviewWindow};

use crate::{
    config::{self, FanData},
    fan_control::{self, FanControlState},
    hw::{hw, Tdp},
};

/// 周期性推送风扇转速与温度。
/// 全局仅启动一个推送线程 — 旧版每次调用都会泄漏一个新线程。
#[tauri::command]
pub async fn get_fan_speeds(window: WebviewWindow) {
    static STARTED: AtomicBool = AtomicBool::new(false);
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    let _ = std::thread::Builder::new()
        .name("fan-push".into())
        .spawn(move || {
            println!("{}", "开始推送风扇信息".green());
            loop {
                std::thread::sleep(std::time::Duration::from_secs_f64(2.5));
                // 窗口隐藏(仅托盘)时不采集不推送
                if !window.is_visible().unwrap_or(true) {
                    continue;
                }
                if window.emit("get-fan-speeds", hw().fan_speeds()).is_err() {
                    break;
                }
            }
            STARTED.store(false, Ordering::SeqCst);
        });
}

/// 按前端曲线启动风扇控制
#[tauri::command]
pub async fn start_fan_control(
    fan_data: FanData,
    state: State<'_, FanControlState>,
) -> Result<(), String> {
    fan_control::start(&fan_data, Arc::clone(&state.running))
}

/// 停止风扇控制并恢复自动
#[tauri::command]
pub async fn stop_fan_control(state: State<'_, FanControlState>) -> Result<(), String> {
    fan_control::stop(&state.running);
    Ok(())
}

#[tauri::command]
pub async fn save_fan_config(fan_data: FanData) -> Result<(), String> {
    config::save(&fan_data)
}

#[tauri::command]
pub async fn load_fan_config() -> Result<FanData, String> {
    config::load()
}

/// 读取 TDP, 返回 (cpu1, cpu2, gpu1, gpu2, tcc) — 与前端解构顺序一致
#[tauri::command]
pub async fn get_tdp() -> Result<(i64, i64, i64, i64, i64), String> {
    let t = tauri::async_runtime::spawn_blocking(|| hw().get_tdp())
        .await
        .map_err(|e| e.to_string())?;
    Ok((t.cpu1, t.cpu2, t.gpu1, t.gpu2, t.tcc))
}

/// 写入 TDP(内部含约 2s 写入间隔, 放到阻塞线程池执行)
#[tauri::command]
pub async fn set_tdp(t: Tdp) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || hw().set_tdp(t))
        .await
        .map_err(|e| e.to_string())
}

/// 自定义 RGB 颜色(硬件调用方法待逆向, 与旧版一致仅打印)
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Rgb {
    pub r: i64,
    pub g: i64,
    pub b: i64,
}

#[tauri::command]
pub async fn set_rgb(rgb: Rgb) {
    println!("set_rgb(暂未实现): {rgb:?}");
}

#[tauri::command]
pub async fn get_rgb() -> Rgb {
    Rgb { r: 0, g: 0, b: 0 }
}

/// 开启键盘彩色模式(AC 供电)
#[tauri::command]
pub async fn set_rgb_color_y() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(|| hw().set_led_ac(true))
        .await
        .map_err(|e| e.to_string())
}

/// 关闭键盘彩色模式(AC 供电)
#[tauri::command]
pub async fn set_rgb_color_n() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(|| hw().set_led_ac(false))
        .await
        .map_err(|e| e.to_string())
}

/// 键盘彩色模式是否开启
#[tauri::command]
pub async fn get_rgb_color() -> Result<bool, String> {
    let state = tauri::async_runtime::spawn_blocking(|| hw().led_ac_state())
        .await
        .map_err(|e| e.to_string())?;
    Ok(state == 2)
}
