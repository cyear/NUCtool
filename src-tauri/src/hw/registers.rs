//! EC/ACPI WMI 寄存器定义与命令编码
//!
//! WMI 方法 `AcpiTest_MULong.GetSetULong` 接收一个 u64 命令字:
//!
//! - 读: `0x0000_0100_0000_0000 | 寄存器`
//! - 写: `(数值 << 16) | 寄存器`
//!
//! 与旧版字符串常量(如 `"0x000001000000046C"`)完全等价,
//! 只是改为类型安全的数值编码。

/// 读命令标志位
pub const READ_FLAG: u64 = 0x0000_0100_0000_0000;

// ---- 寄存器 ----
/// GPU TDP PL1
pub const REG_TDP_GPU1: u64 = 0x072D;
/// GPU TDP PL2
pub const REG_TDP_GPU2: u64 = 0x072E;
/// CPU TDP PL1
pub const REG_TDP_CPU1: u64 = 0x0783;
/// CPU TDP PL2
pub const REG_TDP_CPU2: u64 = 0x0784;
/// 温度墙偏移
pub const REG_TDP_TCC: u64 = 0x0786;
/// 左风扇转速 高字节
pub const REG_FAN_L1: u64 = 0x046C;
/// 左风扇转速 低字节
pub const REG_FAN_L2: u64 = 0x046D;
/// 右风扇转速 高字节
pub const REG_FAN_R1: u64 = 0x0464;
/// 右风扇转速 低字节
pub const REG_FAN_R2: u64 = 0x0465;
/// CPU 温度
pub const REG_TEMP_CPU: u64 = 0x043E;
/// GPU 温度
pub const REG_TEMP_GPU: u64 = 0x044F;
/// 风扇模式(读: 0x6C10/0x6C00 为自动; 写: 见下方值)
pub const REG_FAN_MODE: u64 = 0x0751;
/// 左风扇转速写入
pub const REG_FAN_SET_L: u64 = 0x1809;
/// 右风扇转速写入
pub const REG_FAN_SET_R: u64 = 0x1804;
/// AC 供电键盘 LED 彩色模式
pub const REG_LED_AC: u64 = 0x07EA;
/// DC 供电键盘 LED 彩色模式(暂未使用)
#[allow(dead_code)]
pub const REG_LED_DC: u64 = 0x07EB;

// ---- 写入值 ----
/// LAPAC71H 接管风扇(Turbo 模式)
pub const FAN_TAKEOVER_AC71H: u64 = 0x40;
/// LAPKC71F 接管风扇(Turbo 模式)
pub const FAN_TAKEOVER_KC71F: u64 = 0x50;
/// 风扇恢复自动
pub const FAN_AUTO: u64 = 0xA0;
/// LED 彩色模式 开
pub const LED_ON: u64 = 0x2A;
/// LED 彩色模式 关
pub const LED_OFF: u64 = 0x0A;

/// 构造读命令
pub const fn read(reg: u64) -> u64 {
    READ_FLAG | reg
}

/// 构造写命令
pub const fn write(reg: u64, value: u64) -> u64 {
    (value << 16) | reg
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 校验编码与旧版字符串常量完全一致
    #[test]
    fn encoding_matches_legacy_strings() {
        assert_eq!(read(REG_TDP_GPU1), 0x000001000000072D);
        assert_eq!(read(REG_TDP_GPU2), 0x000001000000072E);
        assert_eq!(read(REG_TDP_CPU1), 0x0000010000000783);
        assert_eq!(read(REG_TDP_CPU2), 0x0000010000000784);
        assert_eq!(read(REG_TDP_TCC), 0x0000010000000786);
        assert_eq!(read(REG_FAN_L1), 0x000001000000046C);
        assert_eq!(read(REG_FAN_L2), 0x000001000000046D);
        assert_eq!(read(REG_FAN_R1), 0x0000010000000464);
        assert_eq!(read(REG_FAN_R2), 0x0000010000000465);
        assert_eq!(read(REG_TEMP_CPU), 0x000001000000043E);
        assert_eq!(read(REG_TEMP_GPU), 0x000001000000044F);
        assert_eq!(read(REG_FAN_MODE), 0x0000010000000751);
        assert_eq!(write(REG_FAN_MODE, FAN_TAKEOVER_AC71H), 0x0000000000400751);
        assert_eq!(write(REG_FAN_MODE, FAN_TAKEOVER_KC71F), 0x0000000000500751);
        assert_eq!(write(REG_FAN_MODE, FAN_AUTO), 0x0000000000A00751);
        assert_eq!(read(REG_LED_AC), 0x00000100000007EA);
        assert_eq!(write(REG_LED_AC, LED_ON), 0x00000000002A07EA);
        assert_eq!(write(REG_LED_AC, LED_OFF), 0x00000000000A07EA);
        // 旧版: format!("0x000000000{:02x}1809", speed)
        assert_eq!(write(REG_FAN_SET_L, 0xC8), 0x0000000000C81809);
        assert_eq!(write(REG_FAN_SET_R, 0x64), 0x0000000000641804);
        // 旧版: format!("0x000000000{:02x}0783", tdp)
        assert_eq!(write(REG_TDP_CPU1, 0x2D), 0x00000000002D0783);
    }
}
