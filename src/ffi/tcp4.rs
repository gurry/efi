use ffi::{
    base::{
        EFI_IPv4_ADDRESS, 
        EFI_STATUS, 
        EFI_HANDLE, 
        EFI_EVENT,
        EFI_GUID,
        UINT8,
        UINT16,
        UINT32,
        BOOLEAN,
        VOID
    },
    managed_network::EFI_MANAGED_NETWORK_CONFIG_DATA,
    simple_network::EFI_SIMPLE_NETWORK_MODE,
    ip4::EFI_IP4_MODE_DATA,
};

pub const EFI_TCP4_SERVICE_BINDING_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0x00720665, 0x67EB, 0x4a99, [0xBA, 0xF7, 0xD3, 0xC3, 0x3A, 0x1C, 0x7C, 0xC9]);

pub const EFI_TCP4_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0x65530BC7, 0xA359, 0x410f, [0xB0, 0x10, 0x5A, 0xAD, 0xC7, 0xEC, 0x2B, 0x62]);

#[repr(C)]
pub struct EFI_TCP4_PROTOCOL {
    pub GetModeData: EFI_TCP4_GET_MODE_DATA,
    pub Configure: EFI_TCP4_CONFIGURE,
    pub Routes: EFI_TCP4_ROUTES,
    pub Connect: EFI_TCP4_CONNECT,
    pub Accept: EFI_TCP4_ACCEPT,
    pub Transmit: EFI_TCP4_TRANSMIT,
    pub Receive: EFI_TCP4_RECEIVE,
    pub Close: EFI_TCP4_CLOSE,
    pub Cancel: EFI_TCP4_CANCEL,
    pub Poll: EFI_TCP4_POLL,
}


pub type EFI_TCP4_GET_MODE_DATA = extern "win64" fn(
    This: *const EFI_TCP4_PROTOCOL,
    Tcp4State: *mut EFI_TCP4_CONNECTION_STATE,
    Tcp4ConfigData: *mut EFI_TCP4_CONFIG_DATA,
    Ip4ModeData: *mut EFI_IP4_MODE_DATA,
    MnpConfigData: *mut EFI_MANAGED_NETWORK_CONFIG_DATA,
    SnpModeData: *mut EFI_SIMPLE_NETWORK_MODE 
) -> EFI_STATUS;

#[repr(C)]
pub struct EFI_TCP4_ACCESS_POINT {
    pub UseDefaultAddress: BOOLEAN,
    pub StationAddress: EFI_IPv4_ADDRESS,
    pub SubnetMask: EFI_IPv4_ADDRESS,
    pub StationPort: UINT16,
    pub RemoteAddress: EFI_IPv4_ADDRESS,
    pub RemotePort: UINT16,
    pub ActiveFlag: BOOLEAN,
}

#[repr(C)]
pub struct EFI_TCP4_OPTION {
    pub ReceiveBufferSize: UINT32,
    pub SendBufferSize: UINT32,
    pub MaxSynBackLog: UINT32,
    pub ConnectionTimeout: UINT32,
    pub DataRetries: UINT32,
    pub FinTimeout: UINT32,
    pub TimeWaitTimeout: UINT32,
    pub KeepAliveProbes: UINT32,
    pub KeepAliveTime: UINT32,
    pub KeepAliveInterval: UINT32,
    pub EnableNagle: BOOLEAN,
    pub EnableTimeStamp: BOOLEAN,
    pub EnableWindowScaling: BOOLEAN,
    pub EnableSelectiveAck: BOOLEAN,
    pub EnablePathMtuDiscovery: BOOLEAN,
}

#[repr(C)]
pub struct EFI_TCP4_CONFIG_DATA {
    pub TypeOfService: UINT8,
    pub TimeToLive: UINT8,
    pub AccessPoint: EFI_TCP4_ACCESS_POINT,
    pub ControlOption: *const EFI_TCP4_OPTION,
}

#[repr(C)]
pub enum EFI_TCP4_CONNECTION_STATE{
    Tcp4StateClosed = 0,
    Tcp4StateListen = 1,
    Tcp4StateSynSent = 2,
    Tcp4StateSynReceived = 3,
    Tcp4StateEstablished = 4,
    Tcp4StateFinWait1 = 5,
    Tcp4StateFinWait2 = 6,
    Tcp4StateClosing = 7,
    Tcp4StateTimeWait = 8,
    Tcp4StateCloseWait = 9,
    Tcp4StateLastAck = 10
}

pub type EFI_TCP4_CONFIGURE = extern "win64" fn(
    This: *const EFI_TCP4_PROTOCOL,
    TcpConfigData: *const EFI_TCP4_CONFIG_DATA,
) -> EFI_STATUS;


pub type EFI_TCP4_ROUTES = extern "win64" fn(
    This: *const EFI_TCP4_PROTOCOL,
    DeleteRoute: BOOLEAN,
    SubnetAddress: *const EFI_IPv4_ADDRESS,
    SubnetMask: *const EFI_IPv4_ADDRESS,
    GatewayAddress: *const EFI_IPv4_ADDRESS
);

pub type EFI_TCP4_CONNECT = extern "win64" fn(
    This: *const EFI_TCP4_PROTOCOL,
    ConnectionToken: *const EFI_TCP4_CONNECTION_TOKEN
) -> EFI_STATUS;

#[repr(C)]
pub struct EFI_TCP4_COMPLETION_TOKEN {
    pub Event: EFI_EVENT,
    pub Status: EFI_STATUS,
}

#[repr(C)]
pub struct EFI_TCP4_CONNECTION_TOKEN {
    pub CompletionToken: EFI_TCP4_COMPLETION_TOKEN,
}

pub type EFI_TCP4_ACCEPT = extern "win64" fn(
    This: *const EFI_TCP4_PROTOCOL,
    ListenToken: *const EFI_TCP4_LISTEN_TOKEN
) -> EFI_STATUS;

#[repr(C)]
pub struct EFI_TCP4_LISTEN_TOKEN {
    pub CompletionToken: EFI_TCP4_COMPLETION_TOKEN,
    pub NewChildHandle: EFI_HANDLE,
}

#[repr(C)]
pub struct EFI_TCP4_TRANSMIT_DATA {
    pub Push: BOOLEAN,
    pub Urgent: BOOLEAN,
    pub DataLength: UINT32,
    pub FragmentCount: UINT32,
    // The actual definition of the FragmentTable field is like this -> FragmentTable: [EFI_TCP4_FRAGMENT_DATA; 1],
    // but we changed it to the pointer definition below because it's easier to deal with during FFI.
    pub FragmentTable: *const EFI_TCP4_FRAGMENT_DATA,
}

pub type EFI_TCP4_TRANSMIT = extern "win64" fn(
    This: *const EFI_TCP4_PROTOCOL,
    Token: *const EFI_TCP4_IO_TOKEN
) -> EFI_STATUS;

#[repr(C)]
pub union PacketUnion {
    pub RxData: *const EFI_TCP4_RECEIVE_DATA,
    pub TxData: *const EFI_TCP4_TRANSMIT_DATA,
}

#[repr(C)]
pub struct EFI_TCP4_IO_TOKEN {
    pub CompletionToken: EFI_TCP4_COMPLETION_TOKEN,
    pub Packet: PacketUnion
}

// #define EFI_CONNECTION_FIN EFIERR (104)
// #define EFI_CONNECTION_RESET EFIERR (105)
// #define EFI_CONNECTION_REFUSED EFIERR (106)

#[repr(C)]
pub struct EFI_TCP4_RECEIVE_DATA {
    pub UrgentFlag: BOOLEAN,
    pub DataLength: UINT32,
    pub FragmentCount: UINT32,
    // The actual definition of the FragmentTable field is like this -> FragmentTable: [EFI_TCP4_FRAGMENT_DATA; 1],
    // but we changed it to the pointer definition below because it's easier to deal with during FFI.
    pub FragmentTable: *const EFI_TCP4_FRAGMENT_DATA,
}

#[repr(C)]
pub struct EFI_TCP4_FRAGMENT_DATA {
    pub FragmentLength: UINT32,
    pub FragmentBuffer: *const VOID,
}

pub type EFI_TCP4_RECEIVE = extern "win64" fn(
    This: *const EFI_TCP4_PROTOCOL,
    Token: *const EFI_TCP4_IO_TOKEN
) -> EFI_STATUS;

pub type EFI_TCP4_CLOSE = extern "win64" fn(
    This: *const EFI_TCP4_PROTOCOL,
    CloseToken: *const EFI_TCP4_CLOSE_TOKEN
) -> EFI_STATUS;

#[repr(C)]
pub struct EFI_TCP4_CLOSE_TOKEN {
    pub CompletionToken: EFI_TCP4_COMPLETION_TOKEN,
    pub AbortOnClose: BOOLEAN,
}

pub type EFI_TCP4_CANCEL = extern "win64" fn(
    This: *const EFI_TCP4_PROTOCOL,
    Token: *const EFI_TCP4_COMPLETION_TOKEN
) -> EFI_STATUS;

pub type EFI_TCP4_POLL = extern "win64" fn(
    This: *const EFI_TCP4_PROTOCOL
) -> EFI_STATUS;