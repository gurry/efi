use ffi::{
    base::{EFI_STATUS, EFI_TIME, EFI_TIME_CAPABILITIES, EFI_TABLE_HEADER, UINTN, NOT_DEFINED},
    EFI_SPECIFICATION_VERSION,
};

pub const EFI_RUNTIME_SERVICES_SIGNATURE: UINTN = 0x56524553544e5552;
pub const EFI_RUNTIME_SERVICES_REVISION: UINTN = EFI_SPECIFICATION_VERSION;

#[repr(C)]
pub struct EFI_RUNTIME_SERVICES {
    pub Hdr: EFI_TABLE_HEADER,

    pub GetTime: EFI_GET_TIME,
    pub SetTime: EFI_SET_TIME,
    pub GetWakeupTime: EFI_GET_WAKEUP_TIME,
    pub SetWakeupTime: EFI_SET_WAKEUP_TIME,

    pub SetVirtualAddressMap: EFI_SET_VIRTUAL_ADDRESS_MAP,
    pub ConvertPointer: EFI_CONVERT_POINTER,

    pub GetVariable: EFI_GET_VARIABLE,
    pub GetNextVariableName: EFI_GET_NEXT_VARIABLE_NAME,
    pub SetVariable: EFI_SET_VARIABLE,

    pub GetNextHighMonotonicCount: EFI_GET_NEXT_HIGH_MONO_COUNT,
    pub ResetSystem: EFI_RESET_SYSTEM,

    pub UpdateCapsule: EFI_UPDATE_CAPSULE,
    pub QueryCapsuleCapabilities: EFI_QUERY_CAPSULE_CAPABILITIES,

    pub QueryVariableInfo: EFI_QUERY_VARIABLE_INFO,
}

pub type EFI_RAISE_TPL = *const NOT_DEFINED;
pub type EFI_SET_TIME = *const NOT_DEFINED;
pub type EFI_GET_WAKEUP_TIME = *const NOT_DEFINED;
pub type EFI_SET_WAKEUP_TIME = *const NOT_DEFINED;
pub type EFI_SET_VIRTUAL_ADDRESS_MAP = *const NOT_DEFINED;
pub type EFI_CONVERT_POINTER = *const NOT_DEFINED;
pub type EFI_GET_VARIABLE = *const NOT_DEFINED;
pub type EFI_GET_NEXT_VARIABLE_NAME = *const NOT_DEFINED;
pub type EFI_SET_VARIABLE = *const NOT_DEFINED;
pub type EFI_GET_NEXT_HIGH_MONO_COUNT = *const NOT_DEFINED;
pub type EFI_RESET_SYSTEM = *const NOT_DEFINED;
pub type EFI_UPDATE_CAPSULE = *const NOT_DEFINED;
pub type EFI_QUERY_CAPSULE_CAPABILITIES = *const NOT_DEFINED;
pub type EFI_QUERY_VARIABLE_INFO = *const NOT_DEFINED;

pub type EFI_GET_TIME = extern "win64" fn(
    Time: *mut EFI_TIME,
    Capabilities: *mut EFI_TIME_CAPABILITIES
) -> EFI_STATUS;