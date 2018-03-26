use ::{
    Result,
    system_table,
    EfiError,
};

use ffi::{
    TRUE,
    EFI_EVENT,
    EFI_IPv4_ADDRESS,
    EFI_IPv6_ADDRESS,
    VOID,
    IsSuccess,
    boot_services::{
        EFI_BOOT_SERVICES,
        EVT_NOTIFY_SIGNAL,
        EFI_EVENT_NOTIFY,
        TPL_CALLBACK
    },
    tcp4::{
        EFI_TCP4_COMPLETION_TOKEN,
        EFI_TCP4_CONNECTION_TOKEN,
        EFI_TCP4_IO_TOKEN,
        EFI_TCP4_RECEIVE_DATA,
        EFI_TCP4_TRANSMIT_DATA,
        EFI_TCP4_CLOSE_TOKEN,
        EFI_TCP4_CONFIG_DATA,
        EFI_TCP4_ACCESS_POINT,
        EFI_TCP4_OPTION,
    },
};

use core::{ptr, mem};

#[derive(Debug, Copy, Clone)]
pub struct Ipv4Addr(EFI_IPv4_ADDRESS);

impl From<EFI_IPv4_ADDRESS> for Ipv4Addr {
    fn from(val: EFI_IPv4_ADDRESS) -> Self {
        Ipv4Addr(val)
    }
}

impl From<Ipv4Addr > for EFI_IPv4_ADDRESS {
    fn from(val: Ipv4Addr) -> Self {
        val.0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Ipv6Addr(EFI_IPv6_ADDRESS);

impl From<EFI_IPv6_ADDRESS> for Ipv6Addr {
    fn from(val: EFI_IPv6_ADDRESS) -> Self {
        Ipv6Addr(val)
    }
}

impl From<Ipv6Addr > for EFI_IPv6_ADDRESS {
    fn from(val: Ipv6Addr) -> Self {
        val.0
    }
}

pub enum IpAddr {
    V4(Ipv4Addr),
    V6(Ipv6Addr)
}

pub struct SocketAddrV4 {
    ip: Ipv4Addr,
    port: u16,
}

impl SocketAddrV4 {
    pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        Self { ip, port }
    }

    pub fn ip(&self) -> &Ipv4Addr {
        &self.ip
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

pub struct SocketAddrV6 {
    ip: Ipv6Addr,
    port: u16,
}

impl SocketAddrV6 {
    pub fn new(ip: Ipv6Addr, port: u16) -> Self {
        Self { ip, port }
    }

    pub fn ip(&self) -> &Ipv6Addr {
        &self.ip
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

pub enum SocketAddr {
    V4(SocketAddrV4),
    V6(SocketAddrV6)
}

pub struct Tcp4Stream {
    bs: *mut EFI_BOOT_SERVICES,
    connect_token: EFI_TCP4_CONNECTION_TOKEN,
    recv_token: EFI_TCP4_IO_TOKEN,
    send_token: EFI_TCP4_IO_TOKEN,
    close_token: EFI_TCP4_CLOSE_TOKEN
}

impl Tcp4Stream {
    pub fn new() -> Self {
        Self { 
            bs: system_table().BootServices,
            connect_token: EFI_TCP4_CONNECTION_TOKEN::default(),
            recv_token: EFI_TCP4_IO_TOKEN::default(),
            send_token: EFI_TCP4_IO_TOKEN::default(),
            close_token: EFI_TCP4_CLOSE_TOKEN::default(),
        }
    }

    // TODO: Ideally this interface should be identical to the one in stdlib which is:
    // pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<TcpStream> {
    pub fn connect(addr: SocketAddrV4) -> Result<Self> {
        let config_data = EFI_TCP4_CONFIG_DATA {
            TypeOfService: 0,
            TimeToLive: 255,
            AccessPoint: EFI_TCP4_ACCESS_POINT {
                UseDefaultAddress: TRUE,
                StationAddress: EFI_IPv4_ADDRESS::zero(),
                SubnetMask: EFI_IPv4_ADDRESS::zero(),
                StationPort: 0,
                RemoteAddress: (*addr.ip()).into(), // TODO: this deref is awkward. Can we do better?
                RemotePort: addr.port(),
                ActiveFlag: TRUE,
            },
            ControlOption: ptr::null() as *const EFI_TCP4_OPTION 
        };

        let stream = Self::new();
        unsafe {
            let null_callback = mem::transmute::<*const VOID, EFI_EVENT_NOTIFY>(ptr::null());
            // TODO: is there a better way than using a macro to return early? How about newtyping the usize return type of FFI calls and then working off that?
            ret_on_err!(((*stream.bs).CreateEvent)(EVT_NOTIFY_SIGNAL, TPL_CALLBACK, null_callback, ptr::null(), mem::transmute(stream.connect_token.CompletionToken.Event)));
            ret_on_err!(((*stream.bs).CreateEvent)(EVT_NOTIFY_SIGNAL, TPL_CALLBACK, null_callback, ptr::null(), mem::transmute(stream.send_token.CompletionToken.Event)));
            ret_on_err!(((*stream.bs).CreateEvent)(EVT_NOTIFY_SIGNAL, TPL_CALLBACK, null_callback, ptr::null(), mem::transmute(stream.recv_token.CompletionToken.Event)));
            ret_on_err!(((*stream.bs).CreateEvent)(EVT_NOTIFY_SIGNAL, TPL_CALLBACK, null_callback, ptr::null(), mem::transmute(stream.close_token.CompletionToken.Event)));
        }
        
        Ok(stream)
    }
}