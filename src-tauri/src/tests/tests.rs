#[cfg(all(test, windows))]
use crate::win_plug::{
    wmi::wmi_security,
};
#[cfg(test)]
use std::{
    thread::sleep,
    time::Duration
};
#[cfg(test)]
use crate::plug::struct_set::ApiFan;

#[test]
fn test_api_fan() {
    println!("请随时准备好你的NUC控制台基准模式，出现异常请打开基准模式");
    wmi_security();
    let api = ApiFan::init();
    api.set_fan_auto();
    sleep(Duration::from_secs(1));
    assert_eq!(api.get_fan_mode(), 2);

    api.set_fan_control();
    sleep(Duration::from_secs(1));
    sleep(Duration::from_secs(1));
    assert_eq!(api.get_fan_mode(), 1);

    api.set_fan(0, 0);
    sleep(Duration::from_secs(2));
    assert_eq!(api.get_fan_l(), 0);
    assert_eq!(api.get_fan_r(), 0);

    #[cfg(windows)]
    api.set_fan(200, 200);
    #[cfg(unix)]
    api.set_fan(255, 255);
    sleep(Duration::from_secs(1));
    assert_ne!(api.get_fan_l(), 0);
    assert_ne!(api.get_fan_r(), 0);

    api.set_fan_auto();
    sleep(Duration::from_secs(1));
    assert_eq!(api.get_fan_mode(), 2);
}

#[test]
#[cfg(windows)]
fn led_color() {
    println!("请查看你的LED");
    wmi_security();
    let api = ApiFan::init();
    api.set_ac_led_color_y();
    sleep(Duration::from_secs(3));
    assert_eq!(api.get_ac_led_color(), 2);
    
    api.set_ac_led_color_n();
    sleep(Duration::from_secs(3));
    assert_eq!(api.get_ac_led_color(), 1);
    
}