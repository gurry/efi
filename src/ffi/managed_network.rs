use ffi::base::{
    UINT16,
    UINT32,
    BOOLEAN,
};

#[derive(Debug)]
#[repr(C)]
pub struct EFI_MANAGED_NETWORK_CONFIG_DATA {
    pub ReceivedQueueTimeoutValue: UINT32,
    pub TransmitQueueTimeoutValue: UINT32,
    pub ProtocolTypeFilter: UINT16,
    pub EnableUnicastReceive: BOOLEAN,
    pub EnableMulticastReceive: BOOLEAN,
    pub EnableBroadcastReceive: BOOLEAN,
    pub EnablePromiscuousReceive: BOOLEAN,
    pub FlushQueuesOnReset: BOOLEAN,
    pub EnableReceiveTimestamps: BOOLEAN,
    pub DisableBackgroundPolling: BOOLEAN,
}
