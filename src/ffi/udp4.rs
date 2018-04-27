use ffi::{
    base::{
        EFI_IPv4_ADDRESS,
        EFI_STATUS,
        EFI_EVENT,
        EFI_GUID,
        EFI_SUCCESS,
        UINT8,
        UINT16,
        UINT32,
        UINTN,
        BOOLEAN,
        VOID,
        EFI_TIME
    },
    managed_network::EFI_MANAGED_NETWORK_CONFIG_DATA,
    simple_network::EFI_SIMPLE_NETWORK_MODE,
    ip4::EFI_IP4_MODE_DATA,
};
use core::{mem, ptr};

pub const EFI_UDP4_SERVICE_BINDING_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0x83f01464, 0x99bd, 0x45e5, [0xb3, 0x83, 0xaf, 0x63, 0x05, 0xd8, 0xe9, 0xe6]);

pub const EFI_UDP4_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0x3ad9df29, 0x4501, 0x478d, [0xb1, 0xf8, 0x7f, 0x7f, 0xe7, 0x0e, 0x50, 0xf3]);

#[repr(C)]
pub struct EFI_UDP4_PROTOCOL {
    pub GetModeData: EFI_UDP4_GET_MODE_DATA,
    pub Configure: EFI_UDP4_CONFIGURE,
    pub Groups: EFI_UDP4_GROUPS,
    pub Routes: EFI_UDP4_ROUTES,
    pub Transmit: EFI_UDP4_TRANSMIT,
    pub Receive: EFI_UDP4_RECEIVE,
    pub Cancel: EFI_UDP4_CANCEL,
    pub Poll: EFI_UDP4_POLL,
}

pub type EFI_UDP4_GET_MODE_DATA = extern "win64" fn(
    This: *const EFI_UDP4_PROTOCOL,
    Udp4ConfigData: *mut EFI_UDP4_CONFIG_DATA,
    Ip4ModeData: *mut EFI_IP4_MODE_DATA,
    MnpConfigData: *mut EFI_MANAGED_NETWORK_CONFIG_DATA,
    SnpModeData: *mut EFI_SIMPLE_NETWORK_MODE,
) -> EFI_STATUS;

#[repr(C)]
pub struct EFI_UDP4_CONFIG_DATA {
    //Receiving Filters
    pub AcceptBroadcast: BOOLEAN,
    pub AcceptPromiscuous: BOOLEAN,
    pub AcceptAnyPort: BOOLEAN,
    pub AllowDuplicatePort: BOOLEAN,
    // I/O parameters
    pub TypeOfService: UINT8,
    pub TimeToLive: UINT8,
    pub DoNotFragment: BOOLEAN,
    pub ReceiveTimeout: UINT32,
    pub TransmitTimeout: UINT32,
    // Access Point
    pub UseDefaultAddress: BOOLEAN,
    pub StationAddress: EFI_IPv4_ADDRESS,
    pub SubnetMask: EFI_IPv4_ADDRESS,
    pub StationPort: UINT16,
    pub RemoteAddress: EFI_IPv4_ADDRESS,
    pub RemotePort: UINT16,
}

pub type EFI_UDP4_CONFIGURE = extern "win64" fn(
    This: *const EFI_UDP4_PROTOCOL,
    UdpConfigData: *const EFI_UDP4_CONFIG_DATA,
) -> EFI_STATUS;

pub type EFI_UDP4_GROUPS = extern "win64" fn(
    This: *const EFI_UDP4_PROTOCOL,
    JoinFlag: BOOLEAN,
    MulticastAddress: *const EFI_IPv4_ADDRESS,
) -> EFI_STATUS;

pub type EFI_UDP4_ROUTES = extern "win64" fn(
    This: *const EFI_UDP4_PROTOCOL,
    DeleteRoute: BOOLEAN,
    SubnetAddress: *const EFI_IPv4_ADDRESS,
    SubnetMask: *const EFI_IPv4_ADDRESS,
    GatewayAddress: *const EFI_IPv4_ADDRESS
) -> EFI_STATUS;

pub type EFI_UDP4_TRANSMIT = extern "win64" fn(
    This: *const EFI_UDP4_PROTOCOL,
    Token: *const EFI_UDP4_COMPLETION_TOKEN,
) -> EFI_STATUS;

#[repr(C)]
pub struct EFI_UDP4_COMPLETION_TOKEN {
    pub Event: EFI_EVENT,
    pub Status: EFI_STATUS,
    pub Packet: PacketUnion,
}

impl Default for EFI_UDP4_COMPLETION_TOKEN  {
    fn default() -> Self {
        Self {
            Event: ptr::null() as EFI_EVENT,
            Status: EFI_SUCCESS,
            Packet: PacketUnion { TxData: ptr::null() as *const EFI_UDP4_TRANSMIT_DATA }
         }
    }
}

#[repr(C)]
pub union PacketUnion {
    pub RxData: *const EFI_UDP4_RECEIVE_DATA,
    pub TxData: *const EFI_UDP4_TRANSMIT_DATA,
}

pub const EFI_NETWORK_UNREACHABLE: UINTN = with_high_bit_set!(100);
pub const EFI_HOST_UNREACHABLE: UINTN = with_high_bit_set!(101) ;
pub const EFI_PROTOCOL_UNREACHABLE: UINTN = with_high_bit_set!(102);
pub const EFI_PORT_UNREACHABLE: UINTN = with_high_bit_set!(103);

#[repr(C)]
pub struct EFI_UDP4_RECEIVE_DATA {
    pub TimeStamp: EFI_TIME,
    pub RecycleSignal: EFI_EVENT,
    pub UdpSession: EFI_UDP4_SESSION_DATA,
    pub DataLength: UINT32,
    pub FragmentCount: UINT32,
    pub FragmentTable: [EFI_UDP4_FRAGMENT_DATA; 1],
}

#[repr(C)]
pub struct EFI_UDP4_SESSION_DATA {
    pub SourceAddress: EFI_IPv4_ADDRESS,
    pub SourcePort: UINT16,
    pub DestinationAddress: EFI_IPv4_ADDRESS,
    pub DestinationPort: UINT16,
}

#[repr(C)]
pub struct EFI_UDP4_FRAGMENT_DATA {
    pub FragmentLength: UINT32,
    pub FragmentBuffer: *const VOID,
}

#[repr(C)]
pub struct EFI_UDP4_TRANSMIT_DATA {
    pub UdpSessionData: *const EFI_UDP4_SESSION_DATA,
    pub GatewayAddress: *const EFI_IPv4_ADDRESS,
    pub DataLength: UINT32,
    pub FragmentCount: UINT32,
    pub FragmentTable: [EFI_UDP4_FRAGMENT_DATA; 1],
}

pub type EFI_UDP4_RECEIVE = extern "win64" fn(
    This: *const EFI_UDP4_PROTOCOL,
    Token: *const EFI_UDP4_COMPLETION_TOKEN,
) -> EFI_STATUS;

pub type EFI_UDP4_CANCEL = extern "win64" fn(
    This: *const EFI_UDP4_PROTOCOL,
    Token: *const EFI_UDP4_COMPLETION_TOKEN,
) -> EFI_STATUS;

pub type EFI_UDP4_POLL = extern "win64" fn(
    This: *const EFI_UDP4_PROTOCOL,
) -> EFI_STATUS;