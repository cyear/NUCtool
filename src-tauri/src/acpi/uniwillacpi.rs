use std::ffi::OsStr;
use std::io;
use std::os::windows::ffi::OsStrExt;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::IO::DeviceIoControl;

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

// Main fan RPM
const EC_MAIN_FAN_RPM_1: u16 = 0x0464;
const EC_MAIN_FAN_RPM_2: u16 = 0x0465;

// Secondary fan RPM
const EC_SECOND_FAN_RPM_1: u16 = 0x046C;
const EC_SECOND_FAN_RPM_2: u16 = 0x046D;

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
const EC_FAN_MODE: u16 = 0x0751;

// System Power (W) 疑似
const EC_APC_WATT: u16 = 0x044C;

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
        self.read_u8(EC_FAN_MODE)
    }

    /// 写入 Fan Mode
    pub fn fan_write_mode(&self, mode: FanModeByte) -> io::Result<()> {
        self.write_u8(EC_FAN_MODE, mode as u8)
    }

    /// 读取 CPU PL1
    pub fn cpu_read_pl1(&self) -> io::Result<u8> {
        self.read_u8(EC_CPU_PL1)
    }

    /// 写入 CPU PL1
    pub fn cpu_write_pl1(&self, w: u8) -> io::Result<()> {
        self.write_u8(EC_CPU_PL1, w)
    }

    /// 读取 CPU PL2
    pub fn cpu_read_pl2(&self) -> io::Result<u8> {
        self.read_u8(EC_CPU_PL2)
    }

    /// 写入 CPU PL2
    pub fn cpu_write_pl2(&self, w: u8) -> io::Result<()> {
        self.write_u8(EC_CPU_PL2, w)
    }

    /// 读取 CPU PL4
    pub fn cpu_read_pl4(&self) -> io::Result<u8> {
        self.read_u8(EC_CPU_PL4)
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

}

impl Drop for UniwillAcpiEc {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}
