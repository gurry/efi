use ffi::{boot_services::{EFI_BOOT_SERVICES, EFI_INTERFACE_TYPE, EFI_ALLOCATE_TYPE, EFI_MEMORY_TYPE}, EFI_HANDLE, UINTN, CHAR16, VOID};
use ::{Result, Guid, Void, to_res, utils::Wrapper, Opaque, OpaqueDevice, OpaqueAgent, OpaqueImage, OpaqueController, to_boolean};
use protocols::{Protocol, DevicePathProtocol};
use core::{ptr, mem, slice};


bitflags! {
    pub struct OpenProtocolAttributes: u32 {
        const BY_HANDLE_PROTOCOL = 0x00000001;
        const GET_PROTOCOL =  0x00000002;
        const BY_TEST_PROTOCOL = 0x00000004;
        const BY_CHILD_CONTROLLER = 0x00000008;
        const BY_DRIVER = 0x00000010;
        const EXCLUSIVE = 0x00000020;
    }
}

#[repr(C)]
pub struct BootServices<'a>(&'a EFI_BOOT_SERVICES);

 impl<'a> BootServices<'a> {
     // TODO: tying the proto's lifetime to the BootServices lifetime param 'a may be less optimal than tying it to some lifetime param on OpaqueDevice.
     // After all it's the device that's going to carry the protocol pointer inside it.
     // For that we may need to expose a lifetime param on OpaqeDevice
     // This whole story about adding memory safety requires more thinking
    pub fn install_protocol_interface<T: Protocol + Wrapper>(&mut self, handle: Option<&'a OpaqueDevice>, protocol: &'a T, interface_type: InterfaceType) -> Result<&'a OpaqueDevice> {
        let handle_ptr: EFI_HANDLE = handle.map_or(ptr::null(), |v| unsafe { mem::transmute(v) });
        let guid_ptr = &T::guid() as *const Guid;

        let status = unsafe {
            (self.0.InstallProtocolInterface)(handle_ptr, guid_ptr, mem::transmute(interface_type), mem::transmute(protocol.inner_ptr()))
        };

        to_res(unsafe { mem::transmute(handle_ptr) }, status)
    }

    pub fn open_protocol<T: Protocol>(&self, handle: &Opaque, agent_handle: &OpaqueAgent, controller_handle: Option<&OpaqueController>, attributes: OpenProtocolAttributes) -> Result<&T> {
        let guid_ptr = &T::guid() as *const Guid;
        let controller_handle_ptr: EFI_HANDLE = controller_handle.map_or(ptr::null(), |v| unsafe { mem::transmute(v) });
        let mut protocol: *mut T::FfiType = ptr::null_mut();

        let status = unsafe {
            (self.0.OpenProtocol)(mem::transmute(handle), guid_ptr, mem::transmute(&mut protocol), mem::transmute(agent_handle), controller_handle_ptr, attributes.bits())
        };

        to_res(unsafe { mem::transmute(protocol) }, status)

    }

    pub fn close_protocol<T: Protocol>(&self, handle: &Opaque, agent_handle: &OpaqueAgent, controller_handle: Option<&OpaqueController>) -> Result<()> {
        let guid_ptr = &T::guid() as *const Guid;
        let controller_handle_ptr: EFI_HANDLE = controller_handle.map_or(ptr::null(), |v| unsafe { mem::transmute(v) });

        let status = unsafe {
            (self.0.CloseProtocol)(mem::transmute(handle), guid_ptr, mem::transmute(agent_handle), controller_handle_ptr)
        };

        to_res((), status)

    }

    pub fn locate_protocol<T: Protocol>(&self) -> Result<&T> {
        // TODO: add the 'registration' argument also to this method
        let guid_ptr = &T::guid() as *const Guid;
        let registration: *mut Void = ptr::null_mut();
        let mut protocol: *mut T::FfiType = ptr::null_mut();

        let status = unsafe {
            (self.0.LocateProtocol)(guid_ptr, registration, mem::transmute(&mut protocol))
        };

        to_res(unsafe { mem::transmute(protocol) }, status)
    }

    // TODO: instead of exposing both image params and source buffer should we put these options inside an enum and take an enum?
    // That way there's no chance of the caller specifying both kinds of inputs
    pub fn load_image(&mut self, boot_policy: bool, parent_image_handle: &'a OpaqueImage, device_path: &'a DevicePathProtocol, source_buffer: Option<&'a [u8]>) -> Result<&'a OpaqueImage> {
        let source_buf_ptr : *const VOID = source_buffer.map_or(ptr::null(), |v| unsafe { mem::transmute(v.as_ptr()) });
        let source_buf_len = source_buffer.map_or(0, |v| v.len());
        let image_handle: *mut *const VOID = ptr::null_mut();

        let status = unsafe {
            (self.0.LoadImage)(to_boolean(boot_policy), mem::transmute(parent_image_handle), mem::transmute(device_path), source_buf_ptr, source_buf_len, image_handle)
        };

        to_res(unsafe { mem::transmute(image_handle) }, status)
    }

    pub fn start_image(&mut self,  image_handle: &'a OpaqueImage) -> Result<&[u16]> {
        let exit_data_size: UINTN = 0;
        let exit_data: *const CHAR16 = ptr::null();

        let status = unsafe {
            (self.0.StartImage)(mem::transmute(image_handle), mem::transmute(&exit_data_size), mem::transmute(&exit_data))
        };

        to_res(unsafe { slice::from_raw_parts(exit_data, exit_data_size) }, status)
    }
}

pub enum InterfaceType {
    NativeInterface
}

impl From<EFI_INTERFACE_TYPE> for InterfaceType {
    fn from(value: EFI_INTERFACE_TYPE) -> Self {
        match value {
            EFI_INTERFACE_TYPE::EFI_NATIVE_INTERFACE => InterfaceType::NativeInterface
        }
    }
}

impl From<InterfaceType> for EFI_INTERFACE_TYPE {
    fn from(value: InterfaceType) -> Self {
        match value {
            InterfaceType::NativeInterface => EFI_INTERFACE_TYPE::EFI_NATIVE_INTERFACE 
        }
    }
}

pub type AllocateType = EFI_ALLOCATE_TYPE;

pub type MemoryType = EFI_MEMORY_TYPE;