use ffi::{boot_services::{EFI_BOOT_SERVICES, EFI_INTERFACE_TYPE}, EFI_HANDLE};
use ::{Result, Guid, Void, to_res, utils::Wrapper, Opaque, OpaqueDevice, OpaqueAgent, OpaqueController};
use protocols::Protocol;
use core::{ptr, mem};


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
pub struct BootServices(EFI_BOOT_SERVICES);

 impl<'a> BootServices {
     // TODO: the lifetime annotations on this method may not be enough enforce the lifetime required on the  protocol argument (i.e. it should remain alive as long as it's installed)
     // So take a look at them again
    pub fn install_protocol_interface<T: Protocol + Wrapper>(&'a self, handle: Option<&'a OpaqueDevice>, protocol: &'a T, interface_type: InterfaceType) -> Result<&'a OpaqueDevice> {
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