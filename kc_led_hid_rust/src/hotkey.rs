use crate::{Result, com::Com};
use windows::{
    Win32::System::{
        Com::{
            CLSCTX_INPROC_SERVER, CoCreateInstance, CoSetProxyBlanket, EOAC_NONE,
            RPC_C_AUTHN_LEVEL_CALL, RPC_C_IMP_LEVEL_IMPERSONATE,
        },
        Rpc::{RPC_C_AUTHN_WINNT, RPC_C_AUTHZ_NONE},
        Variant::{VARIANT, VariantClear, VariantToUInt32},
        Wmi::{
            IEnumWbemClassObject, IWbemLocator, WBEM_FLAG_CONNECT_USE_MAX_WAIT,
            WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY, WBEM_GENERIC_FLAG_TYPE,
            WbemLocator,
        },
    },
    core::{BSTR, PCWSTR, w},
};

const BRIGHTNESS_OSD_CODE: u32 = 185;

/// Poll the factory EC's brightness hotkey events from the owning thread.
pub struct FnBrightnessEvents {
    events: IEnumWbemClassObject,
    _com: Com,
}

impl FnBrightnessEvents {
    pub fn open() -> Result<Self> {
        let com = Com::initialize()?;
        let events = unsafe {
            let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)?;
            let empty = BSTR::new();
            let services = locator.ConnectServer(
                &BSTR::from(r"ROOT\WMI"),
                &empty,
                &empty,
                &empty,
                WBEM_FLAG_CONNECT_USE_MAX_WAIT.0,
                &empty,
                None,
            )?;
            CoSetProxyBlanket(
                &services,
                RPC_C_AUTHN_WINNT,
                RPC_C_AUTHZ_NONE,
                PCWSTR::null(),
                RPC_C_AUTHN_LEVEL_CALL,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                None,
                EOAC_NONE,
            )?;
            services.ExecNotificationQuery(
                &BSTR::from("WQL"),
                &BSTR::from("SELECT * FROM AcpiTest_EventULong"),
                WBEM_GENERIC_FLAG_TYPE(WBEM_FLAG_FORWARD_ONLY.0 | WBEM_FLAG_RETURN_IMMEDIATELY.0),
                None,
            )?
        };
        Ok(Self { events, _com: com })
    }

    /// Drain up to 32 queued EC events without waiting. Returns the number of
    /// brightness hotkey presses in that batch.
    pub fn poll(&self) -> Result<usize> {
        let mut presses = 0;
        for _ in 0..32 {
            let mut event = [None];
            let mut count = 0;
            unsafe { self.events.Next(0, &mut event, &mut count) }.ok()?;
            let [Some(event)] = event else { break };
            let code = unsafe {
                let mut value = VARIANT::default();
                event.Get(w!("ULong"), 0, &mut value, None, None)?;
                let code = VariantToUInt32(&value);
                VariantClear(&mut value)?;
                code?
            };
            presses += usize::from(code == BRIGHTNESS_OSD_CODE);
        }
        Ok(presses)
    }
}
