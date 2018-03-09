use ffi::{boot_services::{EFI_BOOT_SERVICES, EFI_INTERFACE_TYPE}, EFI_HANDLE};
use ::{Result, Guid, Void, to_res, utils::Wrapper};
use protocols::Protocol;
use core::{ptr, mem};


#[repr(C)]
pub struct DeviceHandle(EFI_HANDLE);

#[repr(C)]
pub struct BootServices(EFI_BOOT_SERVICES);

 impl<'a> BootServices {
    pub fn install_protocol_interface<T: Protocol + Wrapper>(&'a self, handle: Option<&'a DeviceHandle>, protocol: &'a T, interface_type: InterfaceType) -> Result<&'a DeviceHandle> {
        let guid_ptr = &T::guid() as *const Guid;
        
        let handle_ptr: *const EFI_HANDLE = handle.map_or(ptr::null(), |v| unsafe { mem::transmute(v) });

        let status = unsafe {
            (self.0.InstallProtocolInterface)(mem::transmute(handle), guid_ptr, mem::transmute(interface_type), mem::transmute(protocol.inner_ptr()))
        };

        to_res(unsafe { mem::transmute(handle_ptr) }, status)
    }

    // TODO: add the 'registration' argument also to this method
    pub fn locate_protocol<T: Protocol>(&self) -> Result<&T> {
        let guid_ptr = &T::guid() as *const Guid;
        let registration: *mut Void = ptr::null_mut();
        let mut protocol: *mut T::FfiType = ptr::null_mut();

        let status = unsafe {
            (self.0.LocateProtocol)(guid_ptr, registration, mem::transmute::<&mut *mut T::FfiType, *mut *mut Void>(&mut protocol))
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