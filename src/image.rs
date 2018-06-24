use {Result, io::{self, Read}, system_table, image_handle, EfiErrorKind};
use ffi::{
    load_file::{EFI_LOAD_FILE_PROTOCOL, EFI_LOAD_FILE_PROTOCOL_GUID}, 
    loaded_image::{EFI_LOADED_IMAGE_PROTOCOL, EFI_LOADED_IMAGE_PROTOCOL_GUID},
    device_path::{EFI_DEVICE_PATH_PROTOCOL,
        EFI_DEVICE_PATH_PROTOCOL_GUID,
        EFI_DEVICE_PATH_UTILITIES_PROTOCOL,
        EFI_DEVICE_PATH_UTILITIES_PROTOCOL_GUID,
        MEDIA_DEVICE_PATH,
        MEDIA_FILEPATH_DP,
    },
    EFI_HANDLE,
    EFI_STATUS,
    EFI_SUCCESS,
    EFI_BUFFER_TOO_SMALL,
    EFI_INVALID_PARAMETER,
    EFI_DEVICE_ERROR,
    boot_services::{EFI_INTERFACE_TYPE, EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL},
    UINTN,
    UINT16,
    CHAR16,
    BOOLEAN,
    VOID,
    FALSE,
};
use core::{ptr, mem, slice, cmp};


// TODO: we should create a virtualfs (filesystem) and put all our images there.
// Each image will have its own file path. Then we can call LoadImage on these file paths.
// And there will be a way to enumerate through all these images as well

/// A trait that provides the length of the object that implements it.
/// An example can be a file implementing this interface to expose a way to get its length.
pub trait Len { // TODO: Move this a more general module like 'io' or something.
    fn len(&mut self) -> Result<usize>; // TODO: was forced to use &mut self because some reaers like HTTP reader mutated when the read lenght (e.g. do a PUT request on their underlying HTTP stream and thus mutating it). Is interior mutability the answer?
}

/// Loads image read from the given reader
pub fn load_image<R: Read + Len>(reader: &mut R) -> Result<LoadedImage> {
    let loader = Loader::new(reader);
    let bs = (*system_table()).BootServices;

    let loaded_img_handle = unsafe {
        // Install our load file protocol and get a newly generated handle to it
        let mut proto_handle: EFI_HANDLE = ptr::null_mut();
        ret_on_err!(((*bs).InstallProtocolInterface)(&mut proto_handle, &EFI_LOAD_FILE_PROTOCOL_GUID, EFI_INTERFACE_TYPE::EFI_NATIVE_INTERFACE, mem::transmute(&loader.proto)));

        // Open loaded image protocol on the currently running image in order to obtain its device handle
        let current_image_handle = image_handle();
        let loaded_image: *mut EFI_LOADED_IMAGE_PROTOCOL = ptr::null_mut();
        ret_on_err!(((*bs).OpenProtocol)(current_image_handle, &EFI_LOADED_IMAGE_PROTOCOL_GUID, mem::transmute(&loaded_image), current_image_handle, ptr::null(), EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL));

        if loaded_image.is_null() { // If above call returned null protocol that means no such protocol is associated with the handle (which is odd)
            return Err(EfiErrorKind::LoadError.into()); // TODO: Need proper error here
        }

        // Open device path protocol on the device handle of the currently running image
        let current_image_device_path: *mut EFI_DEVICE_PATH_PROTOCOL  = ptr::null_mut();
        ret_on_err!(((*bs).OpenProtocol)((*loaded_image).DeviceHandle, &EFI_DEVICE_PATH_PROTOCOL_GUID, mem::transmute(&current_image_device_path), current_image_handle, ptr::null(), EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL));

        if current_image_device_path.is_null() { // If above call returned null protocol that means no such protocol is associated with the handle (which is odd)
            return Err(EfiErrorKind::LoadError.into()); // TODO: Need proper error here
        }

        // Get hold of device path utils proto which we use below
        let dev_path_utils: *mut EFI_DEVICE_PATH_UTILITIES_PROTOCOL   = ptr::null_mut();
        ret_on_err!(((*bs).LocateProtocol)(&EFI_DEVICE_PATH_UTILITIES_PROTOCOL_GUID, ptr::null(), mem::transmute(&dev_path_utils)));

        if dev_path_utils.is_null() { // If above call returned null protocol that means no such protocol is associated with the handle (which is odd)
            return Err(EfiErrorKind::LoadError.into()); // TODO: Need proper error here
        }

        // Close loaded image and device path protocols. Don't need them anymore
        ret_on_err!(((*bs).CloseProtocol)((*loaded_image).DeviceHandle, &EFI_DEVICE_PATH_PROTOCOL_GUID, current_image_handle, ptr::null()));
        ret_on_err!(((*bs).CloseProtocol)(current_image_handle, &EFI_LOADED_IMAGE_PROTOCOL_GUID, current_image_handle, ptr::null()));

        // TODO: Must create a safe RAII based and ergonomic abstraction over device paths.
        // It could implement iterator over nodes and have a UEFI-spec-compliant display impl
        let dummy_image_file_name = "image_file";
        let file_path_node  = ((*dev_path_utils).CreateDeviceNode)(MEDIA_DEVICE_PATH, MEDIA_FILEPATH_DP, dummy_image_file_name.len() as UINT16); // safe to cast to UINT16 since we know the file name is pretty short
        const DEV_PATH_NODE_HEADER_SIZE: usize = 4;
        let node_data_start: *mut u8 = (file_path_node as *mut u8).offset(DEV_PATH_NODE_HEADER_SIZE as isize);
        ptr::copy_nonoverlapping(dummy_image_file_name.as_ptr(), node_data_start, dummy_image_file_name.len());

        // Create a new device path and associate it with the our load file protocol. This path will be used for loading the image in LoadImage EFI call later
        let dev_path = ((*dev_path_utils).AppendDeviceNode)(current_image_device_path, file_path_node); // TODO: Is this appraoch okay? Should we create a more proper path than this?
        ret_on_err!(((*bs).InstallProtocolInterface)(&mut proto_handle, &EFI_DEVICE_PATH_PROTOCOL_GUID, EFI_INTERFACE_TYPE::EFI_NATIVE_INTERFACE, mem::transmute(dev_path)));

        // Now load the image using the new device path
        let mut loaded_img_handle: EFI_HANDLE = ptr::null_mut();
        let loadimg_status = ((*bs).LoadImage)(FALSE, current_image_handle, dev_path, ptr::null(), 0, &mut loaded_img_handle); // TODO: should we pass true or false to first arg? What difference does it make? Should we expose it out to the caller?

        // Uninstall the load file and device path protocols since our load file object is about to go out of scope
        ret_on_err!(((*bs).UninstallProtocolInterface)(proto_handle, &EFI_DEVICE_PATH_PROTOCOL_GUID, mem::transmute(&dev_path)));
        ret_on_err!(((*bs).UninstallProtocolInterface)(proto_handle, &EFI_LOAD_FILE_PROTOCOL_GUID, mem::transmute(&loader.proto)));

        // We check LoadImage status here because we want to run the above two 
        // uninstall calls regardless of whether LoadImage failed or succeeded
        ret_on_err!(loadimg_status);

        // TODO: IMPORTANT - must uninstall the protocols even if one of the preceding calls (like LoadImage) fails. Can we use RAII somehow to ensure this? May be put all the unsafe shit above inside the Loader and then impl Drop for it?

        loaded_img_handle
    };

    Ok(LoadedImage(loaded_img_handle))
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
    cached_len: Option<usize>,
}

impl<'a, R: 'a + Read + Len> Loader<'a, R> {
    fn new(reader: &'a mut R) -> Self {
        Self { proto: EFI_LOAD_FILE_PROTOCOL { LoadFile: load_file_callback::<'a, R> }, reader, cached_len: None }
    }
}

pub extern "win64" fn load_file_callback<'a, R: 'a + Read + Len>(
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
            Ok(l) => Some(l),
            Err(e) => return e.into()
        }
    };

    let file_len = loader.cached_len.expect("len should not have been none here");
    let incoming_buf_size = unsafe { *buffer_size };

    // If buffer_ptr is null the firmware just wants to know the size of the file
    // So we just set that and return
    if buffer_ptr.is_null() {
        unsafe { *buffer_size  = file_len };

        // The UEFI spec 2.4 DOES NOT say that we should return EFI_BUFFER_TOO_SMALL
        // when the incoming buffer_ptr is null. However, if you return any other status,
        // such as EFI_SUCCESS, it doesn't work. The firmware never tries to downlaod file
        // in that case.
        return EFI_BUFFER_TOO_SMALL;
    } 

    // As per UEFI spec, if the provided buf is smaller 
    // than the file we have, we must return EFI_BUFFER_TOO_SMALL
    if incoming_buf_size < file_len { 
        return EFI_BUFFER_TOO_SMALL
    }

    // Everything good. Let's read the data.
    let mut buf = unsafe { slice::from_raw_parts_mut(buffer_ptr as *mut u8, *buffer_size) };
    match read_to_fill_buf(&mut loader.reader, &mut buf) {
        Ok(bytes_read) => {
            unsafe { *buffer_size = bytes_read };
            EFI_SUCCESS
        },
        Err(_) => EFI_DEVICE_ERROR,
    }
}

fn read_to_fill_buf<R: Read>(reader: &mut R, buf: &mut [u8]) -> io::Result<usize> {
    let mut bytes_read = 0;
    loop {
        match reader.read(&mut buf[bytes_read..]) {
            Ok(n) => {
                bytes_read += n;
                if n == 0 || bytes_read == buf.len() { // Either EOF or we filled the buf
                    return Ok(bytes_read)
                } else if bytes_read > buf.len() { // WTF. Should never happen 
                    return Err(io::ErrorKind::Other.into())
                }
            },
            Err(_) => return Err(io::ErrorKind::Other.into()) , // TODO: Do not swallow upstream error here
        }
    }
}

pub struct LoadedImage(EFI_HANDLE);

/// The data returned by a running image when it exits.
/// Contains a UCS-2 string part followed by an optional binary data.
/// The interpretation of the binary data part is up to the application
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
    fn len(&mut self) -> Result<usize> {
        Ok(<[u8]>::len(self))
    }
}