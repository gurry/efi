use ::Guid;
use ffi::device_path::{EFI_DEVICE_PATH_PROTOCOL, EFI_DEVICE_PATH_PROTOCOL_GUID};
use protocols::Protocol;

pub struct DevicePathProtocol(EFI_DEVICE_PATH_PROTOCOL);

impl Protocol for DevicePathProtocol {
    type FfiType = EFI_DEVICE_PATH_PROTOCOL;
    fn guid() -> Guid {
        EFI_DEVICE_PATH_PROTOCOL_GUID
    }
}