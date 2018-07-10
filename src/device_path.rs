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

use {EfiError, EfiErrorKind, Result, utils::as_slice};
use core::{mem, ptr, fmt, slice};
use system_table;
use alloc::{String, boxed::Box, Vec};

// TODO: the whole concept of wrapping device path pointers like
// this is not safe. We need to analyze memory lifetimes etc.
// to make this safe. Either that or declare it unsafe or drop the idea.

// TODO: Deallocate underlying inner pointer in drop. Blocked on proper implementation of Box on this platform
// TODO: Do we need to clean up the path_utils fucker as well on drop?
pub struct DeviceNode 
{
    inner: *const EFI_DEVICE_PATH_PROTOCOL, // TODO: instead of raw ptr can we carry this in some sort of Box for UEFI (normal box doesn't work apparently)
    
    // TODO: This thing needs to be wrapped in RC-like ref counting mechanism
    // Otherwise we have a shaky situation with respect to Clone
    path_utils: *mut EFI_DEVICE_PATH_UTILITIES_PROTOCOL, // Carrying this around so that we don't have to ask for it 'cause that operation is fallible and don't want to fail later (fucks up Clone impl for example)
}

impl DeviceNode {
    // TODO: right now the args are weakly-typed. Will make it strongly typed later (will need an enum for node types and sub-types)
    pub fn new<'a, D: Into<Option<&'a [u8]>>>(node_type: u8, node_subtype: u8, data: D) -> Result<Self> {
        let path_utils = path_utils()?;
        let data = data.into().unwrap_or(&[]);

        let inner = unsafe {
            const DEV_PATH_NODE_HEADER_SIZE: usize = 4;
            let node  = ((*path_utils).CreateDeviceNode)(node_type, node_subtype, (data.len() + DEV_PATH_NODE_HEADER_SIZE) as UINT16); // safe to cast to UINT16 since we know the file name is pretty short
            let node_data_start = (node as *mut u8).offset(DEV_PATH_NODE_HEADER_SIZE as isize);
            ptr::copy_nonoverlapping(data.as_ptr(), node_data_start, data.len());
            node
        };
        
        Ok(DeviceNode { inner, path_utils })
    }

    pub fn as_ptr(&self) -> *const EFI_DEVICE_PATH_PROTOCOL {
        self.inner
    }

    pub fn into_path(self) -> DevicePath {
        let path = unsafe {
            ((*self.path_utils).AppendDeviceNode)(ptr::null(), self.inner)
            // TODO: ACHTUNG! - must free self.inner at this point or we leak memory
        };

        DevicePath { inner: path, path_utils: self.path_utils }
    }
}

impl fmt::Display for DeviceNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display = to_string(self.inner, false).map_err(|_| fmt::Error)?; // TODO: don't swallow lower level I/O
        write!(f, "{}", display)?;
        Ok(())
    }
}

// TODO: Deallocate underlying inner pointer in drop. Blocked on proper implementation of Box on this platform
// TODO: Do we need to clean up the path_utils fucker as well on drop?
pub struct DevicePath {
    inner: *const EFI_DEVICE_PATH_PROTOCOL, // TODO: instead of raw ptr can we carry this in some sort of Box for UEFI (normal box doesn't work apparently)
    
    // TODO: This thing needs to be wrapped in RC-like ref counting mechanism
    // Otherwise we have a shaky situation with respect to Clone
    path_utils: *mut EFI_DEVICE_PATH_UTILITIES_PROTOCOL, // Carrying this around so that we don't have to ask for it 'cause that operation is fallible and don't want to fail later (fucks up Clone impl for example)
}

impl DevicePath {
    pub (crate) fn from_ptr(ptr: *const EFI_DEVICE_PATH_PROTOCOL) -> Result<Self> {
        Ok(DevicePath { inner: ptr, path_utils: path_utils()? })
    }

    pub fn as_ptr(&self) -> *const EFI_DEVICE_PATH_PROTOCOL {
        self.inner
    }

    pub fn try_clone(&self) -> Result<Self> {
        let path = unsafe {
            ((*self.path_utils).DuplicateDevicePath)(self.inner)
        };

        Ok(DevicePath { inner: path, path_utils: self.path_utils })
    }
}

impl fmt::Display for DevicePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display = to_string(self.inner, false).map_err(|_| fmt::Error)?; // TODO: don't swallow lower level I/O
        write!(f, "{}", display)?;
        Ok(())
    }
}

fn to_string(path: *const EFI_DEVICE_PATH_PROTOCOL, is_single_node: bool) -> Result<String> {
    let bs = (*system_table()).BootServices;

    let protocol: *mut EFI_DEVICE_PATH_TO_TEXT_PROTOCOL   = ptr::null_mut();
    unsafe {
        // TODO: Are we supposed to call CloseProtocol on a protocol pointer obtained via LocateProtocol?
        // UEFI documentation seems to suggest it's not required but doesn't the firmeware need to know we're
        // no longer using the pointer and hence if needed it can clean it up? Check this.
        let status = ((*bs).LocateProtocol)(&EFI_DEVICE_PATH_TO_TEXT_PROTOCOL_GUID, ptr::null(), mem::transmute(&protocol));

        if !IsSuccess(status) {
            return Err(EfiErrorKind::DeviceError.into());
        }

        if protocol.is_null() { // If above call returned null protocol that means no such protocol is associated with the handle (which is odd)
            return Err(EfiErrorKind::DeviceError.into()); // TODO: Need proper error here
        }
    }

    let text_ptr = unsafe { if is_single_node {
        ((*protocol).ConvertDeviceNodeToText)(path, FALSE, FALSE)
    } else {
        ((*protocol).ConvertDevicePathToText)(path, FALSE, FALSE)
    }} as *mut CHAR16 ;

    let utf16_buf = unsafe { as_slice(text_ptr) };

    let utf8_string = String::from_utf16(utf16_buf).map_err(|_| EfiError::from(EfiErrorKind::DeviceError))?; // TODO: Can we do something to propagate the underlying error?

    // TODO: the below is dangerous. There are no guarantees how Box 
    // will release this ptr, but we hope it'll call our allocator 
    // which in turn will call UEFI's heap free routine.
    // Secondly, won't the compiler optimize this statement away?
    unsafe { Box::from_raw(text_ptr) };

    Ok(utf8_string)
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

pub fn create_file_path_node<P: AsRef<str>>(relative_file_path: P) -> Result<DeviceNode> { // TODO: return value should be strongly typed as FileDeviceNode 
    let relative_file_path = relative_file_path.as_ref();

    // Convert to UTF16, becuase UEFI expects UCS-2 (We can't do anything about non-representable code points coming from UTF8)
    let utf16_buf = relative_file_path.encode_utf16().collect::<Vec<_>>();
    let bytes_buf = unsafe { slice::from_raw_parts(utf16_buf.as_slice().as_ptr() as *const u8, utf16_buf.len() * 2) }; // * 2 because u16 is 2 bytes

    DeviceNode::new(MEDIA_DEVICE_PATH, MEDIA_FILEPATH_DP, bytes_buf)
}

pub fn append_path(path1: &DevicePath, path2: &DevicePath) -> Result<DevicePath> { // TODO: return value should be strongly typed as FileDevicePath 
    let dev_path_utils = path_utils()?;

    let path = unsafe {
        ((*dev_path_utils).AppendDeviceNode)(path1.as_ptr(), path2.as_ptr())
    };

    DevicePath::from_ptr(path)
}