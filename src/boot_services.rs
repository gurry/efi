use ffi::{boot_services::{
        EFI_BOOT_SERVICES, 
        EFI_INTERFACE_TYPE, 
        EFI_ALLOCATE_TYPE, 
        EFI_MEMORY_TYPE,
        TPL_APPLICATION,
        TPL_CALLBACK,
        TPL_NOTIFY,
        TPL_HIGH_LEVEL,
        EVT_TIMER,
        EVT_RUNTIME,
        EVT_NOTIFY_WAIT,
        EVT_NOTIFY_SIGNAL,
        EVT_SIGNAL_EXIT_BOOT_SERVICES,
        EVT_SIGNAL_VIRTUAL_ADDRESS_CHANGE,
        EFI_EVENT_NOTIFY 
    }, 
    EFI_HANDLE,
    EFI_STATUS,
    EFI_EVENT,
    EFI_SUCCESS,
    UINTN, 
    CHAR16, 
    VOID,
};

use ::{Result, Guid, Void, to_res, utils::Wrapper, Opaque, OpaqueDevice, OpaqueAgent, OpaqueImage, OpaqueController, OpaqueEvent, to_boolean};
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

#[repr(u32)]
#[derive(Debug)]
pub enum EventType {
    Timer = EVT_TIMER,
    Runtime = EVT_RUNTIME,
    NotifyWait = EVT_NOTIFY_WAIT,
    NotifySignal = EVT_NOTIFY_SIGNAL,
    ExitBootServices = EVT_SIGNAL_EXIT_BOOT_SERVICES,
    SignalVirtualAddressChange = EVT_SIGNAL_VIRTUAL_ADDRESS_CHANGE
}

#[repr(usize)]
#[derive(Debug)]
pub enum Tpl {
    Appliction = TPL_APPLICATION,
    Callback = TPL_CALLBACK,
    Notify = TPL_NOTIFY,
    HighLevel = TPL_HIGH_LEVEL
}

#[repr(C)]
pub struct BootServices<'a>(&'a EFI_BOOT_SERVICES);

extern "win64" fn global_notify_func<F: Fn()>(_event: EFI_EVENT, context: *const VOID) -> EFI_STATUS {
    if !context.is_null() {
        let closure: &F = unsafe { mem::transmute(context) };
        closure();
    }

    EFI_SUCCESS
}


// TODO: Regarding lifetimes of protocols and handles this is what the UEFI documentation has to say:

// The caller is responsible for ensuring that there are no references to a protocol interface that 
// has been removed. In some cases, outstanding reference information is not available in the 
// protocol, so the protocol, once added, cannot be removed. Examples include Console I/O, Block I/O, 
// Disk I/O, and (in general) handles to device protocols.
// If the last protocol interface is removed from a handle, the handle is freed and is no longer valid

impl<'a> BootServices<'a> {
    pub fn create_event<F: Fn()>(&mut self, type_: EventType, notify_tpl: Tpl, notify_function: Option<&'a F>) -> Result<&'a OpaqueEvent> {
        let context: *const VOID = notify_function.map_or(ptr::null(), |v| unsafe { mem::transmute(v) } );
        let notify_function: EFI_EVENT_NOTIFY = unsafe { notify_function.map_or(mem::transmute::<*const VOID, EFI_EVENT_NOTIFY>(ptr::null()), |_| global_notify_func::<F>) } ;
        let mut event: EFI_EVENT = ptr::null();

        let status = unsafe {
            (self.0.CreateEvent)(mem::transmute(type_), mem::transmute(notify_tpl), notify_function, context, &mut event)
        };

        to_res(unsafe { mem::transmute(event) }, status)
    }

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