
use ffi::base::{
    EFI_IPv4_ADDRESS, 
    TRUE,
    FALSE,
    UINT8,
    UINT32,
    BOOLEAN,
};

use core::ptr;

#[derive(Debug)]
#[repr(C)]
pub struct EFI_IP4_MODE_DATA {
    pub IsStarted: BOOLEAN,
    pub MaxPacketSize: UINT32,
    pub ConfigData: EFI_IP4_CONFIG_DATA,
    pub IsConfigured: BOOLEAN,
    pub GroupCount: UINT32,
    pub GroupTable: *const EFI_IPv4_ADDRESS,
    pub RouteCount: UINT32,
    pub RouteTable: *const EFI_IP4_ROUTE_TABLE,
    pub IcmpTypeCount: UINT32,
    pub IcmpTypeList: *const EFI_IP4_ICMP_TYPE,
}

impl EFI_IP4_MODE_DATA {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for EFI_IP4_MODE_DATA {
    fn default() -> Self {
        Self {
            IsStarted: FALSE,
            MaxPacketSize: 1024,
            ConfigData: EFI_IP4_CONFIG_DATA::default(),
            IsConfigured: FALSE,
            GroupCount: 0,
            GroupTable: ptr::null(),
            RouteCount: 0,
            RouteTable: ptr::null(),
            IcmpTypeCount: 0,
            IcmpTypeList: ptr::null(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct EFI_IP4_CONFIG_DATA {
    pub DefaultProtocol: UINT8,
    pub AcceptAnyProtocol: BOOLEAN,
    pub AcceptIcmpErrors: BOOLEAN,
    pub AcceptBroadcast: BOOLEAN,
    pub AcceptPromiscuous: BOOLEAN,
    pub UseDefaultAddress: BOOLEAN,
    pub StationAddress: EFI_IPv4_ADDRESS,
    pub SubnetMask: EFI_IPv4_ADDRESS,
    pub TypeOfService: UINT8,
    pub TimeToLive: UINT8,
    pub DoNotFragment: BOOLEAN,
    pub RawData: BOOLEAN,
    pub ReceiveTimeout: UINT32,
    pub TransmitTimeout: UINT32,
} 

impl Default for EFI_IP4_CONFIG_DATA {
    fn default() -> Self {
        Self {
            DefaultProtocol: 6, // 6 stands for TCP
            AcceptAnyProtocol: FALSE,
            AcceptIcmpErrors: TRUE,
            AcceptBroadcast: TRUE,
            AcceptPromiscuous: FALSE,
            UseDefaultAddress: TRUE,
            StationAddress: EFI_IPv4_ADDRESS::zero(),
            SubnetMask: EFI_IPv4_ADDRESS::zero(),
            TypeOfService: 0,
            TimeToLive: 0,
            DoNotFragment: FALSE,
            RawData: FALSE,
            ReceiveTimeout: 0,
            TransmitTimeout: 0,
        }
    }
}
#[derive(Debug)]
#[repr(C)]
pub struct EFI_IP4_ROUTE_TABLE {
    pub SubnetAddress: EFI_IPv4_ADDRESS,
    pub SubnetMask: EFI_IPv4_ADDRESS,
    pub GatewayAddress: EFI_IPv4_ADDRESS,
}

#[derive(Debug)]
#[repr(C)]
pub struct EFI_IP4_ICMP_TYPE {
    pub Type: UINT8,
    pub Code: UINT8,
}