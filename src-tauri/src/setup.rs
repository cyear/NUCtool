//! 应用初始化: 系统托盘、自启动、更新检查、窗口特效

use std::{error::Error, process, thread, time::Duration};

use colored::Colorize;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    plugin::PermissionState,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::UpdaterExt;

use crate::{config, hw::hw};

pub fn init(app: &mut App) -> Result<(), Box<dyn Error>> {
    #[cfg(debug_assertions)]
    if let Some(w) = app.get_webview_window("main") {
        w.open_devtools();
    }

    // 后台检查更新
    tauri::async_runtime::spawn(check_update(app.handle().clone()));

    // 自启动: beta.config 为 "1" 时启用
    let autostart = app.autolaunch();
    if config::autostart_flag() {
        if let Err(e) = autostart.enable() {
            println!("启用自启动失败: {e:?}");
        }
    } else {
        let _ = autostart.disable();
    }
    println!("自启动状态: {:?}", autostart.is_enabled());

    // 系统托盘
    let show = MenuItemBuilder::with_id("show", "显示界面").build(app)?;
    let debug = MenuItemBuilder::with_id("debug", "调试模式").build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "退出程序").build(app)?;
    let menu = MenuBuilder::new(app)
        .items(&[&show, &debug, &quit_item])
        .build()?;
    TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .icon(app.default_window_icon().ok_or("缺少应用图标")?.clone())
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_window(app, "main"),
            "debug" => show_window(app, "tdp"),
            "quit" => quit(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_window(tray.app_handle(), "main");
            }
        })
        .build(app)?;

    // 窗口亚克力效果(失败不影响使用 — 旧版此处 expect 会直接崩溃)
    for label in ["main", "tdp"] {
        if let Some(w) = app.get_webview_window(label) {
            if let Err(e) = window_vibrancy::apply_acrylic(&w, Some((18, 18, 18, 125))) {
                println!("窗口 {label} 亚克力效果应用失败(不影响使用): {e:?}");
            }
        }
    }
    Ok(())
}

fn show_window(app: &AppHandle, label: &str) {
    if let Some(w) = app.get_webview_window(label) {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// 托盘退出: 先把风扇复位为自动, 再退出进程
fn quit(app: &AppHandle) {
    println!("退出请求");
    let granted = app
        .notification()
        .permission_state()
        .map(|s| s == PermissionState::Granted)
        .unwrap_or(false);
    if granted {
        let _ = app
            .notification()
            .builder()
            .body("风扇已恢复自动, 安全退出")
            .show();
    }
    thread::spawn(|| {
        // 阻塞等待复位完成后再退出(旧版为"睡 1 秒再复位", 存在退出前未复位的风险)
        hw().fan_auto_blocking();
        thread::sleep(Duration::from_millis(300));
        println!("退出");
        process::exit(0);
    });
}

/// 检查更新, 有新版本时询问用户
async fn check_update(app: AppHandle) {
    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => {
            println!("更新器初始化失败: {e:?}");
            return;
        }
    };
    let update = match updater.check().await {
        Ok(Some(u)) => u,
        Ok(None) => {
            println!("当前已是最新版本");
            return;
        }
        Err(e) => {
            // 无网络等情况: 静默跳过(旧版曾因此闪退, 0.3.6 修复过一次)
            println!("检查更新失败: {e:?}");
            return;
        }
    };
    println!("发现新版本: v{}", update.version);
    let body = update.body.clone().unwrap_or_default();
    let title = format!("NUCtool 有新版本 v{}", update.version);
    app.dialog()
        .message(body)
        .title(title)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "更新".to_string(),
            "取消".to_string(),
        ))
        .show(move |yes| {
            if !yes {
                println!("用户取消更新");
                return;
            }
            tauri::async_runtime::spawn(async move {
                let mut downloaded = 0usize;
                let result = update
                    .download_and_install(
                        |chunk, total| {
                            downloaded += chunk;
                            println!("已下载 {downloaded} / {total:?}");
                        },
                        || println!("下载完成"),
                    )
                    .await;
                match result {
                    Ok(()) => {
                        println!("更新完成, 重启应用");
                        app.restart();
                    }
                    Err(e) => println!("更新失败: {e:?}"),
                }
            });
        });
}
