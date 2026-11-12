use std::error::Error;

use wmi::{Variant, WMIConnection};

const WMI_NAMESPACE: &str = r"ROOT\WMI";
const WMI_CLASS: &str = "AcpiTest_MULong";
const WMI_METHOD: &str = "GetSetULong";

// ============================================================
// Uniwill WMI interface
// ============================================================

pub struct UniwillWmiEc {
    wmi: WMIConnection,
    instance_path: String,
    // active: bool,
}

impl UniwillWmiEc {
    /// 创建 WMI EC 连接。
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let wmi = WMIConnection::with_namespace_path(WMI_NAMESPACE)?;

        let mut objects = wmi.exec_query("SELECT * FROM AcpiTest_MULong")?;

        let object = objects
            .next()
            .ok_or("AcpiTest_MULong instance not found")??;

        let instance_path = object.path()?;

        let _active: bool = object.get_property("Active")?.try_into()?;

        let _instance_name: String = object.get_property("InstanceName")?.try_into()?;

        // println!(
        //     "WMI Instance: {}",
        //     instance_path
        // );

        // println!(
        //     "InstanceName: {}",
        //     instance_name
        // );

        // println!(
        //     "Active: {}",
        //     active
        // );

        Ok(Self {
            wmi,
            instance_path,
            // active,
        })
    }

    // pub fn instance_path(&self) -> &str {
    //     &self.instance_path
    // }

    // pub fn active(&self) -> bool {
    //     self.active
    // }

    pub fn get_set(&self, data: u64) -> Result<u32, Box<dyn Error>> {
        // 获取 class definition
        let class = self.wmi.get_object(WMI_CLASS)?;

        // 获取 GetSetULong 输入参数定义
        let input_class = class
            .get_method(WMI_METHOD)?
            .ok_or("GetSetULong input signature not found")?;

        // 创建输入参数实例
        let input = input_class.spawn_instance()?;

        // uint64 Data
        input.put_property("Data", Variant::UI8(data))?;

        // 调用具体 instance 的方法
        let output = self
            .wmi
            .exec_method(&self.instance_path, WMI_METHOD, Some(&input))?
            .ok_or("GetSetULong returned no output")?;

        // uint32 Return
        let return_value = output.get_property("Return")?;

        let return_value: u32 = return_value.try_into()?;

        Ok(return_value)
    }

    // /// 读取 EC byte。
    // pub fn read_u8(
    //     &self,
    //     address: u16,
    // ) -> Result<u8, Box<dyn Error>> {
    //     let data =
    //         ((0x0100u64) << 32)
    //         | address as u64;

    //     let result =
    //         self.get_set(data)?;

    //     Ok(
    //         (result & 0xFF)
    //             as u8
    //     )
    // }

    // /// 写入 EC byte。
    // pub fn write_u8(
    //     &self,
    //     address: u16,
    //     value: u8,
    // ) -> Result<(), Box<dyn Error>> {
    //     let data =
    //         ((value as u64) << 16)
    //         | address as u64;

    //     self.get_set(data)?;

    //     Ok(())
    // }

    // /// 连续读取。
    // pub fn read_many(
    //     &self,
    //     addresses: &[u16],
    // ) -> Result<Vec<(u16, u8)>, Box<dyn Error>> {
    //     let mut result =
    //         Vec::with_capacity(
    //             addresses.len()
    //         );

    //     for &address in addresses {
    //         result.push((
    //             address,
    //             self.read_u8(address)?,
    //         ));
    //     }

    //     Ok(result)
    // }

    // /// 连续写入。
    // pub fn write_many(
    //     &self,
    //     values: &[(u16, u8)],
    // ) -> Result<(), Box<dyn Error>> {
    //     for &(address, value) in values {
    //         self.write_u8(
    //             address,
    //             value
    //         )?;
    //     }

    //     Ok(())
    // }
}

pub fn get_model() -> Result<String, Box<dyn Error>> {
    let wmi = WMIConnection::with_namespace_path(
        r"ROOT\CIMV2"
    )?;

    let mut objects = wmi.exec_query(
        "SELECT Name FROM Win32_ComputerSystemProduct"
    )?;

    let object = objects
        .next()
        .ok_or("ComputerSystemProduct not found")??;

    let model: String =
        object.get_property("Name")?.try_into()?;

    Ok(model)
}

pub fn get_gpu_driver() -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let wmi = WMIConnection::with_namespace_path(
        r"ROOT\CIMV2"
    )?;

    let mut objects = wmi.exec_query(
        "SELECT DeviceName, DriverVersion \
         FROM Win32_PnPSignedDriver \
         WHERE DeviceClass = 'DISPLAY'"
    )?;

    let mut result = Vec::new();

    while let Some(object) = objects.next() {
        let object = object?;

        let name: String =
            object.get_property("DeviceName")?.try_into()?;

        let version: String =
            object.get_property("DriverVersion")?.try_into()?;

        result.push((name, version));
    }

    Ok(result)
}

pub fn get_gsc_driver() -> Result<Option<String>, Box<dyn Error>> {
    let wmi = WMIConnection::with_namespace_path(
        r"ROOT\CIMV2"
    )?;

    let mut objects = wmi.exec_query(
        "SELECT DeviceName, DriverVersion \
         FROM Win32_PnPSignedDriver \
         WHERE DeviceName = 'Intel(R) Graphics System Controller Firmware Interface'"
    )?;

    if let Some(object) = objects.next() {
        let object = object?;

        let version: String =
            object.get_property("DriverVersion")?.try_into()?;

        return Ok(Some(version));
    }

    Ok(None)
}