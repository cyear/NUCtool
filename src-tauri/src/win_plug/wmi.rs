use anyhow::{anyhow, Context};
use serde::Deserialize;
use windows::core::{w, BSTR, VARIANT};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoInitializeSecurity, CLSCTX_INPROC_SERVER,
    COINIT_MULTITHREADED, EOAC_NONE, RPC_C_AUTHN_LEVEL_DEFAULT, RPC_C_IMP_LEVEL_IMPERSONATE,
};
use windows::Win32::System::Wmi::{
    IWbemClassObject, IWbemLocator, IWbemServices, WbemLocator, WBEM_FLAG_FORWARD_ONLY,
    WBEM_FLAG_RETURN_ERROR_OBJECT, WBEM_FLAG_RETURN_WBEM_COMPLETE, WBEM_INFINITE,
};
use wmi::{COMLibrary, WMIConnection};
use colored::Colorize;

pub fn wmi_security() {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)
            .ok()
            .context("Initializing COM")
            .expect("Initializing Error");
        match CoInitializeSecurity(
            None,
            -1,
            None,
            None,
            RPC_C_AUTHN_LEVEL_DEFAULT,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            None,
            EOAC_NONE,
            None,
        ) {
            Ok(_) => {}
            Err(e) => {
                println!("Error: {e}");
            }
        }
        // .context("Initializing COM security")
        // .expect("Initializing COM security Error");
    }
    // println!("{}", "初始化 COM security".green());
}

pub fn wmi_init() -> (IWbemClassObject, IWbemServices, BSTR, BSTR) {
    // Connect to the required namespace on the local DCOM server.
    let loc: IWbemLocator = unsafe {
        CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)
            .context("Failed to get WbemLocator")
            .unwrap()
    };
    let svc = unsafe {
        loc.ConnectServer(&BSTR::from(r"ROOT\WMI"), None, None, None, 0, None, None)
            .context("Connecting to server")
            .expect("Connecting Error")
    };

    // Allocate null-terminated 16-bit character strings for the object class name and method name.
    let cls_name = BSTR::from("AcpiTest_MULong");
    let method_name = BSTR::from("GetSetULong");

    // List instances of the requested object by name, and get the path of the first.
    let object_enum = unsafe {
        svc.CreateInstanceEnum(
            &cls_name,
            WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_ERROR_OBJECT,
            None,
        )
        .context("Get cls_name")
        .expect("Get cls_name Error")
    };
    let mut objects = [None; 1];
    let mut count: u32 = 0;
    unsafe {
        object_enum
            .Next(WBEM_INFINITE, &mut objects, &mut count)
            .ok()
            .context("Retrieving first")
            .unwrap();
    }
    let mut obj_path = VARIANT::new();
    unsafe {
        objects[0]
            .as_ref()
            .ok_or_else(|| anyhow!("Missing object"))
            .expect("Error")
            .Get(w!("__RELPATH"), 0, &mut obj_path, None, None)
            .context("Retrieving object path")
            .expect("Error");
    }
    let obj_path = BSTR::try_from(&obj_path)
        .context("Converting object path to string")
        .expect("Error");
    drop(objects);
    drop(object_enum);
    // println!("Instance: {obj_path}");

    // Get an input parameter object from the object class.
    let mut cls: Option<IWbemClassObject> = None;
    unsafe {
        svc.GetObject(
            &cls_name,
            WBEM_FLAG_RETURN_WBEM_COMPLETE,
            None,
            Some(&mut cls),
            None,
        )
        .context("Getting class MSFT_Disk")
        .unwrap();
    }
    let cls = cls.ok_or_else(|| anyhow!("Missing class")).expect("Error");
    let mut in_cls: Option<IWbemClassObject> = None;
    let mut out_cls: Option<IWbemClassObject> = None;
    unsafe {
        cls.GetMethod(&method_name, 0, &mut in_cls, &mut out_cls)
            .context("Getting method")
            .expect("Get method Error");
    }
    // println!("{}", "初始化 WMI".green());
    (in_cls.unwrap(), svc, obj_path, method_name)
}

pub fn wmi_set(in_cls: &IWbemClassObject, svc: &IWbemServices, obj_path: &BSTR, method_name: &BSTR, size: &str) -> i64 {
    let in_params = unsafe {
        in_cls
            .SpawnInstance(0)
            .context("Creating input params")
            .unwrap()
    };
    unsafe {
        // in_params.Put(&BSTR::from("AssignDriveLetter"), 0, &VARIANT::from(true), 0).context("Setting AssignDriveLetter")?;
        in_params
            .Put(
                &BSTR::from("Data"),
                0,
                &VARIANT::from(
                    u64::from_str_radix(&size[2..], 16)
                        .unwrap()
                        .to_string()
                        .as_str(),
                ),
                0,
            )
            .context("Setting Size")
            .unwrap();
    }
    // Call the method and check the return value.
    // println!("Calling method with {}", unsafe { in_params.GetObjectText(0).unwrap() });
    let mut out_params: Option<IWbemClassObject> = None;
    unsafe {
        svc.ExecMethod(
            obj_path,
            method_name,
            WBEM_FLAG_RETURN_WBEM_COMPLETE,
            None,
            &in_params,
            Some(&mut out_params),
            None,
        )
        .context("Failed to call CreatePartition")
        .unwrap();
    }
    let out_params = out_params
        .ok_or_else(|| anyhow!("Missing output parameters"))
        .unwrap();
    let mut return_value = VARIANT::new();
    unsafe {
        out_params
            .Get(w!("Return"), 0, &mut return_value, None, None)
            .unwrap();
    }
    // println!("Return value = {return_value}.");
    return_value.to_string().parse::<i64>().unwrap()
}

pub fn get_model() -> i64 {
    #[derive(Deserialize, Debug)]
    struct LaptopInfo {
        model: String,
        manufacturer: String,
    }
    let com_con = COMLibrary::new().unwrap();
    let wmi_con = WMIConnection::new(com_con.into()).unwrap();
    let results: Vec<LaptopInfo> = wmi_con
        .raw_query("SELECT Model, Manufacturer FROM Win32_ComputerSystem")
        .unwrap();
    for laptop in results {
        println!("Manufacturer: {}", laptop.manufacturer.blue());
        println!("Model: {}", laptop.model.blue());
        match laptop.model.as_str() {
            "LAPKC71F" => return 0,
            "LAPAC71H" => return 1,
            _ => return 0,
        }
    }
    0
}
