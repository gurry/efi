use ffi::base::{EFI_GUID, UINT8};

pub const EFI_DEVICE_PATH_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0x09576e91, 0x6d3f, 0x11d2, [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b]);

#[derive(Debug)]
#[repr(C)]
pub struct EFI_DEVICE_PATH_PROTOCOL {
    Type: UINT8,
    SubType: UINT8,
    Length: [UINT8; 2]
}