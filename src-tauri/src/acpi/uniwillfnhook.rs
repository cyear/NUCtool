use crate::acpi::uniwillwcf::UniwillWcfEc;
use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::{
    Foundation::{
        // HINSTANCE,
        LPARAM,
        LRESULT,
        WPARAM,
    },
    UI::WindowsAndMessaging::{
        CallNextHookEx,
        // DispatchMessageW,
        // GetMessageW,
        // SetWindowsHookExW,
        // UnhookWindowsHookEx,
        HC_ACTION,
        KBDLLHOOKSTRUCT,
        // MSG,
        // WH_KEYBOARD_LL,
        WM_KEYDOWN,
        WM_KEYUP,
    },
};

// ==========================================
// Fn 状态
// ==========================================

static FN_DOWN: AtomicBool = AtomicBool::new(false);

// ==========================================
// Low Level Keyboard Hook
// ==========================================

pub unsafe extern "system" fn uniwillfnkeyhook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let keyboard = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };

        let vk = keyboard.vkCode as u16;

        let scan_code = keyboard.scanCode as u16;

        // ==================================
        // Fn
        //
        // VK = FF
        // ScanCode = 78
        // ==================================

        if vk == 0x00FF && scan_code == 0x0078 {
            match wparam.0 as u32 {
                // Fn Down
                WM_KEYDOWN => {
                    FN_DOWN.store(true, Ordering::Relaxed);

                    // println!("Fn Down");
                }

                // Fn Up
                WM_KEYUP => {
                    FN_DOWN.store(false, Ordering::Relaxed);

                    // println!("Fn Up");
                }

                _ => {}
            }

            // Fn 本身不继续传递
            return LRESULT(1);
        }

        // ==================================
        // 1 ~ 9
        // ==================================

        if (0x31..=0x39).contains(&vk) {
            let is_down = wparam.0 as u32 == WM_KEYDOWN;

            if is_down && FN_DOWN.load(Ordering::Relaxed) {
                let number = vk - 0x30;

                println!("检测到 Fn + {}", number);
                let wcf = match UniwillWcfEc::new() {
                    Ok(wcf) => wcf,
                    Err(e) => {
                        eprintln!("加载 NUCtool DLL 失败: {}", e);
                        None
                    }
                    .expect("加载 NUCtool DLL 失败"),
                };
                let ret = wcf.connect();
                println!("connect: {}", ret);

                handle_fn_number(number, wcf);

                // ==================================
                // 拦截数字
                // 不让 1~9 进入输入框
                // ==================================

                return LRESULT(1);
            }
        }
    }

    // ==================================
    // 其他按键正常传递
    // ==================================

    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

// ==========================================
// Fn + 1~9
// ==========================================

fn handle_fn_number(number: u16, wcf: UniwillWcfEc) {
    match number {
        1 => {
            println!("执行 Fn + 1 功能");
            println!(
                "benchmark_off apply_profile: {}",
                wcf.apply_benchmark_mode(0)
            );
            println!("performance apply_profile: {}", wcf.apply_profile(1));
        }

        2 => {
            println!("执行 Fn + 2 功能");
            println!(
                "benchmark_off apply_profile: {}",
                wcf.apply_benchmark_mode(0)
            );
            println!("balanced apply_profile: {}", wcf.apply_profile(2));
        }

        3 => {
            println!("执行 Fn + 3 功能");
            println!(
                "benchmark_off apply_profile: {}",
                wcf.apply_benchmark_mode(0)
            );
            println!("quiet apply_profile: {}", wcf.apply_profile(3));
        }

        4 => {
            println!("执行 Fn + 4 功能");
            println!(
                "benchmark_on apply_profile: {}",
                wcf.apply_benchmark_mode(1)
            );
        }

        5 => {
            println!("执行 Fn + 5 功能");
        }

        6 => {
            println!("执行 Fn + 6 功能");
        }

        7 => {
            println!("执行 Fn + 7 功能");
        }

        8 => {
            println!("执行 Fn + 8 功能");
        }

        9 => {
            println!("执行 Fn + 9 功能");
        }

        _ => {}
    }
}
