#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use std::{
    env,
    thread,
    sync::{Arc, Mutex}
};
use color_eyre::{
    // eyre::eyre,
    Result
};
use tracing::instrument;
use tracing_appender::{
    non_blocking,
    rolling::{
        Rotation,
        RollingFileAppender
    }
};
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, Registry,
};

#[cfg(windows)]
mod win_plug;
#[cfg(unix)]
mod linux_plug;
mod plug;
mod tests;

use plug::{
    setup,
    struct_set::FanControlState,
    config::{save_fan_config, load_fan_config},
    fan::{get_fan_speeds, start_fan_control, stop_fan_control},
    tdp::{get_tdp, set_tdp, set_rgb, get_rgb, set_rgb_color_y, set_rgb_color_n, get_rgb_color},
};

#[cfg(windows)]
use win_plug::{
    permissions::privilege_escalation,
    wmi::wmi_security,
};
#[cfg(unix)]
use linux_plug::{
    sysfs::sys_init,
};

#[instrument]
fn main() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // 输出到控制台中
    let formatting_layer = fmt::layer().pretty().with_writer(std::io::stderr);
    // 输出到文件中
    let file_appender = RollingFileAppender::builder()
        .max_log_files(60)
        .rotation(Rotation::HOURLY)
        .filename_prefix("NUCtool.LOG")
        .filename_suffix("log")
        .build("target/LOG/")?;
    let (non_blocking_appender, _guard) = non_blocking(file_appender);
    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking_appender);

    // 注册
    Registry::default()
        .with(env_filter)
        .with(ErrorLayer::default())
        .with(formatting_layer)
        .with(file_layer)
        .init();

    color_eyre::install()?;

    let builder = tauri::Builder::default();
    // #[cfg(debug_assertions)]
    // {
    //     let devtools = tauri_plugin_devtools::init();
    //     builder = builder.plugin(devtools);
    // }
    #[cfg(windows)]
    privilege_escalation();
    #[cfg(windows)]
    thread::spawn(move || {
        wmi_security();
        plug::fan::fan_reset();
    });
    #[cfg(unix)]
    sys_init();
    let fan_control_state = FanControlState {
        is_running: Arc::new(Mutex::new(false)),
    };
    builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]))
        )
        .manage(fan_control_state)
        .setup(|app| setup::init(app))
        .invoke_handler(tauri::generate_handler![
            start_fan_control, stop_fan_control, save_fan_config,
            load_fan_config, get_fan_speeds, get_tdp,
            set_tdp, set_rgb, get_rgb,
            set_rgb_color_y, set_rgb_color_n, get_rgb_color
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())?;
    Ok(())
}
