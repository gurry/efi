use ffi::{
    base::{EFI_GUID, EFI_HANDLE, EFI_STATUS, EFI_TABLE_HEADER, UINT32, UINTN, CHAR16, BOOLEAN, VOID, NOT_DEFINED},
    device_path::EFI_DEVICE_PATH_PROTOCOL
};
// use base::{Event, Handle, Handles, MemoryType, Status};
// use guid;
// use table;


#[repr(C)]
pub struct EFI_BOOT_SERVICES {
  pub Hdr: EFI_TABLE_HEADER,

  pub RaiseTPL: EFI_RAISE_TPL,
  pub RestoreTPL: EFI_RESTORE_TPL,

  pub AllocatePages: EFI_ALLOCATE_PAGES,
  pub FreePages: EFI_FREE_PAGES,
  pub GetMemoryMap: EFI_GET_MEMORY_MAP,
  pub AllocatePool: EFI_ALLOCATE_POOL,
  pub FreePool: EFI_FREE_POOL,

  pub CreateEvent: EFI_CREATE_EVENT,
  pub SetTimer: EFI_SET_TIMER,
  pub WaitForEvent: EFI_WAIT_FOR_EVENT,
  pub SignalEvent: EFI_SIGNAL_EVENT,
  pub CloseEvent: EFI_CLOSE_EVENT,
  pub CheckEvent: EFI_CHECK_EVENT,

  pub InstallProtocolInterface: EFI_INSTALL_PROTOCOL_INTERFACE,
  pub ReinstallProtocolInterface: EFI_REINSTALL_PROTOCOL_INTERFACE,
  pub UninstallProtocolInterface: EFI_UNINSTALL_PROTOCOL_INTERFACE,
  pub HandleProtocol: EFI_HANDLE_PROTOCOL,
  pub Reserve: *const VOID,
  pub RegisterProtocolNotify: EFI_REGISTER_PROTOCOL_NOTIFY,
  pub LocateHandle: EFI_LOCATE_HANDLE,
  pub LocateDevicePath: EFI_LOCATE_DEVICE_PATH,
  pub InstallConfigurationTable: EFI_INSTALL_CONFIGURATION_TABLE,

  pub LoadImage: EFI_IMAGE_LOAD,
  pub StartImage: EFI_IMAGE_START,
  pub Exit: EFI_EXIT,
  pub UnloadImage: EFI_IMAGE_UNLOAD,
  pub ExitBootServices: EFI_EXIT_BOOT_SERVICES,

  pub GetNextMonotonicCount: EFI_GET_NEXT_MONOTONIC_COUNT,
  pub Stall: EFI_STALL,
  pub SetWatchdogTimer: EFI_SET_WATCHDOG_TIMER,

  pub ConnectController: EFI_CONNECT_CONTROLLER,
  pub DisconnectController: EFI_DISCONNECT_CONTROLLER,

  pub OpenProtocol: EFI_OPEN_PROTOCOL,
  pub CloseProtocol: EFI_CLOSE_PROTOCOL,
  pub OpenProtocolInformation: EFI_OPEN_PROTOCOL_INFORMATION,

  pub ProtocolsPerHandle: EFI_PROTOCOLS_PER_HANDLE,
  pub LocateHandleBuffer: EFI_LOCATE_HANDLE_BUFFER,
  pub LocateProtocol: EFI_LOCATE_PROTOCOL,
  pub InstallMultipleProtocolInterfaces: EFI_INSTALL_MULTIPLE_PROTOCOL_INTERFACES,
  pub UninstallMultipleProtocolInterfaces: EFI_UNINSTALL_MULTIPLE_PROTOCOL_INTERFACES,

  pub CalculateCrc32: EFI_CALCULATE_CRC32,

  pub CopyMem: EFI_COPY_MEM,
  pub SetMem: EFI_SET_MEM,
  pub CreateEventEx: EFI_CREATE_EVENT_EX,
}

// The below are methods currently not defined
pub type EFI_RAISE_TPL = *const NOT_DEFINED;
pub type EFI_RESTORE_TPL = *const NOT_DEFINED;
pub type EFI_ALLOCATE_PAGES = *const NOT_DEFINED;
pub type EFI_FREE_PAGES = *const NOT_DEFINED;
pub type EFI_GET_MEMORY_MAP = *const NOT_DEFINED;
pub type EFI_ALLOCATE_POOL = *const NOT_DEFINED;
pub type EFI_FREE_POOL = *const NOT_DEFINED;
pub type EFI_CREATE_EVENT = *const NOT_DEFINED;
pub type EFI_SET_TIMER = *const NOT_DEFINED;
pub type EFI_WAIT_FOR_EVENT = *const NOT_DEFINED;
pub type EFI_SIGNAL_EVENT = *const NOT_DEFINED;
pub type EFI_CLOSE_EVENT = *const NOT_DEFINED;
pub type EFI_CHECK_EVENT = *const NOT_DEFINED;
pub type EFI_REINSTALL_PROTOCOL_INTERFACE = *const NOT_DEFINED;
pub type EFI_UNINSTALL_PROTOCOL_INTERFACE = *const NOT_DEFINED;
pub type EFI_HANDLE_PROTOCOL = *const NOT_DEFINED;
pub type EFI_REGISTER_PROTOCOL_NOTIFY = *const NOT_DEFINED;
pub type EFI_LOCATE_HANDLE = *const NOT_DEFINED;
pub type EFI_LOCATE_DEVICE_PATH = *const NOT_DEFINED;
pub type EFI_INSTALL_CONFIGURATION_TABLE = *const NOT_DEFINED;
pub type EFI_EXIT = *const NOT_DEFINED;
pub type EFI_IMAGE_UNLOAD = *const NOT_DEFINED;
pub type EFI_EXIT_BOOT_SERVICES = *const NOT_DEFINED;
pub type EFI_GET_NEXT_MONOTONIC_COUNT = *const NOT_DEFINED;
pub type EFI_STALL = *const NOT_DEFINED;
pub type EFI_SET_WATCHDOG_TIMER = *const NOT_DEFINED;
pub type EFI_CONNECT_CONTROLLER = *const NOT_DEFINED;
pub type EFI_DISCONNECT_CONTROLLER = *const NOT_DEFINED;
pub type EFI_OPEN_PROTOCOL_INFORMATION = *const NOT_DEFINED;
pub type EFI_PROTOCOLS_PER_HANDLE = *const NOT_DEFINED;
pub type EFI_LOCATE_HANDLE_BUFFER = *const NOT_DEFINED;
pub type EFI_INSTALL_MULTIPLE_PROTOCOL_INTERFACES = *const NOT_DEFINED;
pub type EFI_UNINSTALL_MULTIPLE_PROTOCOL_INTERFACES = *const NOT_DEFINED;
pub type EFI_CALCULATE_CRC32 = *const NOT_DEFINED;
pub type EFI_COPY_MEM = *const NOT_DEFINED;
pub type EFI_SET_MEM = *const NOT_DEFINED;
pub type EFI_CREATE_EVENT_EX = *const NOT_DEFINED;



#[derive(Debug)]
pub enum EFI_INTERFACE_TYPE {
    EFI_NATIVE_INTERFACE = 0
}

pub type EFI_INSTALL_PROTOCOL_INTERFACE = extern "win64" fn(
    Handle: EFI_HANDLE,
    Protocol: *const EFI_GUID,
    InterfaceType: EFI_INTERFACE_TYPE,
    Interface: *const VOID
) -> EFI_STATUS;

pub const EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL: UINT32 = 0x00000001;
pub const EFI_OPEN_PROTOCOL_GET_PROTOCOL: UINT32 = 0x00000002;
pub const EFI_OPEN_PROTOCOL_TEST_PROTOCOL: UINT32 = 0x00000004;
pub const EFI_OPEN_PROTOCOL_BY_CHILD_CONTROLLER: UINT32 = 0x00000008;
pub const EFI_OPEN_PROTOCOL_BY_DRIVER: UINT32 = 0x00000010;
pub const EFI_OPEN_PROTOCOL_EXCLUSIVE: UINT32 = 0x00000020;

pub type EFI_OPEN_PROTOCOL =  extern "win64" fn(
    Handle: EFI_HANDLE,
    Protocol: *const EFI_GUID,
    Interface: *mut *mut VOID,
    AgentHandle: EFI_HANDLE,
    ControllerHandle: EFI_HANDLE,
    Attributes: UINT32
) -> EFI_STATUS;

pub type EFI_CLOSE_PROTOCOL = extern "win64" fn(
  Handle: EFI_HANDLE,
  Protocol: *const EFI_GUID,
  AgentHandle: EFI_HANDLE,
  ControllerHandle: EFI_HANDLE
) -> EFI_STATUS;

pub type EFI_LOCATE_PROTOCOL = extern "win64" fn(
    Protocol: *const EFI_GUID,
    Registration: *mut VOID,
    Interface: *mut *mut VOID
) -> EFI_STATUS;

pub type EFI_IMAGE_LOAD = extern "win64" fn(
    BootPolicy: BOOLEAN,
    ParentImageHandle: EFI_HANDLE,
    DevicePath: *const EFI_DEVICE_PATH_PROTOCOL,
    SourceBuffer: *const VOID,
    SourceSize: UINTN,
    ImageHandle: *mut EFI_HANDLE
) -> EFI_STATUS;

pub type EFI_IMAGE_START = extern "win64" fn(
    ImageHandle: EFI_HANDLE,
    ExitDataSize: *mut UINTN,
    ExitData: *mut *const CHAR16
) -> EFI_STATUS;
