
use ffi::{
    managed_network::EFI_MANAGED_NETWORK_CONFIG_DATA,
    simple_network::EFI_SIMPLE_NETWORK_MODE,
    base::{
        EFI_IPv4_ADDRESS, 
        EFI_GUID,
        EFI_STATUS,
        EFI_EVENT,
        EFI_SUCCESS,
        EFI_TIME,
        VOID,
        TRUE,
        FALSE,
        UINT8,
        UINT16,
        UINT32,
        UINTN,
        BOOLEAN,
    },
};

use core::ptr;

pub const EFI_IP4_SERVICE_BINDING_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0xc51711e7, 0xb4bf, 0x404a, [0xbf, 0xb8, 0x0a, 0x04, 0x8e, 0xf1, 0xff, 0xe4]);
pub const EFI_IP4_PROTOCOL_GUID : EFI_GUID = EFI_GUID(0x41d94cd2, 0x35b6, 0x455a, [0x82, 0x58, 0xd4, 0xe5, 0x13, 0x34, 0xaa, 0xdd]);
pub const EFI_IP4_CONFIG_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0x3b95aa31, 0x3793, 0x434b, [0x86, 0x67, 0xc8, 0x07, 0x08, 0x92, 0xe0, 0x5e]);

pub struct EFI_IP4_PROTOCOL {
    pub GetModeData: EFI_IP4_GET_MODE_DATA,
    pub Configure: EFI_IP4_CONFIGURE,
    pub Groups: EFI_IP4_GROUPS,
    pub Routes: EFI_IP4_ROUTES,
    pub Transmit: EFI_IP4_TRANSMIT,
    pub Receive: EFI_IP4_RECEIVE,
    pub Cancel: EFI_IP4_CANCEL,
    pub Poll: EFI_IP4_POLL,
}

pub type EFI_IP4_GET_MODE_DATA = extern "win64" fn(
    This: *const EFI_IP4_PROTOCOL,
    Ip4ModeData: *mut EFI_IP4_MODE_DATA,
    MnpConfigData: *mut EFI_MANAGED_NETWORK_CONFIG_DATA,
    SnpModeData: *mut EFI_SIMPLE_NETWORK_MODE
) -> EFI_STATUS;

pub type EFI_IP4_CONFIGURE = extern "win64" fn(
    This: *const EFI_IP4_PROTOCOL,
    IpConfigData: *const EFI_IP4_CONFIG_DATA
) -> EFI_STATUS;

pub type EFI_IP4_GROUPS = extern "win64" fn(
    This: *const EFI_IP4_PROTOCOL,
    JoinFlag: BOOLEAN,
    GroupAddress: *const EFI_IPv4_ADDRESS
) -> EFI_STATUS;

pub type EFI_IP4_ROUTES = extern "win64" fn(
    This: *const EFI_IP4_PROTOCOL,
    DeleteRoute: BOOLEAN,
    SubnetAddress: *const EFI_IPv4_ADDRESS,
    SubnetMask: *const EFI_IPv4_ADDRESS,
    GatewayAddres: *const EFI_IPv4_ADDRESS
) -> EFI_STATUS;

pub type EFI_IP4_TRANSMIT = extern "win64" fn(
    This: *const EFI_IP4_PROTOCOL,
    Token: *const EFI_IP4_COMPLETION_TOKEN
) -> EFI_STATUS;

pub type EFI_IP4_RECEIVE = extern "win64" fn(
    This: *const EFI_IP4_PROTOCOL,
    Token: *const EFI_IP4_COMPLETION_TOKEN
) -> EFI_STATUS;

pub type EFI_IP4_CANCEL = extern "win64" fn(
    This: *const EFI_IP4_PROTOCOL,
    Token: *const EFI_IP4_COMPLETION_TOKEN
) -> EFI_STATUS;

pub type EFI_IP4_POLL = extern "win64" fn(
    This: *const EFI_IP4_PROTOCOL
) -> EFI_STATUS;

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

#[repr(C)]
pub struct EFI_IP4_COMPLETION_TOKEN {
    pub Event: EFI_EVENT,
    pub Status: EFI_STATUS,
    pub Packet: PacketUnion,
}

impl Default for EFI_IP4_COMPLETION_TOKEN {
    fn default() -> Self {
        Self {
            Event: ptr::null() as EFI_EVENT,
            Status: EFI_SUCCESS,
            Packet: PacketUnion { TxData: ptr::null() as *const EFI_IP4_TRANSMIT_DATA }
         }
    }
}

#[repr(C)]
pub union PacketUnion {
    pub RxData: *const EFI_IP4_RECEIVE_DATA,
    pub TxData: *const EFI_IP4_TRANSMIT_DATA,
}

#[derive(Debug)]
#[repr(C)]
pub struct EFI_IP4_RECEIVE_DATA {
    pub TimeStamp: EFI_TIME,
    pub RecycleSignal: EFI_EVENT,
    pub Header: *const EFI_IP4_HEADER,
    pub OptionsLength: UINT32,
    pub Options: *const VOID,
    pub DataLength: UINT32,
    pub FragmentCount: UINT32,
    pub FragmentTable: [EFI_IP4_FRAGMENT_DATA; 1],
}

 // Be careful with packed. Borrowing or writing to its fields could be UB. 
 // See these issues: https://github.com/rust-lang/rust/issues/46043, https://github.com/rust-lang/rust/issues/27060
 // When creating this struct, better to perhaps write directly to a raw buffer and then transmute into it
 // instead of assigning to individual fields. This is a protocol header anyway. So it should be kinda idiomatic to do raw writes like this.
#[repr(packed)]
pub struct EFI_IP4_HEADER {
    pub HeaderLengthAndVersion: UINT8, // This is actually two bit fields called "HeaderLength" and "Version" with 4 bits each. Rust doesn't support bitfields (yet)
    pub TypeOfService: UINT8,
    pub TotalLength: UINT16,
    pub Identification: UINT16,
    pub Fragmentation: UINT16,
    pub TimeToLive: UINT8,
    pub Protocol: UINT8,
    pub Checksum: UINT16,
    pub SourceAddress: EFI_IPv4_ADDRESS,
    pub DestinationAddress: EFI_IPv4_ADDRESS,
}

#[derive(Debug)]
#[repr(C)]
pub struct EFI_IP4_FRAGMENT_DATA {
    pub FragmentLength: UINT32,
    pub FragmentBuffer: *const VOID,
}

#[derive(Debug)]
#[repr(C)]
pub struct EFI_IP4_TRANSMIT_DATA {
    pub DestinationAddress: EFI_IPv4_ADDRESS,
    pub OverrideData: *const EFI_IP4_OVERRIDE_DATA,
    pub OptionsLength: UINT32,
    pub OptionsBuffer: *const VOID,
    pub TotalDataLength: UINT32,
    pub FragmentCount: UINT32,
    pub FragmentTable: [EFI_IP4_FRAGMENT_DATA; 1],
}

#[derive(Debug)]
#[repr(C)]
pub struct EFI_IP4_OVERRIDE_DATA {
    pub SourceAddress: EFI_IPv4_ADDRESS,
    pub GatewayAddress: EFI_IPv4_ADDRESS,
    pub Protocol: UINT8,
    pub TypeOfService: UINT8,
    pub TimeToLive: UINT8,
    pub DoNotFragment: BOOLEAN,
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