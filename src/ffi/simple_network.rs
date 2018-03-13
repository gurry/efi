use ffi::base::{
    EFI_MAC_ADDRESS,
    UINT8,
    UINT32,
    UINTN,
    BOOLEAN,
};

pub const MAX_MCAST_FILTER_CNT: UINTN = 16;

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
