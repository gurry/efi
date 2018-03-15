use ::{
    Guid,
    Result,
    Opaque,
    OpaqueImage,
    OpaqueDevice,
    SystemTable,
    to_res,
    protocols::DevicePathProtocol,
    boot_services::MemoryType,
};
use ffi::loaded_image::{EFI_LOADED_IMAGE_PROTOCOL, EFI_LOADED_IMAGE_PROTOCOL_GUID};
use core::{mem, slice};
use protocols::Protocol;

pub struct LoadedImageProtocol(EFI_LOADED_IMAGE_PROTOCOL);
impl_wrapper!(LoadedImageProtocol, EFI_LOADED_IMAGE_PROTOCOL);

impl Protocol for LoadedImageProtocol {
    type FfiType = EFI_LOADED_IMAGE_PROTOCOL;
    fn guid() -> Guid {
        EFI_LOADED_IMAGE_PROTOCOL_GUID
    }
}

impl LoadedImageProtocol {
    pub fn revision(&self) -> u32 {
        self.0.Revision
    }

    pub fn parent_handle(&self) -> &OpaqueImage {
        unsafe { mem::transmute(self.0.ParentHandle) }
    }

    pub fn system_table(&self) -> &SystemTable {
        unsafe { mem::transmute(self.0.SystemTable) }
    }

    pub fn device_handle(&self) -> &OpaqueDevice {
        unsafe { mem::transmute(self.0.DeviceHandle) }
    }

    pub fn file_path(&self) -> &DevicePathProtocol {
        unsafe { mem::transmute(self.0.FilePath) }
    }

    pub fn reserved(&self) -> &Opaque {
        unsafe { mem::transmute(self.0.Reserved) }
    }

    pub fn load_options(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.0.LoadOptions as *const u8, self.0.LoadOptionsSize as usize) }
    }

    pub fn image_base(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.0.ImageBase as *const u8, self.0.ImageSize as usize) }
    }

    pub fn image_code_type(&self) -> MemoryType {
        self.0.ImageCodeType
    }

    pub fn image_data_type(&self) -> MemoryType  {
        self.0.ImageDataType
    }

    pub fn unload(&self, handle: &OpaqueImage) -> Result<()> {
        let status = unsafe {
            (self.0.Unload)(mem::transmute(handle))
        };

        to_res((), status)
    }
}