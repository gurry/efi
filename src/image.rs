use {Result, io::{self, Read}, system_table, image_handle, EfiErrorKind};
use ffi::{
    media::{EFI_LOAD_FILE_PROTOCOL, EFI_LOAD_FILE_PROTOCOL_GUID}, 
    loaded_image::{EFI_LOADED_IMAGE_PROTOCOL, EFI_LOADED_IMAGE_PROTOCOL_GUID},
    device_path::{EFI_DEVICE_PATH_PROTOCOL, EFI_DEVICE_PATH_PROTOCOL_GUID},
    EFI_HANDLE,
    EFI_STATUS,
    EFI_SUCCESS,
    EFI_BUFFER_TOO_SMALL,
    EFI_INVALID_PARAMETER,
    EFI_DEVICE_ERROR,
    boot_services::{EFI_INTERFACE_TYPE, EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL},
    UINTN,
    CHAR16,
    BOOLEAN,
    VOID,
    FALSE,
};
use device_path::{DevicePath, create_file_path_node, append_path};
use core::{self, ptr, mem, slice, cmp};
use alloc::vec::Vec;


// TODO: we should create a virtualfs (filesystem) and put all our images there.
// Each image will have its own file path. Then we can call LoadImage on these file paths.
// And there will be a way to enumerate through all these images as well

/// A trait that provides the length of the object that implements it.
/// An example can be a file implementing this interface to expose a way to get its length.
pub trait Len { // TODO: Move this a more general module like 'io' or something.
    fn len(&mut self) -> Result<Option<u64>>; // TODO: was forced to use &mut self because some reaers like HTTP reader mutated when the read lenght (e.g. do a PUT request on their underlying HTTP stream and thus mutating it). Is interior mutability the answer?
}

// TODO: this whole shit about wrapping raw paths into DevicePath type is unsafe. Address this unsafety
pub fn load_image_from_path(path: &mut DevicePath) -> Result<LoadedImage> {
    let bs = (*system_table()).BootServices;
    let current_image_handle = image_handle();
    let path = path.as_ptr();

    let loaded_img_handle = unsafe {
        let mut loaded_img_handle: EFI_HANDLE = ptr::null_mut();
        ret_on_err!(((*bs).LoadImage)(FALSE, current_image_handle, path, ptr::null(), 0, &mut loaded_img_handle)); // TODO: should we pass true or false to first arg? What difference does it make? Should we expose it out to the caller?
        loaded_img_handle
    };

    Ok(LoadedImage(loaded_img_handle))
}

//TODO: Provide a way for the user to specify load options as well
/// Loads image read from the given reader
pub fn load_image<R: Read + Len>(reader: &mut R) -> Result<LoadedImage> {
    let loader = Loader::new(reader);
    let bs = (*system_table()).BootServices;

    let (mut image_path, device_handle) = unsafe {
        // Install our load file protocol and get a newly generated handle to it
        let mut device_handle: EFI_HANDLE = ptr::null_mut();
        ret_on_err!(((*bs).InstallProtocolInterface)(&mut device_handle, &EFI_LOAD_FILE_PROTOCOL_GUID, EFI_INTERFACE_TYPE::EFI_NATIVE_INTERFACE, mem::transmute(&loader.proto)));

        // Open loaded image protocol on the currently running image in order to obtain its device handle
        let current_image_handle = image_handle();
        let loaded_image: *mut EFI_LOADED_IMAGE_PROTOCOL = ptr::null_mut();
        ret_on_err!(((*bs).OpenProtocol)(current_image_handle, &EFI_LOADED_IMAGE_PROTOCOL_GUID, mem::transmute(&loaded_image), current_image_handle, ptr::null(), EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL)); // TODO: should we use GET_PROTOCOL instead of BY_HANDLE_PROTOCOL? Not clear from UEFI documentation.


        if loaded_image.is_null() { // If above call returned null protocol that means no such protocol is associated with the handle (which is odd)
            return Err(EfiErrorKind::LoadError.into()); // TODO: Need proper error here
        }

        ret_on_err!(((*bs).CloseProtocol)(current_image_handle, &EFI_LOADED_IMAGE_PROTOCOL_GUID, current_image_handle, ptr::null()));

        // Open device path protocol on the device handle of the currently running image
        let current_image_device_path: *mut EFI_DEVICE_PATH_PROTOCOL  = ptr::null_mut();
        ret_on_err!(((*bs).OpenProtocol)((*loaded_image).DeviceHandle, &EFI_DEVICE_PATH_PROTOCOL_GUID, mem::transmute(&current_image_device_path), current_image_handle, ptr::null(), EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL)); // TODO: should we use GET_PROTOCOL instead of BY_HANDLE_PROTOCOL? Not clear from UEFI documentation.

        if current_image_device_path.is_null() { // If above call returned null protocol that means no such protocol is associated with the handle (which is odd)
            return Err(EfiErrorKind::LoadError.into()); // TODO: Need proper error here
        }

        ret_on_err!(((*bs).CloseProtocol)((*loaded_image).DeviceHandle, &EFI_DEVICE_PATH_PROTOCOL_GUID, current_image_handle, ptr::null()));

        // Create a new device path and associate it with the our load file protocol. This path will be used for loading the image in LoadImage EFI call later
        let dummy_image_file_name = "image_file";
        let file_path_node = create_file_path_node(dummy_image_file_name)?.into_path();
        let current_image_device_path = DevicePath::from_ptr(current_image_device_path)?;
        let image_path = append_path(&current_image_device_path, &file_path_node)?; // TODO: Is this appraoch okay? Should we create a more proper path than this?
        ret_on_err!(((*bs).InstallProtocolInterface)(&mut device_handle, &EFI_DEVICE_PATH_PROTOCOL_GUID, EFI_INTERFACE_TYPE::EFI_NATIVE_INTERFACE, mem::transmute(image_path.as_ptr())));

        (image_path, device_handle)
    };

    let loaded_image = load_image_from_path(&mut image_path);

    unsafe {
        // Uninstall the load file and device path protocols since our protocol handle is about to go out of scope
        // TODO: how will the device_handle be deallocated?
        ret_on_err!(((*bs).UninstallProtocolInterface)(device_handle, &EFI_LOAD_FILE_PROTOCOL_GUID, mem::transmute(&loader.proto)));
    }

    loaded_image
}

/// Starts an image previously loaded using load_image
pub fn start_image(image: &LoadedImage ) -> Result<ExitData> {
    let bs = (*system_table()).BootServices;

    unsafe {
        let mut exit_data_size: UINTN = 0;
        let mut exit_data_ptr = ptr::null_mut() as *const CHAR16;
        ret_on_err!(((*bs).StartImage)(image.0, &mut exit_data_size, &mut exit_data_ptr));
        Ok(ExitData::from_raw_parts(exit_data_ptr, exit_data_size)) // TODO: Will exit_data_ptr ever be null? Test this by starting an image that doesn't call Exit()
    }
}

#[repr(C)] // repr C needed so that we can safely transmute back to this struct in load_file_callback below
struct Loader<'a, R: 'a + Read + Len> {
    proto: EFI_LOAD_FILE_PROTOCOL,
    reader: &'a mut R,
    cached_len: Option<u64>,
}

impl<'a, R: 'a + Read + Len> Loader<'a, R> {
    fn new(reader: &'a mut R) -> Self {
        Self { proto: EFI_LOAD_FILE_PROTOCOL { LoadFile: load_file_callback::<'a, R> }, reader, cached_len: None }
    }
}

extern "win64" fn load_file_callback<'a, R: 'a + Read + Len>(
    this: *const EFI_LOAD_FILE_PROTOCOL, 
    file_path: *const EFI_DEVICE_PATH_PROTOCOL,
    _boot_policy: BOOLEAN,
    buffer_size: *mut UINTN, 
    buffer_ptr: *mut VOID
) -> EFI_STATUS {
    // TODO: Properly handle boot_policy also

    if this.is_null() || file_path.is_null() || buffer_size.is_null() {
        return EFI_INVALID_PARAMETER;
    }

    let loader: &mut Loader<'a, R> = unsafe { mem::transmute(this) }; // Should be safe to do this transmute since Loader is marked repr C

    // Get file length once and cache it in loader.
    // We cache because for many readers it may be expesive to get the length.
    // E.g. in HTTP it will result in a HEAD call each time.
    if loader.cached_len.is_none() {
        loader.cached_len = match loader.reader.len() {
            Ok(Some(l)) => Some(l),
            Ok(None) => return EfiErrorKind::DeviceError.into(),
            Err(e) => return e.into()
        }
    };

    let file_len = loader.cached_len.expect("len should not have been none here");
    let incoming_buf_size = unsafe { *buffer_size };

    // If buffer_ptr is null the firmware just wants to know the size of the file
    // So we just set that and return
    if buffer_ptr.is_null() {
        // There's a chance *buffer_size can overflow
        // because file_len is u64 while *buffer_size
        // is UINTN (which is an alias for usize).
        // So here we just error out if it overflows.
        if (core::usize::MAX as u64) < file_len {
            return EFI_DEVICE_ERROR;
        }

        unsafe { *buffer_size  = file_len as UINTN };

        // The UEFI spec 2.4 DOES NOT say that we should return EFI_BUFFER_TOO_SMALL
        // when the incoming buffer_ptr is null. However, if you return any other status,
        // such as EFI_SUCCESS, it doesn't work. The firmware never tries to downlaod file
        // in that case.
        return EFI_BUFFER_TOO_SMALL;
    } 

    // As per UEFI spec, if the provided buf is smaller 
    // than the file we have, we must return EFI_BUFFER_TOO_SMALL
    if (incoming_buf_size as u64) < file_len { 
        return EFI_BUFFER_TOO_SMALL
    }

    // Everything good. Let's read the data.
    let mut buf = unsafe { slice::from_raw_parts_mut(buffer_ptr as *mut u8, *buffer_size) };
    match io::fill_buf(&mut loader.reader, &mut buf) {
        Ok(bytes_read) => {
            unsafe { *buffer_size = bytes_read };
            EFI_SUCCESS
        },
        Err(_) => EFI_DEVICE_ERROR,
    }
}


#[derive(Debug)]
pub struct LoadedImage(EFI_HANDLE);

/// The data returned by a running image when it exits.
/// Contains a UCS-2 string part followed by an optional binary data.
/// The interpretation of the binary data part is up to the application
#[derive(Debug)]
pub struct ExitData {
    ptr: *const CHAR16,
    size_in_bytes: UINTN,
    str_end: usize,
}

impl ExitData {
    fn from_raw_parts(ptr: *const CHAR16, size_in_bytes: UINTN) -> Self {
        let buf = Self::create_slice(ptr, size_in_bytes);
        let str_end = buf.iter().position(|c| *c == 0).unwrap_or(buf.len()); // End of str part is the first ocurrence of null terminator or failing that the end of the buf itself
        Self { ptr, size_in_bytes, str_end } 
    }
    
    /// String part of exit data. It is a UCS-2 string with NO null terminator.
    pub fn str_part(&self) -> &[u16] {
        let buf = self.as_slice();
        &buf[..self.str_end]
    }

    /// Binary part of exit data.
    pub fn binary_part(&self) -> &[u16] {
        let buf = self.as_slice();
        let bin_start = cmp::min(self.str_end + 1, buf.len()); // min because we don't want to overshoot the buf end 
        &buf[bin_start..]
    }

    /// The whole buffer with string and binary parts together
    pub fn as_slice(&self) -> &[u16] {
        Self::create_slice(self.ptr, self.size_in_bytes)
    }

    fn create_slice<'a>(ptr: *const CHAR16, size_in_bytes: UINTN) -> &'a [u16] {
        unsafe { slice::from_raw_parts(ptr, size_in_bytes / 2) } // Dividing by two since slice::from_raw_parts' second arg is size in elements not bytes
    }
}

impl Drop for ExitData {
    fn drop(&mut self) { // The exit data ptr is allocated by the image we loaded but must be deallocated by us as per UEFI spec
        let bs = (*system_table()).BootServices;
        unsafe { ((*bs).FreePool)(self.ptr as *const VOID) }; // TODO: Can't do anything if this fails except. So we should log here
    }
}

impl<'a> Len for &'a[u8] {
    fn len(&mut self) -> Result<Option<u64>> {
        Ok(Some(<[u8]>::len(self) as u64))
    }
}

impl Len for io::Cursor<Vec<u8>> {
    fn len(&mut self) -> Result<Option<u64>> {
        Ok(Some(self.get_ref().len() as u64))
    }
}