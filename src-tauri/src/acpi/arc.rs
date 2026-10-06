#![allow(non_snake_case)]

use libloading::{Library, Symbol};
use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::ptr;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ============================================================
// Level Zero basic definitions
// ============================================================

type ZeResult = i32;

// Level Zero opaque handles.
type ZeDriverHandle = *mut c_void;
type ZeDeviceHandle = *mut c_void;
type ZesPowerHandle = *mut c_void;

const ZE_RESULT_SUCCESS: ZeResult = 0;

const ZE_STRUCTURE_TYPE_DEVICE_PROPERTIES: i32 = 0x0000_0003;

#[repr(C)]
#[derive(Clone, Copy)]
struct ZeDeviceUuid {
    id: [u8; 16],
}

#[repr(C)]
struct ZeDeviceProperties {
    stype: i32,
    pNext: *mut c_void,

    // ze_device_type_t
    device_type: u32,

    // uint32_t vendorId
    vendorId: u32,

    // uint32_t deviceId
    deviceId: u32,

    // ze_device_property_flags_t
    flags: u32,

    // uint32_t subdeviceId
    subdeviceId: u32,

    // uint32_t coreClockRate
    coreClockRate: u32,

    // uint64_t maxMemAllocSize
    maxMemAllocSize: u64,

    // uint32_t maxHardwareContexts
    maxHardwareContexts: u32,

    // uint32_t maxCommandQueuePriority
    maxCommandQueuePriority: u32,

    // uint32_t numThreadsPerEU
    numThreadsPerEU: u32,

    // uint32_t physicalEUSimdWidth
    physicalEUSimdWidth: u32,

    // uint32_t numEUsPerSubslice
    numEUsPerSubslice: u32,

    // uint32_t numSubslicesPerSlice
    numSubslicesPerSlice: u32,

    // uint32_t numSlices
    numSlices: u32,

    // uint64_t timerResolution
    timerResolution: u64,

    // uint32_t timestampValidBits
    timestampValidBits: u32,

    // uint32_t kernelTimestampValidBits
    kernelTimestampValidBits: u32,

    // ze_device_uuid_t
    uuid: ZeDeviceUuid,

    // char name[ZE_MAX_DEVICE_NAME]
    name: [u8; 256],
}