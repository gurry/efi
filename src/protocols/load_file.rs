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

pub struct LoadFileProtocol<'a, C: 'a + FnMut(&DevicePathProtocol, &mut [u8]) -> Result<usize>> {
    inner: EFI_LOAD_FILE_PROTOCOL,
    load_file: &'a mut C
}

impl<'a, C: 'a + FnMut(&DevicePathProtocol, &mut [u8]) -> Result<usize>> LoadFileProtocol<'a, C> {
    pub fn new(load_file: &'a mut C) -> Self {
        Self { inner: EFI_LOAD_FILE_PROTOCOL { LoadFile: load_file_callback::<'a, C> }, load_file }
    }
}

impl<'a, C: 'a + FnMut(&DevicePathProtocol, &mut [u8]) -> Result<usize>> Protocol for LoadFileProtocol<'a, C> {
    type FfiType = EFI_LOAD_FILE_PROTOCOL;
    fn guid() -> Guid {
        EFI_LOAD_FILE_PROTOCOL_GUID
    }
}

impl<'a, C: 'a + FnMut(&DevicePathProtocol, &mut [u8]) -> Result<usize>> Wrapper for LoadFileProtocol<'a, C> {
     type Inner = EFI_LOAD_FILE_PROTOCOL;
    fn inner_ptr(&self) -> *const Self::Inner {
        &self.inner
    }
}

pub extern "win64" fn load_file_callback<'a, C: 'a + FnMut(&DevicePathProtocol, &mut [u8]) -> Result<usize>>(
    this: *const EFI_LOAD_FILE_PROTOCOL, 
    file_path: *const EFI_DEVICE_PATH_PROTOCOL,
    _boot_policy: BOOLEAN,
    buffer_size: *mut UINTN, 
    buffer_ptr: *mut VOID
) -> EFI_STATUS {

    // TODO: Properly handle boot_policy also
    let load_file_protocol: &mut LoadFileProtocol<'a, C> = unsafe { mem::transmute(this) };

    let file_path: &DevicePathProtocol = unsafe { mem::transmute(file_path) };

    match (load_file_protocol.load_file)(file_path, unsafe { slice::from_raw_parts_mut(buffer_ptr as *mut u8, *buffer_size) }) {
        Ok(bytes_written) => { 
            unsafe { *buffer_size  = bytes_written };
            EFI_SUCCESS
        },
        Err(e) => e.into()
    }
}

