use ffi::base::{EFI_GUID, UINT8, UINT16, NOT_DEFINED};

pub const EFI_DEVICE_PATH_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0x09576e91, 0x6d3f, 0x11d2, [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b]);

#[derive(Debug)]
#[repr(C)]
pub struct EFI_DEVICE_PATH_PROTOCOL {
    Type: UINT8,
    SubType: UINT8,
    Length: [UINT8; 2]
}


pub const EFI_DEVICE_PATH_UTILITIES_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0x379be4e, 0xd706, 0x437d, [0xb0, 0x37, 0xed, 0xb8, 0x2f, 0xb7, 0x72, 0xa4]);

#[repr(C)]
pub struct EFI_DEVICE_PATH_UTILITIES_PROTOCOL {
    pub GetDevicePathSize: EFI_DEVICE_PATH_UTILS_GET_DEVICE_PATH_SIZE, 
    pub DuplicateDevicePath: EFI_DEVICE_PATH_UTILS_DUP_DEVICE_PATH,
    pub AppendDevicePath: EFI_DEVICE_PATH_UTILS_APPEND_PATH,
    pub AppendDeviceNode: EFI_DEVICE_PATH_UTILS_APPEND_NODE,
    pub AppendDevicePathInstance: EFI_DEVICE_PATH_UTILS_APPEND_INSTANCE,
    pub GetNextDevicePathInstance: EFI_DEVICE_PATH_UTILS_GET_NEXT_INSTANCE,
    pub IsDevicePathMultiInstance: EFI_DEVICE_PATH_UTILS_IS_MULTI_INSTANCE,
    pub CreateDeviceNode: EFI_DEVICE_PATH_UTILS_CREATE_NODE,
}

pub type EFI_DEVICE_PATH_UTILS_GET_DEVICE_PATH_SIZE = *const NOT_DEFINED;
pub type EFI_DEVICE_PATH_UTILS_DUP_DEVICE_PATH = *const NOT_DEFINED;
pub type EFI_DEVICE_PATH_UTILS_APPEND_INSTANCE = *const NOT_DEFINED;
pub type EFI_DEVICE_PATH_UTILS_GET_NEXT_INSTANCE = *const NOT_DEFINED;
pub type EFI_DEVICE_PATH_UTILS_IS_MULTI_INSTANCE = *const NOT_DEFINED;

pub type EFI_DEVICE_PATH_UTILS_APPEND_PATH = extern "win64" fn(
    Src1: *const EFI_DEVICE_PATH_PROTOCOL,
    Src2: *const EFI_DEVICE_PATH_PROTOCOL
) -> *const EFI_DEVICE_PATH_PROTOCOL;

pub type EFI_DEVICE_PATH_UTILS_APPEND_NODE = extern "win64" fn(
    DevicePath: *const EFI_DEVICE_PATH_PROTOCOL,
    DevicePath: *const EFI_DEVICE_PATH_PROTOCOL
) -> *const EFI_DEVICE_PATH_PROTOCOL;

pub type EFI_DEVICE_PATH_UTILS_CREATE_NODE = extern "win64" fn(
    NodeType: UINT8,
    NodeSubType: UINT8,
    NodeLength: UINT16
) -> *const EFI_DEVICE_PATH_PROTOCOL;

// Device path types
pub const HARDWARE_DEVICE_PATH: UINT8 = 0x01; 
pub const ACPI_DEVICE_PATH: UINT8 = 0x02;
pub const MESSAGING_DEVICE_PATH: UINT8 = 0x03;
pub const MEDIA_DEVICE_PATH: UINT8 = 0x04;
pub const BBS_DEVICE_PATH: UINT8 = 0x05; // BIOS Boot Specification device path
pub const END_DEVICE_PATH_TYPE: UINT8 = 0x7f;

// Device path sub-types
pub const MEDIA_FILEPATH_DP: UINT8 = 0x04; // File device path sub type
pub const END_ENTIRE_DEVICE_PATH_SUBTYPE: UINT8 = 0xFF;
pub const END_INSTANCE_DEVICE_PATH_SUBTYPE: UINT8 = 0x01;