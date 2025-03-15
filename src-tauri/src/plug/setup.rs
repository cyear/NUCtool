use std::error::Error;
use std::{fs, process, thread, time::Duration};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::plugin::PermissionState;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, Manager};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::plug::{
    config::get_config_dir,
    fan::fan_reset
};

async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    let up: Update;
    match app.updater() {
        Ok(a) => match a.check().await {
            Ok(Some(update)) => {
                up = update;
            }
            Err(e) => {
                println!("update check failed: {:#?}", e);
                return Ok(());
            }
            _ => return Ok(())
        },
        Err(e) => {
            println!("Updater Error: {:#?}", e);
            return Ok(());
        }
    }
    println!("update found: {:#?}", up.body);
    app.dialog()
        .message(up.clone().body.unwrap().as_str())
        .title("NUCtool 有新版本 v".to_owned() + up.clone().version.as_str())
        .buttons(MessageDialogButtons::OkCancelCustom(
            "更新".to_string(),
            "取消".to_string(),
        ))
        .show(|yes| match yes {
            true => {
                tauri::async_runtime::spawn(async move {
                    let mut downloaded = 0;
                    up.download_and_install(
                        |chunk_length, content_length| {
                            downloaded += chunk_length;
                            println!("downloaded {downloaded} from {content_length:?}");
                        },
                        || {
                            println!("download finished");
                        },
                    )
                    .await
                    .expect("download failed");
                    println!("update installed");
                    app.restart();
                });
            }
            false => {
                println!("update canceled")
            }
        });
    Ok(())
}

pub fn init(app: &mut App) -> Result<(), Box<dyn Error>> {
    #[cfg(debug_assertions)] // only include this code on debug builds
    {
      let window = app.get_webview_window("main").unwrap();
      window.open_devtools();
      window.close_devtools();
    }
    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        match update(handle).await {
            Ok(_) => {},
            Err(e) => {
                println!("update error: {:#?}", e);
            }
        }
    });
    // 自启动
    let autostart_manager = app.autolaunch();
    println!("自启动状态: {}", autostart_manager.is_enabled()?);
    let config_dir = get_config_dir();
    fs::create_dir_all(&config_dir).unwrap();
    let config_beta = config_dir.join("beta.config");
    if !config_beta.exists() {
        println!("beta.config 配置文件不存在");
        fs::write(&config_beta, "0").map_err(|e| e.to_string())?;
    }
    if config_beta.exists() {
        let beta = fs::read_to_string(config_beta)
            .map_err(|e| e.to_string())?
            .parse::<i64>()?;
        if beta == 1 {
            let _ = autostart_manager.enable();
        } else {
            let _ = autostart_manager.disable();
        }
    }
    let window = app.get_webview_window("main").unwrap();
    let h = MenuItemBuilder::with_id("h", "显示界面").build(app)?;
    let q = MenuItemBuilder::with_id("q", "退出程序").build(app)?;
    let d = MenuItemBuilder::with_id("d", "调试模式").build(app)?;
    let menu = MenuBuilder::new(app).items(&[&h, &d, &q]).build()?;
    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        // .title("NUCtool")
        .icon(app.default_window_icon().unwrap().clone())
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "h" => {
                println!("显示 clicked");
                if let Some(webview_window) = app.get_webview_window("main") {
                    let _ = webview_window.show();
                    let _ = webview_window.set_focus();
                }
            }
            "q" => {
                if app.notification().permission_state().unwrap() == PermissionState::Granted {
                    app.notification()
                        .builder()
                        .body("安全退出！")
                        .show()
                        .unwrap();
                }
                thread::spawn(move || {
                    thread::sleep(Duration::from_secs(1));
                    fan_reset();
                    println!("退出");
                    process::exit(0);
                });
            }
            "d" => {
                if let Some(webview_window) = app.get_webview_window("tdp") {
                    let _ = webview_window.show();
                    let _ = webview_window.set_focus();
                }
            }
            _ => (),
        })
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
              button: MouseButton::Left,
              button_state: MouseButtonState::Up,
              ..
            } => {
              println!("left click pressed and released");
              let app = tray.app_handle();
              if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
              }
            }
            _ => {
                // println!("unhandled event {event:?}");
            }
        }).build(app)?;
    #[cfg(windows)]
    window_vibrancy::apply_acrylic(&window, Some((18, 18, 18, 125)))
        .expect("Unsupported platform! 'apply_blur' is only supported on Windows");
    #[cfg(windows)]
    window_vibrancy::apply_acrylic(&app.get_webview_window("tdp").unwrap(), Some((18, 18, 18, 125)))
        .expect("Unsupported platform! 'apply_blur' is only supported on Windows");
    Ok(())
}
