use ::{Guid, Result};
use ffi::{
    load_file::{EFI_LOAD_FILE_PROTOCOL, EFI_LOAD_FILE_PROTOCOL_GUID}, 
    device_path::EFI_DEVICE_PATH_PROTOCOL,
    EFI_STATUS,
    EFI_SUCCESS,
    UINTN,
    BOOLEAN,
    VOID
};

use core::{mem, slice};

use protocols::Protocol;
use super::device_path::DevicePathProtocol;
use ::utils::Wrapper;

pub struct LoadFileProtocol<'a> {
    inner: EFI_LOAD_FILE_PROTOCOL,
    load_file: &'a mut FnMut(&DevicePathProtocol, &mut [u8]) -> Result<usize> // TODO: going with dyn dispatch for now. Will come back to it later.
}

impl<'a> LoadFileProtocol<'a> {
    pub fn new(load_file: &'a mut FnMut(&DevicePathProtocol, &mut [u8]) -> Result<usize>) -> Self {
        Self { inner: EFI_LOAD_FILE_PROTOCOL { LoadFile: load_file_callback }, load_file }
    }
}

impl<'a> Protocol for LoadFileProtocol<'a> {
    type FfiType = EFI_LOAD_FILE_PROTOCOL;
    fn guid() -> Guid {
        EFI_LOAD_FILE_PROTOCOL_GUID
    }
}

impl<'a> Wrapper for LoadFileProtocol<'a> {
     type Inner = EFI_LOAD_FILE_PROTOCOL;
    fn inner_ptr(&self) -> *const Self::Inner {
        &self.inner
    }
}

// TODO: do we need 'pub' on this to be callable from UEFI?
#[no_mangle]
pub extern "win64" fn load_file_callback(
    this: *const EFI_LOAD_FILE_PROTOCOL, 
    file_path: *const EFI_DEVICE_PATH_PROTOCOL,
    boot_policy: BOOLEAN,
    buffer_size: *mut UINTN, 
    buffer_ptr: *mut VOID
) -> EFI_STATUS {

    // TODO: Properly handle boot_policy also
    let load_file_protocol: &mut LoadFileProtocol = unsafe { mem::transmute(this) };

    let file_path: &DevicePathProtocol = unsafe { mem::transmute(file_path) };

    match (load_file_protocol.load_file)(file_path, unsafe { slice::from_raw_parts_mut(buffer_ptr as *mut u8, *buffer_size) }) {
        Ok(bytes_written) => { 
            unsafe { *buffer_size  = bytes_written };
            EFI_SUCCESS
        },
        Err(e) => e.into()
    }
}

