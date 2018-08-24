
use ffi::base::{
    EFI_IPv4_ADDRESS, 
    EFI_GUID,
    EFI_STATUS,
    EFI_EVENT,
    TRUE,
    FALSE,
    UINT8,
    UINT32,
    UINTN,
    BOOLEAN,
};

use core::ptr;

pub const EFI_IP4_SERVICE_BINDING_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0xc51711e7, 0xb4bf, 0x404a, [0xbf, 0xb8, 0x0a, 0x04, 0x8e, 0xf1, 0xff, 0xe4]);
pub const EFI_IP4_PROTOCOL_GUID : EFI_GUID = EFI_GUID(0x41d94cd2, 0x35b6, 0x455a, [0x82, 0x58, 0xd4, 0xe5, 0x13, 0x34, 0xaa, 0xdd]);
pub const EFI_IP4_CONFIG_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0x3b95aa31, 0x3793, 0x434b, [0x86, 0x67, 0xc8, 0x07, 0x08, 0x92, 0xe0, 0x5e]);

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
#[derive(Debug, Clone)]
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

#[derive(Debug)]
#[repr(C)]
pub struct EFI_IP4_IPCONFIG_DATA {
    pub StationAddress: EFI_IPv4_ADDRESS,
    pub SubnetMask: EFI_IPv4_ADDRESS,
    pub RouteTableSize: UINT32,
    pub RouteTable: *const EFI_IP4_ROUTE_TABLE,
}

impl Default for EFI_IP4_IPCONFIG_DATA  {
    fn default() -> Self {
        Self {
            StationAddress: EFI_IPv4_ADDRESS::zero(),
            SubnetMask: EFI_IPv4_ADDRESS::zero(),
            RouteTableSize: 0,
            RouteTable:  ptr::null(),
        }
    }
}

#[repr(C)]
pub struct EFI_IP4_CONFIG_PROTOCOL {
    pub Start: EFI_IP4_CONFIG_START,
    pub Stop: EFI_IP4_CONFIG_STOP,
    pub GetData: EFI_IP4_CONFIG_GET_DATA,
}

pub type EFI_IP4_CONFIG_START = extern "win64" fn(
    This: *const EFI_IP4_CONFIG_PROTOCOL,
    DoneEvent: EFI_EVENT,
    ReconfigEvent: EFI_EVENT
) -> EFI_STATUS;

pub type EFI_IP4_CONFIG_STOP = extern "win64" fn(
    This: *const EFI_IP4_CONFIG_PROTOCOL
) -> EFI_STATUS;

pub type EFI_IP4_CONFIG_GET_DATA = extern "win64" fn(
    This: *const EFI_IP4_CONFIG_PROTOCOL,
    IpConfigDataSize: *mut UINTN,
    IpConfigData: *mut EFI_IP4_IPCONFIG_DATA,
) -> EFI_STATUS;