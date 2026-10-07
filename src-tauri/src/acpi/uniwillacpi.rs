use crate::config::FanData;
use std::{
    io,
    thread,
    time::Duration,
    ffi::OsStr,
    os::windows::ffi::OsStrExt,
};
use serde::{Deserialize, Serialize};
use windows::{
    core::PCWSTR,
    Win32::{
        System::IO::DeviceIoControl,
        Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
        Storage::FileSystem::{
            CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_READ,
            FILE_SHARE_WRITE, OPEN_EXISTING
        }
    }
};

// ============================================================
// Uniwill ACPIDriver
// ============================================================

const DEVICE_PATH: &str = r"\\.\ACPIDriver";

// EC Read:
//   input  = u32 EC address
//   output = u32 value
const IOCTL_EC_READ: u32 = 0x9C40_A488;

// EC Write:
//   input  = u32 EC address + u32 value
//   output = u32 (unused)
const IOCTL_EC_WRITE: u32 = 0x9C40_A48C;

// ============================================================
// EC Addresses
// ============================================================

// Temperature
const EC_CPU_TEMP: u16 = 0x043E;
const EC_GPU_TEMP: u16 = 0x044F;
const EC_PCH_TEMP: u16 = 0x0E0E;

// Main fan RPM
const EC_MAIN_FAN_RPM_1: u16 = 0x0464;
const EC_MAIN_FAN_RPM_2: u16 = 0x0465;

// Secondary fan RPM
const EC_SECOND_FAN_RPM_1: u16 = 0x046C;
const EC_SECOND_FAN_RPM_2: u16 = 0x046D;

// 启用双风扇独立控制
const EC__FAN_CUSTOM_TABLE_1: u16 = 0x07C5;

// 启用风扇表
const EC__FAN_CUSTOM_TABLE_2: u16 = 0x07C6;

// CPU 温度上升阈值起始地址
pub const EC_FAN_CPU_UP_BASE: u16 = 0x0F00;

// CPU 温度下降阈值起始地址
pub const EC_FAN_CPU_DOWN_BASE: u16 = 0x0F10;

// CPU 风扇占空比起始地址
pub const EC_FAN_CPU_DUTY_BASE: u16 = 0x0F20;

// GPU 温度上升阈值起始地址
pub const EC_FAN_GPU_UP_BASE: u16 = 0x0F30;

// GPU 温度下降阈值起始地址
pub const EC_FAN_GPU_DOWN_BASE: u16 = 0x0F40;

// GPU 风扇占空比起始地址
pub const EC_FAN_GPU_DUTY_BASE: u16 = 0x0F50;


// Windows Flip 1
// const EC_WINDOWS_MODE: u16 = 0x767;

// CPU PL1 (W)
const EC_CPU_PL1: u16 = 0x0783;

// CPU PL2 (W)
const EC_CPU_PL2: u16 = 0x0784;

// CPU PL4 (W)
const EC_CPU_PL4: u16 = 0x0785;

// GPU PL1 (W)
const EC_GPU_PL1: u16 = 0x072d;

// GPU PL2 (W)
const EC_GPU_PL2: u16 = 0x072e;

// PSYS PL1 BYTE1 (W)
// const EC_PSYS_PL1_1: u16 = 0x0720;

// PSYS PL1 BYTE2 (W)
const EC_PSYS_PL1_2: u16 = 0x0721;

// FAN MODE
const EC_ADDR_MANUAL_FAN_CTRL: u16 = 0x0751;

// System Power (W) 疑似
const EC_APC_WATT: u16 = 0x044C;

// AC/input 状态
const _EC_AC_STATUS: u16 = 0x043C;

// Battery Cycle Count
const _EC_BAT_CYCLE: u16 = 0x04A6;

// 跟外接电源性能方案有关写入无效 疑似
const _EC_MDOE_STATE: u16 = 0x07AB;
const _EC_MDOE_STATE_: u16 = 0x07AC;

// 风扇同步 疑似 其他待验证
const _EC_ADDR_AP_OEM_BYTE: u16 = 0x0741;

// AC keyboard state
const EC_KEY_AC_SINGLEBL: u16 = 0x07EA;

// DC keyboard state
const EC_KEY_DC_SINGLEBL: u16 = 0x07EB;

// AC RGB
const EC_KEY_AC_RED: u16 = 0x0769;
const EC_KEY_AC_GREEN: u16 = 0x076A;
const EC_KEY_AC_BLUE: u16 = 0x076B;

// DC RGB
const EC_KEY_DC_RED: u16 = 0x07EC;
const EC_KEY_DC_GREEN: u16 = 0x07ED;
const EC_KEY_DC_BLUE: u16 = 0x07EE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardMode {
    Static,
    Rainbow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyboardBacklight {
    pub enabled: bool,
    /// 亮度
    pub brightness: u8,
    /// Static / Rainbow
    pub rainbow: bool,
    /// RGB 0..50
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

// FanModeByte as u8
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanModeByte {
    // NormalMode = 0x00,
    FanBoostMode = 0x40,
    AutoMode = 0x10,
    // Fuck = 0x50,
}

// ============================================================
// Uniwill EC interface
// ============================================================

pub struct UniwillAcpiEc {
    handle: HANDLE,
}

impl UniwillAcpiEc {
    /// 打开 \\.\ACPIDriver
    pub fn open() -> io::Result<Self> {
        let path: Vec<u16> = OsStr::new(DEVICE_PATH)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let handle = unsafe {
            CreateFileW(
                PCWSTR(path.as_ptr()),
                FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                None,
            )
        }
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("CreateFileW failed: {e:?}")))?;

        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }

        Ok(Self { handle })
    }

    /// 读取一个 EC 8-bit 寄存器
    pub fn read_u8(&self, addr: u16) -> io::Result<u8> {
        // int[] { addr }
        // inBufferSize = 4
        //
        // 因此这里传递一个 u32。
        let input = (addr as u32).to_le_bytes();

        let mut output: u32 = 0;
        let mut bytes_returned = 0u32;

        unsafe {
            DeviceIoControl(
                self.handle,
                IOCTL_EC_READ,
                Some(input.as_ptr() as *const _),
                input.len() as u32,
                Some(&mut output as *mut _ as *mut _),
                std::mem::size_of::<u32>() as u32,
                Some(&mut bytes_returned),
                None,
            )
        }
        .map_err(|_| io::Error::last_os_error())?;

        // Data = Convert.ToByte(outBuffer & 0xFF)
        Ok((output & 0xFF) as u8)
    }

    /// 写入一个 EC 8-bit 寄存器
    pub fn write_u8(&self, addr: u16, value: u8) -> io::Result<()> {
        // int[] { addr, data }
        // inBufferSize = 8
        let input = [(addr as u32).to_le_bytes(), (value as u32).to_le_bytes()].concat();

        let mut output: u32 = 0;
        let mut bytes_returned = 0u32;

        unsafe {
            DeviceIoControl(
                self.handle,
                IOCTL_EC_WRITE,
                Some(input.as_ptr() as *const _),
                input.len() as u32,
                Some(&mut output as *mut _ as *mut _),
                std::mem::size_of::<u32>() as u32,
                Some(&mut bytes_returned),
                None,
            )
        }
        .map_err(|_| io::Error::last_os_error())?;

        Ok(())
    }

    /// 读取两个 EC 字节并按照大端序组合成 u16。
    ///
    /// __be16 + be16_to_cpu()，
    /// 因此 0x0464/0x0465 应按：
    ///
    ///   value = byte1 << 8 | byte2
    ///
    pub fn read_be16(&self, addr_high: u16, addr_low: u16) -> io::Result<u16> {
        let high = self.read_u8(addr_high)?;
        let low = self.read_u8(addr_low)?;

        Ok(u16::from_be_bytes([high, low]))
    }

    /// 读取主风扇
    pub fn main_fan_raw(&self) -> io::Result<u16> {
        self.read_be16(EC_MAIN_FAN_RPM_1, EC_MAIN_FAN_RPM_2)
    }

    /// 读取副风扇
    pub fn second_fan_raw(&self) -> io::Result<u16> {
        self.read_be16(EC_SECOND_FAN_RPM_1, EC_SECOND_FAN_RPM_2)
    }

    /// CPU 温度
    pub fn cpu_temperature(&self) -> io::Result<u8> {
        self.read_u8(EC_CPU_TEMP)
    }

    /// GPU 温度
    pub fn gpu_temperature(&self) -> io::Result<u8> {
        self.read_u8(EC_GPU_TEMP)
    }

    /// Fan1 RPM
    pub fn fan1_rpm(&self) -> io::Result<u16> {
        self.main_fan_raw()
    }

    /// Fan2 RPM
    pub fn fan2_rpm(&self) -> io::Result<u16> {
        self.second_fan_raw()
    }

    // /// 读取 Fan1 两个原始字节
    // pub fn fan1_raw_bytes(&self) -> io::Result<(u8, u8)> {
    //     let high = self.read_u8(EC_MAIN_FAN_RPM_1)?;
    //     let low = self.read_u8(EC_MAIN_FAN_RPM_2)?;

    //     Ok((high, low))
    // }

    // /// 读取 Fan2 两个原始字节
    // pub fn fan2_raw_bytes(&self) -> io::Result<(u8, u8)> {
    //     let high = self.read_u8(EC_SECOND_FAN_RPM_1)?;
    //     let low = self.read_u8(EC_SECOND_FAN_RPM_2)?;

    //     Ok((high, low))
    // }

    /// 读取 Fan Mode
    pub fn fan_read_mode(&self) -> io::Result<u8> {
        self.read_u8(EC_ADDR_MANUAL_FAN_CTRL)
    }

    /// 写入 Fan Mode
    pub fn fan_write_mode(&self, mode: FanModeByte) -> io::Result<()> {
        self.write_u8(EC_ADDR_MANUAL_FAN_CTRL, mode as u8)
    }

    /// 读取 CPU PL1
    pub fn cpu_read_pl1(&self) -> u8 {
        let pl1 = self.read_u8(EC_CPU_PL1).unwrap_or(0);
        if pl1 == 0 {
            0x41
        } else {
            pl1
        }
    }

    /// 写入 CPU PL1
    pub fn cpu_write_pl1(&self, w: u8) -> io::Result<()> {
        self.write_u8(EC_CPU_PL1, w)
    }

    /// 读取 CPU PL2
    pub fn cpu_read_pl2(&self) -> u8 {
        let pl2 = self.read_u8(EC_CPU_PL2).unwrap_or(0);
        if pl2 == 0 {
            0x64
        } else {
            pl2
        }
    }

    /// 写入 CPU PL2
    pub fn cpu_write_pl2(&self, w: u8) -> io::Result<()> {
        self.write_u8(EC_CPU_PL2, w)
    }

    /// 读取 CPU PL4
    pub fn cpu_read_pl4(&self) -> u8 {
        let pl4 = self.read_u8(EC_CPU_PL4).unwrap_or(0);
        if pl4 == 0 {
            0x82
        } else {
            pl4
        }
    }

    /// 写入 CPU PL4
    pub fn cpu_write_pl4(&self, w: u8) -> io::Result<()> {
        self.write_u8(EC_CPU_PL4, w)
    }

    /// 读取 GPU PL1
    pub fn gpu_read_pl1(&self) -> io::Result<u8> {
        self.read_u8(EC_GPU_PL1)
    }

    /// 写入 GPU PL1
    pub fn gpu_write_pl1(&self, w: u8) -> io::Result<()> {
        self.write_u8(EC_GPU_PL1, w)
    }

    /// 读取 GPU PL2
    pub fn gpu_read_pl2(&self) -> io::Result<u8> {
        self.read_u8(EC_GPU_PL2)
    }

    /// 写入 GPU PL2
    pub fn gpu_write_pl2(&self, w: u8) -> io::Result<()> {
        self.write_u8(EC_GPU_PL2, w)
    }

    /// 读取 Battery Charging limit
    // pub fn battery_read_charglimit(&self)  -> io::Result<u8> {
    //     self.read_u8(EC_CUSTOM_CHARGELIMIT)
    // }

    /// 写入 Battery Charging limit
    // pub fn battery_write_charglimit(&self, w: u8)  -> io::Result<()> {
    //     self.write_u8(EC_CUSTOM_CHARGELIMIT, w)
    // }

    /// 读取 PSYS PL1
    pub fn psys_read_pl1(&self)  -> io::Result<u8> {
        self.read_u8(EC_PSYS_PL1_2)
    }

    /// 写入 PSYS PL1
    pub fn psys_write_pl1(&self, w: u8)  -> io::Result<()> {
        self.write_u8(EC_PSYS_PL1_2, w)
    }

    /// 读取 APC WATT
    pub fn system_read_power(&self) -> io::Result<u8> {
        self.read_u8(EC_APC_WATT)
    }

    /// 键盘
    pub fn decode_brightness(&self, raw: u8) -> u8 {
        match raw {
            0x08..=0x0C => raw - 8,
            0x01..=0x04 => raw,
            _ => 0,
        }
    }
    pub fn encode_brightness(&self, raw: u8) -> u8 {
        match raw {
            0x08..=0x0C => raw,
            0x01..=0x04 => raw + 8,
            _ => 8,
        }
    }

    pub fn keyboard_read(&self, ac: bool) -> KeyboardBacklight {
        let (enable_addr, red_addr, green_addr, blue_addr) = if ac {
            (
                EC_KEY_AC_SINGLEBL,
                EC_KEY_AC_RED,
                EC_KEY_AC_GREEN,
                EC_KEY_AC_BLUE,
            )
        } else {
            (
                EC_KEY_DC_SINGLEBL,
                EC_KEY_DC_RED,
                EC_KEY_DC_GREEN,
                EC_KEY_DC_BLUE,
            )
        };

        let enable = self.read_u8(enable_addr).unwrap_or(0);
        let r = self.read_u8(red_addr).unwrap_or(0);
        let g = self.read_u8(green_addr).unwrap_or(0);
        let b = self.read_u8(blue_addr).unwrap_or(0);
        
        KeyboardBacklight {
            enabled: (enable & 0x10) == 0,
            brightness: self.decode_brightness(enable & 0x0F),
            rainbow: (enable & 0x20) != 0,
            red: r,
            green: g,
            blue: b
        }
    }

    pub fn keyboard_write_enable(&self, enable: bool, ac: bool) {
        let addr = if ac { EC_KEY_AC_SINGLEBL } else { EC_KEY_DC_SINGLEBL };
        let ret = self.read_u8(addr).unwrap_or(0);
        let _ = self.write_u8(
            addr,
            self.encode_brightness(ret & 0x0F) | if enable { 0x00 } else { 0x10 } | (ret & 0x20)
        ); //self.encode_brightness(if enable { ret & !0x10 } else { ret | 0x10 }));
    }

    pub fn keyboard_write_brightness(&self, brightness: u8, ac: bool) {
        let addr = if ac { EC_KEY_AC_SINGLEBL } else { EC_KEY_DC_SINGLEBL };
        let ret = self.read_u8(addr).unwrap_or(0);
        let _ = self.write_u8(addr, (ret & 0xF0) | (0x08 + brightness));

    }

    pub fn keyboard_write_rainbow(&self, rainbow: bool, ac: bool) {
        let addr = if ac { EC_KEY_AC_SINGLEBL } else { EC_KEY_DC_SINGLEBL };
        let ret = self.read_u8(addr).unwrap_or(0);
        let _ = self.write_u8(
            addr,
            self.encode_brightness(ret & 0x0F) | (ret & 0x10) | if rainbow { 0x20 } else { 0x00 }
        ); //if rainbow { ret | 0x20 } else { ret & !0x20 });
    }

    pub fn keyboard_write_red(&self, red: u8, ac: bool) {
        let addr = if ac { EC_KEY_AC_RED } else { EC_KEY_DC_RED };
        let _ = self.write_u8(addr, red);
    }

    pub fn keyboard_write_green(&self, green: u8, ac: bool) {
        let addr = if ac { EC_KEY_AC_GREEN } else { EC_KEY_DC_GREEN };
        let _ = self.write_u8(addr, green);
    }

    pub fn keyboard_write_blue(&self, blue: u8, ac: bool) {
        let addr = if ac { EC_KEY_AC_BLUE } else { EC_KEY_DC_BLUE };
        let _ = self.write_u8(addr, blue);
    }

    /// 风扇手动模式
    pub fn fan_write_manual(&self, enable: bool) {
        let ret = self.fan_read_mode().unwrap_or(0);
        let _ = if enable {
            if ret&0x40!=0 {
                let w = ret & 0xBF;
                println!("fan_write_manual: ret: {}, write: {}", ret, w);
                self.write_u8(EC_ADDR_MANUAL_FAN_CTRL, w) // !(1 << 6)
            } else {
                Ok(())
            }
        } else {
            Ok(())
        };
    }

    pub fn fan_read_manual(&self) -> bool {
        (self.fan_read_mode().unwrap_or(0) & 0x40) == 0
    }

    /// 双风扇独立控制
    pub fn fan_write_custom_table_1(&self, enable: bool) {
        let _ = if enable {
            self.write_u8(EC__FAN_CUSTOM_TABLE_1, 0x80)
        } else {
            self.write_u8(EC__FAN_CUSTOM_TABLE_1, 0)
        };
    }

    /// 风扇表
    pub fn fan_write_custom_table_2(&self, enable: bool) {
        let _ = if enable {
            self.write_u8(EC__FAN_CUSTOM_TABLE_2, 0x04)
        } else {
            self.write_u8(EC__FAN_CUSTOM_TABLE_2, 0)
        };
    }

    /// 分离风扇
    pub fn fan_write_split(&self, enable: bool) {
        if enable {
            let _ = self.write_u8(_EC_ADDR_AP_OEM_BYTE, 1);
        } else {
            let _ = self.write_u8(_EC_ADDR_AP_OEM_BYTE, 0);
        }
    }
    /// 是否启用双风扇独立控制
    pub fn fan_read_custom_table_1(&self) -> bool {
        (self.read_u8(EC__FAN_CUSTOM_TABLE_1).unwrap_or(0) & 0x80) != 0
    }

    /// 是否启用风扇表
    pub fn fan_read_custom_table_2(&self) -> bool {
        (self.read_u8(EC__FAN_CUSTOM_TABLE_2).unwrap_or(0) & 0x04) != 0
    }

    /// 占空比
    pub fn fan_read_duty(&self) -> bool {
        let cpu = self.read_u8(EC_FAN_CPU_DUTY_BASE).unwrap_or(0);
        let gpu = self.read_u8(EC_FAN_GPU_DUTY_BASE).unwrap_or(0);
        cpu == 0 && gpu == 0
    }

    pub fn fan_write_duty(&self, enable:bool) {
        let w: u8;
        if enable {
            w = 0;
        } else {
            w = 0xFF;
        }
        let _ = self.write_u8(EC_FAN_CPU_DUTY_BASE, w);
        let _ = self.write_u8(EC_FAN_GPU_DUTY_BASE, w);
    }

    /// 新风扇初始化
    pub fn fan_write_init(&self) {
        self.fan_write_manual(true);
        self.fan_write_split(true);
        self.fan_write_custom_table_1(true);
        self.fan_write_custom_table_2(true);
        self.fan_write_duty(true);
    }

    /// 设置风扇
    pub fn fan_write_set(&self, fandata: FanData) {
        let (left, right) = (fandata.left_fan, fandata.right_fan);
        let _ = self.write_u8(EC_FAN_CPU_UP_BASE, 20);
        let _ = self.write_u8(EC_FAN_CPU_DOWN_BASE, 17);
        for i in 1..16 {
            let _ = self.write_u8(EC_FAN_CPU_UP_BASE + i as u16, left[i-1].temperature);
            thread::sleep(Duration::from_millis(10));
            let _ = self.write_u8(EC_FAN_CPU_DOWN_BASE + i as u16, left[i-1].temperature - 3);
            thread::sleep(Duration::from_millis(10));
            let _ = self.write_u8(EC_FAN_CPU_DUTY_BASE + i as u16, left[i-1].speed * 2);
            thread::sleep(Duration::from_millis(10));
        }
        let _ = self.write_u8(EC_FAN_GPU_UP_BASE, 20);
        let _ = self.write_u8(EC_FAN_GPU_DOWN_BASE, 17);
        for i in 1..16 {
            let _ = self.write_u8(EC_FAN_GPU_UP_BASE + i as u16, right[i-1].temperature);
            thread::sleep(Duration::from_millis(10));
            let _ = self.write_u8(EC_FAN_GPU_DOWN_BASE + i as u16, right[i-1].temperature - 3);
            thread::sleep(Duration::from_millis(10));
            let _ = self.write_u8(EC_FAN_GPU_DUTY_BASE + i as u16, right[i-1].speed * 2);
            thread::sleep(Duration::from_millis(10));
        }
    }

    /// 退出
    pub fn fan_write_close(&self) {
        let _ = self.write_u8(EC_FAN_CPU_UP_BASE, 0);
        let _ = self.write_u8(EC_FAN_CPU_DOWN_BASE, 0);
        let _ = self.write_u8(EC_FAN_GPU_UP_BASE, 0);
        let _ = self.write_u8(EC_FAN_GPU_DOWN_BASE, 0);
        self.fan_write_custom_table_1(false);
        self.fan_write_custom_table_2(false);
        self.fan_write_duty(false);
        self.fan_write_split(false);
    }


}

impl Drop for UniwillAcpiEc {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}
