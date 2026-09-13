#![cfg(target_os = "windows")]

use std::mem::transmute;

use tauri::{Runtime, WebviewWindow};

use windows::Win32::Foundation::{
    HWND,
    LPARAM,
    LRESULT,
    WPARAM,
};

use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW,
    DefWindowProcW,
    GetWindowLongPtrW,
    SetWindowLongPtrW,
    GWLP_USERDATA,
    GWLP_WNDPROC,
    WM_ENDSESSION,
    WM_NCDESTROY,
    WM_QUERYENDSESSION,
    WNDPROC,
};

use crate::fan_control::FanControlState;

/// Windows 关机监听上下文
struct ShutdownContext {
    /// 原始 WindowProc
    original_proc: isize,

    /// Tauri 管理的 FanControlState
    state: *const FanControlState,
}

unsafe impl Send for ShutdownContext {}
unsafe impl Sync for ShutdownContext {}

/// 我们自己的 WindowProc
unsafe extern "system" fn shutdown_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let context_ptr = unsafe {
        GetWindowLongPtrW(
            hwnd,
            GWLP_USERDATA,
        )
    };

    if context_ptr == 0 {
        return unsafe {
            DefWindowProcW(
                hwnd,
                msg,
                wparam,
                lparam,
            )
        };
    }

    let context =
        unsafe { &*(context_ptr as *const ShutdownContext) };

    match msg {
        WM_QUERYENDSESSION => {
            println!("Windows 正在请求结束会话...");

            // 不在这里做耗时操作。
            // 允许 Windows 继续关机/重启/注销。
            LRESULT(1)
        }

        WM_ENDSESSION => {
            let ending = wparam.0 != 0;

            println!(
                "收到 WM_ENDSESSION，ending={}",
                ending
            );

            if ending {
                println!(
                    "Windows 即将关闭，停止风扇控制..."
                );

                if !context.state.is_null() {
                    let state =
                        unsafe { &*context.state };

                    // 你的 stop_fan_control_inner
                    // 目前在 lib.rs 根模块，而不是 fan_control.rs。
                    if let Err(e) =
                        crate::stop_fan_control_inner(state)
                    {
                        eprintln!(
                            "Windows 关机时停止风扇控制失败: {}",
                            e
                        );
                    } else {
                        println!(
                            "Windows 关机前风扇控制已停止，已恢复 AutoMode"
                        );
                    }
                }
            }

            call_original_proc(
                context.original_proc,
                hwnd,
                msg,
                wparam,
                lparam,
            )
        }

        WM_NCDESTROY => {
            let result = call_original_proc(
                context.original_proc,
                hwnd,
                msg,
                wparam,
                lparam,
            );

            unsafe {
                SetWindowLongPtrW(
                    hwnd,
                    GWLP_USERDATA,
                    0,
                );

                let _ =
                    Box::from_raw(
                        context_ptr as *mut ShutdownContext
                    );
            }

            result
        }

        _ => call_original_proc(
            context.original_proc,
            hwnd,
            msg,
            wparam,
            lparam,
        ),
    }
}

/// 调用原始 WindowProc
unsafe fn call_original_proc(
    original_proc: isize,
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if original_proc == 0 {
        return unsafe {
            DefWindowProcW(
                hwnd,
                msg,
                wparam,
                lparam,
            )
        };
    }

    let original: WNDPROC =
        unsafe { transmute(original_proc) };

    unsafe {
        CallWindowProcW(
            original,
            hwnd,
            msg,
            wparam,
            lparam,
        )
    }
}

/// 安装 Windows 关机监听
pub fn install<R: Runtime>(
    window: &WebviewWindow<R>,
    state: &FanControlState,
) -> Result<(), String> {
    // Tauri 返回的 HWND 来自 windows 0.61.x。
    let tauri_hwnd = window
        .hwnd()
        .map_err(|e| format!("获取 HWND 失败: {e}"))?;

    let hwnd = HWND(tauri_hwnd.0);

    println!(
        "安装 Windows 关机监听，HWND={:?}",
        hwnd
    );

    let state_ptr =
        state as *const FanControlState;

    unsafe {
        // 获取当前 WindowProc
        let original_proc =
            GetWindowLongPtrW(
                hwnd,
                GWLP_WNDPROC,
            );

        if original_proc == 0 {
            return Err(
                "获取原始 WindowProc 失败"
                    .to_string()
            );
        }

        let context = Box::new(
            ShutdownContext {
                original_proc,
                state: state_ptr,
            },
        );

        let context_ptr =
            Box::into_raw(context);

        // 保存 context
        SetWindowLongPtrW(
            hwnd,
            GWLP_USERDATA,
            context_ptr as isize,
        );

        // 替换 WindowProc
        let previous =
            SetWindowLongPtrW(
                hwnd,
                GWLP_WNDPROC,
                shutdown_wnd_proc as *const () as usize as isize
            );

        if previous == 0 {
            // 安装失败，清理 context
            SetWindowLongPtrW(
                hwnd,
                GWLP_USERDATA,
                0,
            );

            let _ =
                Box::from_raw(context_ptr);

            return Err(
                "安装 Windows WindowProc 失败"
                    .to_string()
            );
        }
    }

    println!(
        "Windows 关机监听安装成功"
    );

    Ok(())
}