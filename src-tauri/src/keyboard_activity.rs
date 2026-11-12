//! Passive system keyboard activity tracking through Windows Raw Input.
//! The hidden window receives input notifications without consuming keystrokes.

use std::{
    ffi::c_void,
    mem,
    ptr,
    sync::{mpsc, Mutex, OnceLock},
    thread,
    time::Instant,
};

type Hwnd = *mut c_void;
type Handle = *mut c_void;
type Wparam = usize;
type Lparam = isize;
type Lresult = isize;

#[repr(C)]
struct RawInputDevice {
    usage_page: u16,
    usage: u16,
    flags: u32,
    target: Hwnd,
}

#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
struct Message {
    hwnd: Hwnd,
    message: u32,
    wparam: Wparam,
    lparam: Lparam,
    time: u32,
    point: Point,
    private: u32,
}

#[repr(C)]
struct WindowClass {
    style: u32,
    window_proc: Option<unsafe extern "system" fn(Hwnd, u32, Wparam, Lparam) -> Lresult>,
    class_extra: i32,
    window_extra: i32,
    instance: Handle,
    icon: Handle,
    cursor: Handle,
    background: Handle,
    menu_name: *const u16,
    class_name: *const u16,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassW(class: *const WindowClass) -> u16;
    fn CreateWindowExW(
        ex_style: u32,
        class_name: *const u16,
        window_name: *const u16,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: Hwnd,
        menu: Handle,
        instance: Handle,
        parameter: *const c_void,
    ) -> Hwnd;
    fn DefWindowProcW(hwnd: Hwnd, message: u32, wparam: Wparam, lparam: Lparam) -> Lresult;
    fn GetMessageW(message: *mut Message, hwnd: Hwnd, min: u32, max: u32) -> i32;
    fn TranslateMessage(message: *const Message) -> i32;
    fn DispatchMessageW(message: *const Message) -> Lresult;
    fn RegisterRawInputDevices(device: *const RawInputDevice, count: u32, size: u32) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> Handle;
}

static LAST_KEYBOARD_ACTIVITY: OnceLock<Mutex<Instant>> = OnceLock::new();
static MONITOR: OnceLock<Result<(), String>> = OnceLock::new();

unsafe extern "system" fn window_proc(
    hwnd: Hwnd,
    message: u32,
    wparam: Wparam,
    lparam: Lparam,
) -> Lresult {
    if message == 0x00ff {
        if let Some(last_activity) = LAST_KEYBOARD_ACTIVITY.get() {
            if let Ok(mut last_activity) = last_activity.lock() {
                *last_activity = Instant::now();
            }
        }
    }
    DefWindowProcW(hwnd, message, wparam, lparam)
}

pub fn idle_seconds() -> Result<u32, String> {
    MONITOR.get_or_init(start_monitor).clone()?;
    let last_activity = LAST_KEYBOARD_ACTIVITY
        .get()
        .ok_or_else(|| "键盘活动监视器未初始化".to_string())?
        .lock()
        .map_err(|_| "键盘活动监视器状态锁定失败".to_string())?;
    Ok(last_activity.elapsed().as_secs().min(u64::from(u32::MAX)) as u32)
}

fn start_monitor() -> Result<(), String> {
    LAST_KEYBOARD_ACTIVITY.get_or_init(|| Mutex::new(Instant::now()));
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);

    thread::Builder::new()
        .name("nuctool-keyboard-activity".into())
        .spawn(move || unsafe {
            let instance = GetModuleHandleW(ptr::null());
            let class_name: Vec<u16> = "NUCtoolKeyboardActivityWindow\0".encode_utf16().collect();
            let class = WindowClass {
                style: 0,
                window_proc: Some(window_proc),
                class_extra: 0,
                window_extra: 0,
                instance,
                icon: ptr::null_mut(),
                cursor: ptr::null_mut(),
                background: ptr::null_mut(),
                menu_name: ptr::null(),
                class_name: class_name.as_ptr(),
            };
            if RegisterClassW(&class) == 0 {
                let _ = ready_tx.send(Err("注册键盘活动窗口类失败".into()));
                return;
            }
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                class_name.as_ptr(),
                0,
                0,
                0,
                0,
                0,
                ptr::null_mut(),
                ptr::null_mut(),
                instance,
                ptr::null(),
            );
            if hwnd.is_null() {
                let _ = ready_tx.send(Err("创建键盘活动监视窗口失败".into()));
                return;
            }

            let device = RawInputDevice {
                usage_page: 1,
                usage: 6,
                flags: 0x0000_0100, // RIDEV_INPUTSINK: receive passively in background.
                target: hwnd,
            };
            if RegisterRawInputDevices(
                &device,
                1,
                mem::size_of::<RawInputDevice>() as u32,
            ) == 0
            {
                let _ = ready_tx.send(Err("注册 Windows Raw Input 键盘监听失败".into()));
                return;
            }

            println!("[KeyboardHID] 键盘活动监视器已启动（Raw Input，不拦截按键）");
            let _ = ready_tx.send(Ok(()));
            let mut message: Message = mem::zeroed();
            while GetMessageW(&mut message, ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        })
        .map_err(|error| format!("启动键盘活动监视线程失败: {error}"))?;

    ready_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .map_err(|error| format!("等待键盘活动监视器启动超时: {error}"))?
}
