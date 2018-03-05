use ffi::{boot_services::EFI_BOOT_SERVICES};
use ::{Result, Guid, Void, to_res};
use protocols::Protocol;
use core::{ptr, mem};

#[repr(C)]
pub struct BootServices(EFI_BOOT_SERVICES);

impl BootServices {
    // TODO: add the 'registration' argument also to this method
    pub fn locate_protocol<T: Protocol>(&self) -> Result<&T> {
        let guid_ptr = &T::guid() as *const Guid;
        let registration: *mut Void = ptr::null_mut();
        let mut protocol: *mut T::FfiType = ptr::null_mut();

        let status = unsafe {
            (self.0.LocateProtocol)(guid_ptr, registration, mem::transmute::<&mut *mut T::FfiType, *mut *mut Void>(&mut protocol))
        };

        to_res(unsafe { mem::transmute(protocol) }, status)
    }
}