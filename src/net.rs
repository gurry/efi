use ::{
    Result,
    system_table,
    image_handle,
    EfiError,
    to_res,
    io::{Read, Write}
};

use ffi::{
    TRUE,
    FALSE,
    EFI_EVENT,
    EFI_HANDLE,
    EFI_STATUS,
    EFI_SUCCESS,
    EFI_IPv4_ADDRESS,
    EFI_IPv6_ADDRESS,
    UINTN,
    UINT32,
    VOID,
    IsSuccess,
    EFI_SERVICE_BINDING_PROTOCOL,
    boot_services::{
        EFI_BOOT_SERVICES,
        EVT_NOTIFY_WAIT,
        TPL_CALLBACK,
        EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL,
    },
    tcp4::{
        EFI_TCP4_PROTOCOL_GUID,
        EFI_TCP4_SERVICE_BINDING_PROTOCOL_GUID,
        EFI_TCP4_PROTOCOL,
        EFI_TCP4_CONNECTION_TOKEN,
        EFI_TCP4_IO_TOKEN,
        EFI_TCP4_RECEIVE_DATA,
        EFI_TCP4_TRANSMIT_DATA,
        EFI_TCP4_CLOSE_TOKEN,
        EFI_TCP4_CONFIG_DATA,
        EFI_TCP4_ACCESS_POINT,
        EFI_TCP4_OPTION,
        EFI_TCP4_FRAGMENT_DATA 
        },
};

use core::{ptr, mem, ops::Drop};

#[derive(Debug, Copy, Clone)]
pub struct Ipv4Addr(EFI_IPv4_ADDRESS);

impl Ipv4Addr {
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Ipv4Addr(EFI_IPv4_ADDRESS {
            Addr: [a, b, c, d]
        })
    }
}

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

impl Ipv6Addr {
    pub fn new(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> Self {
        Ipv6Addr(EFI_IPv6_ADDRESS {
            Addr: unsafe { mem::transmute([a, b, c, d, e, f, g, h]) } // Transmuting from an 8 elem array of u16 to 16 elem array of UINT8
        })
    }
}

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
    binding_protocol: *const EFI_SERVICE_BINDING_PROTOCOL,
    device_handle: EFI_HANDLE,
    protocol: *mut EFI_TCP4_PROTOCOL,
    connect_token: EFI_TCP4_CONNECTION_TOKEN,
    recv_token: EFI_TCP4_IO_TOKEN,
    send_token: EFI_TCP4_IO_TOKEN,
    close_token: EFI_TCP4_CLOSE_TOKEN
}
extern "win64" fn empty_cb(_event: EFI_EVENT, _context: *const VOID) -> EFI_STATUS {
    EFI_SUCCESS
}

impl Tcp4Stream {
    fn new() -> Self {
        Self { 
            bs: system_table().BootServices,
            binding_protocol: ptr::null() as *const EFI_SERVICE_BINDING_PROTOCOL,
            device_handle: ptr::null() as EFI_HANDLE,
            protocol: ptr::null::<EFI_TCP4_PROTOCOL>() as *mut EFI_TCP4_PROTOCOL,
            connect_token: EFI_TCP4_CONNECTION_TOKEN::default(),
            recv_token: EFI_TCP4_IO_TOKEN::default(),
            send_token: EFI_TCP4_IO_TOKEN::default(),
            close_token: EFI_TCP4_CLOSE_TOKEN::default(),
        }
    }

    // TODO: Ideally this interface should be identical to the one in stdlib which is:
    // pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<TcpStream> {
    pub fn connect(addr: SocketAddrV4) -> Result<Self> {
        let ip: EFI_IPv4_ADDRESS = (*addr.ip()).into();
        let config_data = EFI_TCP4_CONFIG_DATA {
            TypeOfService: 0,
            TimeToLive: 255,
            AccessPoint: EFI_TCP4_ACCESS_POINT {
                UseDefaultAddress: TRUE,
                StationAddress: EFI_IPv4_ADDRESS::zero(),
                SubnetMask: EFI_IPv4_ADDRESS::zero(),
                StationPort: 0,
                RemoteAddress: ip,
                RemotePort: addr.port(),
                ActiveFlag: TRUE,
            },
            ControlOption: ptr::null() as *const EFI_TCP4_OPTION 
        };

        let mut stream = Self::new();
        unsafe {
            // TODO: is there a better way than using a macro to return early? How about newtyping the usize return type of FFI calls and then working off that?
            ret_on_err!(((*stream.bs).CreateEvent)(EVT_NOTIFY_WAIT, TPL_CALLBACK, empty_cb, ptr::null(), &mut stream.connect_token.CompletionToken.Event));
            ret_on_err!(((*stream.bs).CreateEvent)(EVT_NOTIFY_WAIT, TPL_CALLBACK, empty_cb, ptr::null(), &mut stream.send_token.CompletionToken.Event));
            ret_on_err!(((*stream.bs).CreateEvent)(EVT_NOTIFY_WAIT, TPL_CALLBACK, empty_cb, ptr::null(), &mut stream.recv_token.CompletionToken.Event));
            ret_on_err!(((*stream.bs).CreateEvent)(EVT_NOTIFY_WAIT, TPL_CALLBACK, empty_cb, ptr::null(), &mut stream.close_token.CompletionToken.Event));

            ret_on_err!(((*stream.bs).LocateProtocol)(&EFI_TCP4_SERVICE_BINDING_PROTOCOL_GUID, ptr::null() as *const VOID, mem::transmute(&stream.binding_protocol)));

            ret_on_err!(((*stream.binding_protocol).CreateChild)(stream.binding_protocol, &mut stream.device_handle));

            ret_on_err!(((*stream.bs).OpenProtocol)(stream.device_handle,
                &EFI_TCP4_PROTOCOL_GUID,
                mem::transmute(&stream.protocol),
                image_handle(),
                ptr::null() as EFI_HANDLE,
                EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL)); // TODO: BY_HANDLE is used for applications. Drivers should use GET. Will we ever support drivers?
        
            ret_on_err!(((*stream.protocol).Configure)(stream.protocol, &config_data));

            ret_on_err!(((*stream.protocol).Connect)(stream.protocol, &mut stream.connect_token));
            stream.wait_for_evt(&stream.connect_token.CompletionToken.Event)?; // TODO: Make sure we also check the status on the Event.Status field
        }

        Ok(stream)
    }

    unsafe fn wait_for_evt(&self, event: *const EFI_EVENT) -> Result<()> {
        let mut _index: UINTN = 0;;
        let status = ((*self.bs).WaitForEvent)(1, event, &mut _index);
        to_res((), status)
    }
}

impl Drop for Tcp4Stream {
    fn drop(&mut self) {
        // TODO: add the code to panic when any of the below calls fail. (Could be difficult) but maybe we can trace something when we do that.
        unsafe {
            ((*self.protocol).Close)(self.protocol, &self.close_token);
            ((*self.bs).CloseProtocol)(self.device_handle, &EFI_TCP4_PROTOCOL_GUID, image_handle(), ptr::null() as EFI_HANDLE);
            ((*self.binding_protocol).DestroyChild)(self.binding_protocol, &mut self.device_handle);

            ((*self.bs).CloseEvent)(self.connect_token.CompletionToken.Event);
            ((*self.bs).CloseEvent)(self.send_token.CompletionToken.Event);
            ((*self.bs).CloseEvent)(self.recv_token.CompletionToken.Event);
            ((*self.bs).CloseEvent)(self.close_token.CompletionToken.Event);
        }
    }
}
impl Read for Tcp4Stream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let fragment_data = EFI_TCP4_FRAGMENT_DATA {
            FragmentLength: buf.len() as UINT32,
            FragmentBuffer: buf.as_ptr() as *const VOID
        };

        let recv_data = EFI_TCP4_RECEIVE_DATA {
            UrgentFlag: FALSE,
            DataLength: buf.len() as UINT32,
            FragmentCount: 1,
            FragmentTable: [fragment_data] // TODO: will this result in a copy? Should be init fragment data in place here?
        };


        self.recv_token.Packet.RxData =  &recv_data;
        ret_on_err!(unsafe { ((*self.protocol).Receive)(self.protocol, &self.recv_token) });

        unsafe { self.wait_for_evt(&self.recv_token.CompletionToken.Event)? }; // TODO: Make sure we also check the status on the Event.Status field
        to_res(buf.len(), self.recv_token.CompletionToken.Status)
    }
}

impl Write for Tcp4Stream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let fragment_data = EFI_TCP4_FRAGMENT_DATA {
            FragmentLength: buf.len() as UINT32,
            FragmentBuffer: buf.as_ptr() as *const VOID
        };

        let send_data = EFI_TCP4_TRANSMIT_DATA {
            Push: FALSE,
            Urgent: FALSE,
            DataLength: buf.len() as UINT32,
            FragmentCount: 1,
            FragmentTable: [fragment_data] // TODO: will this result in a copy? Should be init fragment data in place here?
        };

        self.send_token.Packet.TxData =  &send_data;
        ret_on_err!(unsafe { ((*self.protocol).Transmit)(self.protocol, &self.send_token) });

        unsafe { self.wait_for_evt(&self.send_token.CompletionToken.Event)? }; // TODO: Make sure we also check the status on the Event.Status field
        to_res(buf.len(), self.send_token.CompletionToken.Status)
    }
}