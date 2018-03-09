use ffi::base::{
    EFI_GUID, 
    EFI_STATUS, 
    UINTN,
    BOOLEAN,
    VOID
};


use super::device_path::EFI_DEVICE_PATH_PROTOCOL;

pub const EFI_LOAD_FILE_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0x56EC3091, 0x954C, 0x11d2, [0x8E, 0x3F, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B]);

#[repr(C)]
pub struct EFI_LOAD_FILE_PROTOCOL {
    pub LoadFile: EFI_LOAD_FILE
}

pub type EFI_LOAD_FILE = extern "win64" fn(
    This: *const EFI_LOAD_FILE_PROTOCOL, 
    FilePath: *const EFI_DEVICE_PATH_PROTOCOL,
    BootPolicy: BOOLEAN,
    BufferSize: *mut UINTN, 
    BufferPtr: *mut VOID
) -> EFI_STATUS;