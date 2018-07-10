use ffi::{
    IsSuccess,
    CHAR16,
    FALSE,
    device_path::{
        MEDIA_FILEPATH_DP,
        MEDIA_DEVICE_PATH,
        EFI_DEVICE_PATH_PROTOCOL,
        EFI_DEVICE_PATH_UTILITIES_PROTOCOL,
        EFI_DEVICE_PATH_UTILITIES_PROTOCOL_GUID,
        EFI_DEVICE_PATH_TO_TEXT_PROTOCOL,
        EFI_DEVICE_PATH_TO_TEXT_PROTOCOL_GUID,
    },
    UINT16,
};

use {EfiErrorKind, Result, utils::as_slice};
use core::{mem, ptr, fmt, slice};
use system_table;
use alloc::{String, boxed::Box, Vec};

// TODO: the whole concept of wrapping device path pointers like
// this is not safe. We need to analyze memory lifetimes etc.
// to make this safe. Either that or declare it unsafe or drop the idea.

// TODO: should we deallocate underlying pointer in drop? 
// How do you dealloc a path in UEFI?
pub struct DevicePath{ 
    pub (crate) inner: *const EFI_DEVICE_PATH_PROTOCOL, // TODO: should we not use NotNull here? In fact in all such wrapper structs?
    pub (crate) is_single_node: bool, // Indicates that this is a single node (i.e. without an end node). TODO: This shit is ugly. Ideally node and path should be separate types
}

impl DevicePath {
    pub fn as_ptr(&self) -> *const EFI_DEVICE_PATH_PROTOCOL {
        self.inner
    }

    // TODO: right now the args are weakly-typed. Will make it strongly typed later (will need an enum for node types and sub-types)
    /// Creates a path with a single node.
    /// It is unsafe because there's no guarantee that `data` pointer is valid for `length` elements
    pub fn with_single_node<'a, D: Into<Option<&'a [u8]>>>(node_type: u8, node_subtype: u8, data: D) -> Result<Self> {
        let dev_path_utils = path_utils()?;
        let data = data.into().unwrap_or(&[]);

        let node = unsafe {
            const DEV_PATH_NODE_HEADER_SIZE: usize = 4;
            let node  = ((*dev_path_utils).CreateDeviceNode)(node_type, node_subtype, (data.len() + DEV_PATH_NODE_HEADER_SIZE) as UINT16); // safe to cast to UINT16 since we know the file name is pretty short
            let node_data_start = (node as *mut u8).offset(DEV_PATH_NODE_HEADER_SIZE as isize);
            ptr::copy_nonoverlapping(data.as_ptr(), node_data_start, data.len());
            node
        };
        
        Ok(DevicePath{ inner: node, is_single_node: true })
    }

    pub fn try_clone(&self) -> Result<Self> {
        let dev_path_utils = path_utils()?;

        let path = unsafe {
            ((*dev_path_utils).DuplicateDevicePath)(self.as_ptr())
        };

        Ok(DevicePath { inner: path, is_single_node: self.is_single_node })
    }
}

impl fmt::Display for DevicePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bs = (*system_table()).BootServices;

        let protocol: *mut EFI_DEVICE_PATH_TO_TEXT_PROTOCOL   = ptr::null_mut();
        unsafe {
            // TODO: Are we supposed to call CloseProtocol on a protocol pointer obtained via LocateProtocol?
            // UEFI documentation seems to suggest it's not required but doesn't the firmeware need to know we're
            // no longer using the pointer and hence if needed it can clean it up? Check this.
            let status = ((*bs).LocateProtocol)(&EFI_DEVICE_PATH_TO_TEXT_PROTOCOL_GUID, ptr::null(), mem::transmute(&protocol));

            if !IsSuccess(status) {
                return Err(fmt::Error);
            }

            if protocol.is_null() { // If above call returned null protocol that means no such protocol is associated with the handle (which is odd)
                return Err(fmt::Error); // TODO: Need proper error here
            }
        }

        let text_ptr = unsafe { if self.is_single_node {
            ((*protocol).ConvertDeviceNodeToText)(self.as_ptr(), FALSE, FALSE)
        } else {
            ((*protocol).ConvertDevicePathToText)(self.as_ptr(), FALSE, FALSE)
        }} as *mut CHAR16 ;

        let utf16_buf = unsafe { as_slice(text_ptr) };

        let display_utf8 = String::from_utf16(utf16_buf).map_err(|_| fmt::Error)?; // TODO: Can we do something to propagate the underlying error?

        write!(f, "{}", display_utf8)?;

        // TODO: the below is dangerous. There are no guarantees how Box 
        // will release this ptr, but we hope it'll call our allocator 
        // which in turn will call UEFI's heap free routine.
        // Secondly, won't the compiler optimize this statement away?
        unsafe { Box::from_raw(text_ptr) };

        Ok(())
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

    // Convert to UTF16, becuase UEFI expects UCS-2 (We can't do anything about non-representable code points coming from UTF8)
    let utf16_buf = relative_file_path.encode_utf16().collect::<Vec<_>>();
    let bytes_buf = unsafe { slice::from_raw_parts(utf16_buf.as_slice().as_ptr() as *const u8, utf16_buf.len() * 2) }; // * 2 because u16 is 2 bytes

    DevicePath::with_single_node(MEDIA_DEVICE_PATH, MEDIA_FILEPATH_DP, bytes_buf)
}

pub fn append_path(path1: &DevicePath, path2: &DevicePath) -> Result<DevicePath> { // TODO: return value should be strongly typed as FileDevicePath 
    let dev_path_utils = path_utils()?;

    let path = unsafe {
        ((*dev_path_utils).AppendDeviceNode)(path1.as_ptr(), path2.as_ptr())
    };

    Ok(DevicePath { inner: path, is_single_node: false })
}