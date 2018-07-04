use ffi::{
    device_path::{
        MEDIA_FILEPATH_DP,
        MEDIA_DEVICE_PATH,
        EFI_DEVICE_PATH_PROTOCOL,
        EFI_DEVICE_PATH_UTILITIES_PROTOCOL,
        EFI_DEVICE_PATH_UTILITIES_PROTOCOL_GUID,
    },
    UINT16,
};

use {EfiErrorKind, Result};
use core::{mem, ptr};
use system_table;

// TODO: the whole concept of wrapping device path pointers like
// this is not safe. We need to analyze memory lifetimes etc.
// to make this safe. Either that or declare it unsafe or drop the idea.

// TODO: should we deallocate underlying pointer in drop? 
// How do you dealloc a path in UEFI?
pub struct DevicePath(pub (crate) *const EFI_DEVICE_PATH_PROTOCOL);

impl DevicePath {
    pub (crate) fn as_ptr(&self) -> *const EFI_DEVICE_PATH_PROTOCOL {
        self.0
    }
}

// TODO: Make device paths strongly typed by introducing independent types for
// different kinds of device paths like file, harddrive, usb, ipv4 and so on.

// pub struct FileDevicePath {
// }

fn path_utils() -> Result<*mut EFI_DEVICE_PATH_UTILITIES_PROTOCOL> {
    // TODO: Don't "locate" this protocol every time. Do it once and keep a global pointer.
    let bs = (*system_table()).BootServices;

    let utils: *mut EFI_DEVICE_PATH_UTILITIES_PROTOCOL   = ptr::null_mut();
    unsafe {
        // TODO: Are we supposed to call CloseProtocol on a protocol pointer obtained via LocateProtocol?
        // UEFI documentation seems to suggest it's not required but doesn't the firmeware need to know we're
        // no longer using the pointer and hence if needed it can clean it up? Check this.
        ret_on_err!(((*bs).LocateProtocol)(&EFI_DEVICE_PATH_UTILITIES_PROTOCOL_GUID, ptr::null(), mem::transmute(&utils)));

        if utils.is_null() { // If above call returned null protocol that means no such protocol is associated with the handle (which is odd)
            return Err(EfiErrorKind::LoadError.into()); // TODO: Need proper error here
        }
    }

    Ok(utils)
}

pub fn create_file_path_node<P: AsRef<str>>(relative_file_path: P) -> Result<DevicePath> { // TODO: return value should be strongly typed as FileDevicePath 
    let relative_file_path = relative_file_path.as_ref();
    let dev_path_utils = path_utils()?;

    let file_path_node = unsafe {
        const DEV_PATH_NODE_HEADER_SIZE: usize = 4;
        let file_path_node  = ((*dev_path_utils).CreateDeviceNode)(MEDIA_DEVICE_PATH, MEDIA_FILEPATH_DP, (relative_file_path.len() + DEV_PATH_NODE_HEADER_SIZE) as UINT16); // safe to cast to UINT16 since we know the file name is pretty short
        let node_data_start: *mut u8 = (file_path_node as *mut u8).offset(DEV_PATH_NODE_HEADER_SIZE as isize);
        ptr::copy_nonoverlapping(relative_file_path.as_ptr(), node_data_start, relative_file_path.len());

        file_path_node
    };
    
    Ok(DevicePath(file_path_node))
}

pub fn append_path(path1: &DevicePath, path2: &DevicePath) -> Result<DevicePath> { // TODO: return value should be strongly typed as FileDevicePath 
    let dev_path_utils = path_utils()?;

    let path = unsafe {
        ((*dev_path_utils).AppendDeviceNode)(path1.as_ptr(), path2.as_ptr())
    };

    Ok(DevicePath(path))
}