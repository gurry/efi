use ffi::{
    base::{
        EFI_STATUS, 
        EFI_GUID,
        EFI_HANDLE, 
        UINT32,
        UINT64,
        VOID,
    },
    boot_services::EFI_MEMORY_TYPE,
    device_path::EFI_DEVICE_PATH_PROTOCOL,
    EFI_SYSTEM_TABLE
};

pub const EFI_LOADED_IMAGE_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0x5B1B31A1, 0x9562, 0x11d2, [0x8E, 0x3F, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B]);

#[repr(C)]
pub struct EFI_LOADED_IMAGE_PROTOCOL {
    pub Revision: UINT32,
    pub ParentHandle: EFI_HANDLE,
    pub SystemTable: *const EFI_SYSTEM_TABLE,

    pub DeviceHandle: EFI_HANDLE,
    pub FilePath: *const EFI_DEVICE_PATH_PROTOCOL,
    pub Reserved: *const VOID,

    pub LoadOptionsSize: UINT32,
    pub LoadOptions: *const VOID,

    pub ImageBase: *const VOID,
    pub ImageSize: UINT64,
    pub ImageCodeType: EFI_MEMORY_TYPE,
    pub ImageDataType: EFI_MEMORY_TYPE,
    pub Unload: EFI_IMAGE_UNLOAD
}

pub type EFI_IMAGE_UNLOAD = extern "win64" fn(
    Handle: EFI_HANDLE
) -> EFI_STATUS;