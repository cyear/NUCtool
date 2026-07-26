//! 硬件在环测试(需要 NUC X15 真机, 建议 `cargo test -- --test-threads=1`)
//!
//! 纯逻辑单元测试见 `fan_control.rs` 与 `hw/registers.rs` 内的 `#[cfg(test)]` 模块。

use std::{thread::sleep, time::Duration};

use crate::hw::hw;

#[test]
fn fan_control_cycle() {
    println!("请随时准备好你的NUC控制台基准模式, 出现异常请打开基准模式");
    let hw = hw();

    assert!(hw.fan_auto_blocking());
    sleep(Duration::from_secs(1));
    assert_eq!(hw.fan_mode(), 2);

    assert!(hw.fan_manual());
    sleep(Duration::from_secs(2));
    assert_eq!(hw.fan_mode(), 1);

    assert!(hw.set_fan(0, 0));
    sleep(Duration::from_secs(2));
    let s = hw.fan_speeds();
    assert_eq!(s.left_fan_speed, 0);
    assert_eq!(s.right_fan_speed, 0);

    assert!(hw.set_fan(200, 200));
    sleep(Duration::from_secs(2));
    let s = hw.fan_speeds();
    assert_ne!(s.left_fan_speed, 0);
    assert_ne!(s.right_fan_speed, 0);

    assert!(hw.fan_auto_blocking());
    sleep(Duration::from_secs(1));
    assert_eq!(hw.fan_mode(), 2);
}

#[test]
fn led_color_cycle() {
    println!("请查看你的键盘 LED");
    let hw = hw();

    assert!(hw.set_led_ac(true));
    sleep(Duration::from_secs(3));
    assert_eq!(hw.led_ac_state(), 2);

    assert!(hw.set_led_ac(false));
    sleep(Duration::from_secs(3));
    assert_eq!(hw.led_ac_state(), 1);
}
