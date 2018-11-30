use ffi::base::{
    EFI_MAC_ADDRESS,
    UINT8,
    UINT32,
    UINT64,
    UINTN,
    BOOLEAN,
    FALSE,
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
    pub Revision: UINT64,
    pub Start: EFI_SIMPLE_NETWORK_START,
    pub Stop: EFI_SIMPLE_NETWORK_STOP,
    pub Initialize: EFI_SIMPLE_NETWORK_INITIALIZE,
    pub Reset: EFI_SIMPLE_NETWORK_RESET,
    pub Shutdown: EFI_SIMPLE_NETWORK_SHUTDOWN,
    pub ReceiveFilters: EFI_SIMPLE_NETWORK_RECEIVE_FILTERS,
    pub StationAddress: EFI_SIMPLE_NETWORK_STATION_ADDRESS,
    pub Statistics: EFI_SIMPLE_NETWORK_STATISTICS,
    pub MCastIpToMac: EFI_SIMPLE_NETWORK_MCAST_IP_TO_MAC,
    pub NvData: EFI_SIMPLE_NETWORK_NVDATA,
    pub GetStatus: EFI_SIMPLE_NETWORK_GET_STATUS,
    pub Transmit: EFI_SIMPLE_NETWORK_TRANSMIT,
    pub Receive: EFI_SIMPLE_NETWORK_RECEIVE,
    pub WaitForPacket: EFI_EVENT,
    pub Mode: *mut EFI_SIMPLE_NETWORK_MODE,
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

impl Default for EFI_SIMPLE_NETWORK_MODE  {
    fn default() -> Self {
        Self {
            State: 0,
            HwAddressSize: 0,
            MediaHeaderSize: 0,
            MaxPacketSize: 0,
            NvRamSize: 0,
            NvRamAccessSize: 0,
            ReceiveFilterMask: 0,
            ReceiveFilterSetting: 0,
            MaxMCastFilterCount: 0,
            MCastFilterCount: 0,
            MCastFilter: Default::default(),
            CurrentAddress: EFI_MAC_ADDRESS::default(),
            BroadcastAddress: EFI_MAC_ADDRESS::default(),
            PermanentAddress: EFI_MAC_ADDRESS::default(),
            IfType: 0,
            MacAddressChangeable: FALSE,
            MultipleTxSupported: FALSE,
            MediaPresentSupported: FALSE,
            MediaPresent: FALSE,
        }
    }
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
    InterruptStatus: *mut UINT32,
    TxBuf: *mut *const VOID
) -> EFI_STATUS;