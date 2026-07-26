//! NUCtool — Intel NUC X15 风扇/TDP 控制工具 (Windows)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(windows))]
compile_error!("NUCtool 仅支持 Windows 平台");

mod admin;
mod commands;
mod config;
mod fan_control;
mod hw;
mod setup;
#[cfg(test)]
mod tests;

use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

use fan_control::FanControlState;

fn main() {
    // 非管理员时弹 UAC 以管理员身份重启(当前进程退出)
    admin::ensure_elevated();
    // 启动硬件工作线程, 并把风扇恢复为自动模式
    // (COM/WMI 初始化在工作线程内完成, 不阻塞界面启动)
    hw::hw().fan_auto();
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 重复启动时唤起已有窗口(两个实例同时控制 EC 会互相干扰)
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .manage(FanControlState::default())
        .setup(setup::init)
        .invoke_handler(tauri::generate_handler![
            commands::start_fan_control,
            commands::stop_fan_control,
            commands::save_fan_config,
            commands::load_fan_config,
            commands::get_fan_speeds,
            commands::get_tdp,
            commands::set_tdp,
            commands::set_rgb,
            commands::get_rgb,
            commands::set_rgb_color_y,
            commands::set_rgb_color_n,
            commands::get_rgb_color
        ])
        .on_window_event(|window, event| {
            // 关闭窗口时最小化到托盘
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("NUCtool 启动失败");
}
