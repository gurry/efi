use ::Guid;
use ffi::device_path::{
    EFI_DEVICE_PATH_PROTOCOL,
    EFI_DEVICE_PATH_PROTOCOL_GUID,
    EFI_DEVICE_PATH_UTILITIES_PROTOCOL,
    EFI_DEVICE_PATH_UTILITIES_PROTOCOL_GUID
};
use core::mem;

use protocols::Protocol;

pub struct DevicePathProtocol(EFI_DEVICE_PATH_PROTOCOL);
impl_wrapper!(DevicePathProtocol, EFI_DEVICE_PATH_PROTOCOL);

impl Protocol for DevicePathProtocol {
    type FfiType = EFI_DEVICE_PATH_PROTOCOL;
    fn guid() -> Guid {
        EFI_DEVICE_PATH_PROTOCOL_GUID
    }
}


pub struct DevicePathUtilitiesProtocol(EFI_DEVICE_PATH_UTILITIES_PROTOCOL);
impl_wrapper!(DevicePathUtilitiesProtocol, EFI_DEVICE_PATH_UTILITIES_PROTOCOL);

impl DevicePathUtilitiesProtocol {
    pub fn append_device_path(&self, src1: &DevicePathProtocol, src2: &DevicePathProtocol) -> &'static DevicePathProtocol { // TODO: should the lifetime of returned value be static?
        unsafe { 
            let path = (self.0.AppendDevicePath)(mem::transmute(src1), mem::transmute(src2));
            mem::transmute(path)
        }
    }
}

impl Protocol for DevicePathUtilitiesProtocol {
    type FfiType = EFI_DEVICE_PATH_UTILITIES_PROTOCOL;
    fn guid() -> Guid {
        EFI_DEVICE_PATH_UTILITIES_PROTOCOL_GUID
    }
}