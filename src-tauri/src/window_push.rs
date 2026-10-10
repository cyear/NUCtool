use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tauri::{AppHandle, Manager};

/// NUCtool 全局窗口推送状态。
///
/// enabled = true：允许向前端推送状态。
/// enabled = false：暂停向前端推送状态。
#[derive(Clone, Default)]
pub struct WindowPushState {
    enabled: Arc<AtomicBool>,
}

impl WindowPushState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn disable(&self) {
        self.set_enabled(false);
    }
}


/// 启动窗口状态监控
/// 窗口可见且未最小化时，允许读取 EC 和向前端推送状态
/// 不要求窗口获得焦点
pub fn start_window_push_monitor(app: AppHandle) {
    let state = app.state::<WindowPushState>().inner().clone();

    std::thread::Builder::new()
        .name("window-push-monitor".into())
        .spawn(move || {
            let mut last_enabled: Option<bool> = None;

            loop {
                let (enabled, visible, minimized) =
                    match app.get_webview_window("main") {
                        Some(window) => {
                            let visible =
                                window.is_visible().unwrap_or(false);
                            let minimized =
                                window.is_minimized().unwrap_or(true);

                            (visible && !minimized, visible, minimized)
                        }
                        None => (false, false, true),
                    };

                state.set_enabled(enabled);

                if last_enabled != Some(enabled) {
                    println!(
                        "[WindowPush] enabled={}, visible={}, minimized={}",
                        enabled, visible, minimized
                    );

                    last_enabled = Some(enabled);
                }

                std::thread::sleep(
                    std::time::Duration::from_millis(250),
                );
            }
        })
        .expect("无法启动窗口推送监控线程");
}
