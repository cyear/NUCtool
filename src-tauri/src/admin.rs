//! 管理员权限检测与自动提权
//!
//! 旧版通过 `powershell_script` 起 PowerShell 进程判断/提权,
//! 启动慢且可能被组策略禁用; 现改为原生 Windows API:
//! Token 提权状态检测 + `ShellExecuteW("runas")` 弹 UAC。

use std::{env, os::windows::ffi::OsStrExt, process};

use colored::Colorize;
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY},
        System::Threading::{GetCurrentProcess, OpenProcessToken},
        UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
    },
};

/// 当前进程是否已提权(UAC elevated)
fn is_elevated() -> bool {
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut ret_len = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret_len,
        );
        let _ = CloseHandle(token);
        ok.is_ok() && elevation.TokenIsElevated != 0
    }
}

/// 非管理员时通过 UAC 以管理员身份重启自身, 并退出当前进程
pub fn ensure_elevated() {
    if is_elevated() {
        println!("{}", "已以管理员身份运行".green());
        return;
    }
    let Ok(exe) = env::current_exe() else {
        eprintln!("无法获取程序路径, 跳过提权");
        return;
    };
    println!("请求管理员权限: {}", exe.display());
    // 路径转为以 \0 结尾的 UTF-16
    let mut wide: Vec<u16> = exe.as_os_str().encode_wide().collect();
    wide.push(0);
    let h = unsafe {
        ShellExecuteW(
            None,
            w!("runas"),
            PCWSTR(wide.as_ptr()),
            None,
            None,
            SW_SHOWNORMAL,
        )
    };
    // 返回值 > 32 表示成功; 失败(如用户取消 UAC)也退出当前进程
    if h.0 as isize <= 32 {
        eprintln!("{}", "提权被取消或失败, 程序退出".red());
    }
    process::exit(0);
}
