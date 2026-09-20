use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::{
    Emitter,
    AppHandle,
    Manager,
    WebviewUrl,
    WebviewWindow,
    WebviewWindowBuilder,
};

static OSD_READY: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize)]
pub struct OsdPayload {
    pub title: String,
    pub subtitle: String,
}

/// 创建 OSD 窗口
///
/// 程序启动时调用一次。
pub fn create_osd(app: &AppHandle) -> Result<(), String> {
    // 已经存在就不重复创建
    if app.get_webview_window("osd").is_some() {
        println!("OSD 已存在 跳过创建");
        return Ok(());
    }

    // 创建 OSD 窗口
    let window = WebviewWindowBuilder::new(
        app,
        "osd",
        WebviewUrl::App("osd.html".into()),
    )
    .title("NUCtool OSD")
    .inner_size(300.0, 64.0)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .focused(false)
    .visible(false)
    .build()
    .map_err(|e| e.to_string())?;

    // 不允许 OSD 获得焦点
    window
        .set_focusable(false)
        .map_err(|e| e.to_string())?;

    // 鼠标穿透
    window
        .set_ignore_cursor_events(true)
        .map_err(|e| e.to_string())?;

    // 定位
    position_osd(&window)?;
    println!("OSD 创建成功");
    Ok(())
}

/// OSD 前端加载完成后调用
///
/// 这个 command 由 osd.js 调用。
#[tauri::command]
pub fn osd_ready() {
    OSD_READY.store(true, Ordering::Release);
}

/// Rust 内部使用的 OSD API
///
/// 其他 Rust 函数只需要：
///
/// show_osd(app, "性能模式", "Performance Mode")?;
pub fn show_osd(
    app: &AppHandle,
    title: impl Into<String>,
    subtitle: impl Into<String>,
) -> Result<(), String> {
    // WebView 还没有加载完成
    if !OSD_READY.load(Ordering::Acquire) {
        return Err("OSD is not ready".to_string());
    }

    let window = app
        .get_webview_window("osd")
        .ok_or_else(|| "OSD window not found".to_string())?;

    // 每次显示之前重新定位
    position_osd(&window)?;

    let payload = OsdPayload {
        title: title.into(),
        subtitle: subtitle.into(),
    };

    // 发送给 osd.js
    window
        .emit("osd-show", payload)
        .map_err(|e| e.to_string())?;

    // 显示窗口
    window.show().map_err(|e| e.to_string())?;

    Ok(())
}

/// Tauri command
///
/// JS 可以这样调用：
///
/// invoke("show_osd", {
///     title: "性能模式",
///     subtitle: "Performance Mode"
/// })
#[tauri::command]
pub fn show_osd_command(
    app: AppHandle,
    title: String,
    subtitle: Option<String>,
) -> Result<(), String> {
    show_osd(
        &app,
        title,
        subtitle.unwrap_or_default(),
    )
}

/// 将 OSD 放到当前显示器左上角
fn position_osd(window: &WebviewWindow) -> Result<(), String> {
    let monitor = window
        .current_monitor()
        .map_err(|e| e.to_string())?
        .or_else(|| {
            window
                .primary_monitor()
                .ok()
                .flatten()
        });

    let Some(monitor) = monitor else {
        // 获取不到显示器信息时的 fallback
        window
            .set_position(tauri::Position::Physical(
                tauri::PhysicalPosition::new(20, 20),
            ))
            .map_err(|e| e.to_string())?;

        return Ok(());
    };

    let position = monitor.position();

    let x = position.x + 56;
    let y = position.y + 76;

    window
        .set_position(tauri::Position::Physical(
            tauri::PhysicalPosition::new(x, y),
        ))
        .map_err(|e| e.to_string())?;

    Ok(())
}