use ffi::base::{
    EFI_MAC_ADDRESS,
    UINT8,
    UINT32,
    UINT64,
    UINTN,
    BOOLEAN,
    VOID,
    EFI_STATUS,
    EFI_EVENT,
    EFI_GUID,
    NOT_DEFINED,
};

pub const EFI_SIMPLE_NETWORK_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0xA19832B9, 0xAC25, 0x11D3, [0x9A, 0x2D, 0x00, 0x90, 0x27, 0x3f, 0xc1, 0x4d]);

pub const MAX_MCAST_FILTER_CNT: UINTN = 16;

#[repr(C)]
pub struct EFI_SIMPLE_NETWORK_PROTOCOL {
    Revision: UINT64,
    Start: EFI_SIMPLE_NETWORK_START,
    Stop: EFI_SIMPLE_NETWORK_STOP,
    Initialize: EFI_SIMPLE_NETWORK_INITIALIZE,
    Reset: EFI_SIMPLE_NETWORK_RESET,
    Shutdown: EFI_SIMPLE_NETWORK_SHUTDOWN,
    ReceiveFilters: EFI_SIMPLE_NETWORK_RECEIVE_FILTERS,
    StationAddress: EFI_SIMPLE_NETWORK_STATION_ADDRESS,
    Statistics: EFI_SIMPLE_NETWORK_STATISTICS,
    MCastIpToMac: EFI_SIMPLE_NETWORK_MCAST_IP_TO_MAC,
    NvData: EFI_SIMPLE_NETWORK_NVDATA,
    GetStatus: EFI_SIMPLE_NETWORK_GET_STATUS,
    Transmit: EFI_SIMPLE_NETWORK_TRANSMIT,
    Receive: EFI_SIMPLE_NETWORK_RECEIVE,
    WaitForPacket: EFI_EVENT,
    Mode: *mut EFI_SIMPLE_NETWORK_MODE,
}

#[derive(Debug)]
#[repr(C)]
pub struct EFI_SIMPLE_NETWORK_MODE {
    pub State: UINT32,
    pub HwAddressSize: UINT32,
    pub MediaHeaderSize: UINT32,
    pub MaxPacketSize: UINT32,
    pub NvRamSize: UINT32,
    pub NvRamAccessSize: UINT32,
    pub ReceiveFilterMask: UINT32,
    pub ReceiveFilterSetting: UINT32,
    pub MaxMCastFilterCount: UINT32,
    pub MCastFilterCount: UINT32,
    pub MCastFilter: [EFI_MAC_ADDRESS; MAX_MCAST_FILTER_CNT],
    pub CurrentAddress: EFI_MAC_ADDRESS,
    pub BroadcastAddress: EFI_MAC_ADDRESS,
    pub PermanentAddress: EFI_MAC_ADDRESS,
    pub IfType: UINT8,
    pub MacAddressChangeable: BOOLEAN,
    pub MultipleTxSupported: BOOLEAN,
    pub MediaPresentSupported: BOOLEAN,
    pub MediaPresent: BOOLEAN,
}

pub type EFI_SIMPLE_NETWORK_START = *const NOT_DEFINED;
pub type EFI_SIMPLE_NETWORK_STOP = *const NOT_DEFINED;
pub type EFI_SIMPLE_NETWORK_INITIALIZE = *const NOT_DEFINED;
pub type EFI_SIMPLE_NETWORK_RESET = *const NOT_DEFINED;
pub type EFI_SIMPLE_NETWORK_SHUTDOWN = *const NOT_DEFINED;
pub type EFI_SIMPLE_NETWORK_RECEIVE_FILTERS = *const NOT_DEFINED;
pub type EFI_SIMPLE_NETWORK_STATION_ADDRESS = *const NOT_DEFINED;
pub type EFI_SIMPLE_NETWORK_STATISTICS = *const NOT_DEFINED;
pub type EFI_SIMPLE_NETWORK_MCAST_IP_TO_MAC = *const NOT_DEFINED;
pub type EFI_SIMPLE_NETWORK_NVDATA = *const NOT_DEFINED;
pub type EFI_SIMPLE_NETWORK_RECEIVE = *const NOT_DEFINED;
pub type EFI_SIMPLE_NETWORK_TRANSMIT = *const NOT_DEFINED;


pub type EFI_SIMPLE_NETWORK_GET_STATUS = extern "win64" fn(
    This: *const EFI_SIMPLE_NETWORK_PROTOCOL,
    InterruptStatus: *const UINT32,
    TxBuf: *mut *const VOID
) -> EFI_STATUS;