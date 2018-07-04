use ffi::device_path::EFI_DEVICE_PATH_PROTOCOL;

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