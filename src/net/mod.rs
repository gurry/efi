use ::{
    Result,
    system_table,
    image_handle,
    EfiError,
    EfiErrorKind,
    to_res,
    io::{self, Read, Write}
};
use ffi::{
    TRUE,
    FALSE,
    EFI_EVENT,
    EFI_HANDLE,
    EFI_STATUS,
    EFI_SUCCESS,
    EFI_IPv4_ADDRESS,
    UINTN,
    UINT32,
    VOID,
    EFI_SERVICE_BINDING_PROTOCOL,
    EFI_NO_MAPPING,
    boot_services::{
        EFI_BOOT_SERVICES,
        EVT_NOTIFY_WAIT,
        EVT_NOTIFY_SIGNAL,
        TPL_CALLBACK,
        TPL_NOTIFY,
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
    udp4::{
        EFI_UDP4_SERVICE_BINDING_PROTOCOL_GUID,
        EFI_UDP4_PROTOCOL_GUID,
        EFI_UDP4_PROTOCOL,
        EFI_UDP4_CONFIG_DATA,
        EFI_UDP4_COMPLETION_TOKEN,
        EFI_UDP4_FRAGMENT_DATA,
        EFI_UDP4_TRANSMIT_DATA,
        EFI_UDP4_SESSION_DATA
    },
    ip4::EFI_IP4_MODE_DATA,
};

use core::{ptr, mem, ops::Drop};
pub use self::addr::*;

pub mod addr;
pub mod dns;
pub mod dhcp;
mod parser;

// TODO: There are no timeouts anywhere (e.g. connect, read, write etc.). Add timeouts at all those places
pub struct TcpStream {
    tcp4_stream: Tcp4Stream,
}

fn for_ip4_only<A: ToSocketAddrs, S>(addr: A, callback: fn(SocketAddrV4) -> Result<S>) -> Result<S> {
    let socket_addrs = addr.to_socket_addrs().map_err(|_| ::EfiError::from(::EfiErrorKind::DeviceError))?; // Not just doing into() on EfiErrKind because compiler wants type annotations

    for addr in socket_addrs {
        match addr {
            SocketAddr::V4(addr) => {
                match callback(addr) {
                    Ok(s) => return Ok(s),
                    Err(_) => continue, // TODO: ideally we should be gathering all the errors here to return at the end if no addr works
                }
            },
            SocketAddr::V6(_) => {}
        }
    }

    // TODO: If all Ipv4 addresses didn't work and all we got left with was ipv6,
    // our error must say something to the effect of "Ipv6 not supported yet" 
    Err(EfiErrorKind::DeviceError.into())
}

impl TcpStream {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        Ok(Self {tcp4_stream: for_ip4_only(addr, |addr| Tcp4Stream::connect(addr))? })
    }

    pub fn peer_addr(&self) -> Result<SocketAddr> {
        self.tcp4_stream.peer_addr().map(|a| SocketAddr::V4(a))
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.tcp4_stream.local_addr().map(|a| SocketAddr::V4(a))
    }
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.tcp4_stream.read(buf)
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.tcp4_stream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.tcp4_stream.flush()
    }
}

struct Tcp4Stream {
    bs: *mut EFI_BOOT_SERVICES,
    binding_protocol: *const EFI_SERVICE_BINDING_PROTOCOL,
    device_handle: EFI_HANDLE,
    protocol: *mut EFI_TCP4_PROTOCOL,
    connect_token: EFI_TCP4_CONNECTION_TOKEN,
    recv_token: EFI_TCP4_IO_TOKEN,
    send_token: EFI_TCP4_IO_TOKEN,
    close_token: EFI_TCP4_CLOSE_TOKEN,
    is_connected: bool
}

extern "win64" fn empty_cb(_event: EFI_EVENT, _context: *const VOID) -> EFI_STATUS {
    EFI_SUCCESS
}

static mut RECV_DONE: bool = false;
extern "win64" fn recv_cb(_event: EFI_EVENT, _context: *const VOID) -> EFI_STATUS {
    unsafe { RECV_DONE = true };
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
            is_connected: false
        }
    }

    fn connect(addr: SocketAddrV4) -> Result<Self> {
        // TODO: this function is too ugly right now. Refactor/clean it up.
        let ip: EFI_IPv4_ADDRESS = (*addr.ip()).into();

        let dhcp_config = dhcp::cached_dhcp_config()?
                .ok_or_else(|| ::EfiError::from(::EfiErrorKind::DeviceError))?;

        let station_ip = if let IpAddr::V4(ip) = dhcp_config.ip() { ip.into() } else { EFI_IPv4_ADDRESS::zero() };
        let subnet_mask = if let IpAddr::V4(ip) = dhcp_config.subnet_mask() { ip.into() } else { EFI_IPv4_ADDRESS::zero() };
        let config_data = EFI_TCP4_CONFIG_DATA {
            TypeOfService: 0,
            TimeToLive: 255,
            AccessPoint: EFI_TCP4_ACCESS_POINT {
                UseDefaultAddress: FALSE,
                // TODO: make the use of DefaultAddress here vs using the addr from the DHCP config 
                // configurable via some settings on this class similar to Rust std lib
                StationAddress: station_ip, //EFI_IPv4_ADDRESS::zero(),
                SubnetMask: subnet_mask, //EFI_IPv4_ADDRESS::zero(),
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
            ret_on_err!(((*stream.bs).CreateEvent)(EVT_NOTIFY_SIGNAL, TPL_NOTIFY, recv_cb, ptr::null(), &mut stream.recv_token.CompletionToken.Event));
            ret_on_err!(((*stream.bs).CreateEvent)(EVT_NOTIFY_WAIT, TPL_CALLBACK, empty_cb, ptr::null(), &mut stream.close_token.CompletionToken.Event));

            ret_on_err!(((*stream.bs).LocateProtocol)(&EFI_TCP4_SERVICE_BINDING_PROTOCOL_GUID, ptr::null() as *const VOID, mem::transmute(&stream.binding_protocol)));

            ret_on_err!(((*stream.binding_protocol).CreateChild)(stream.binding_protocol, &mut stream.device_handle));

            ret_on_err!(((*stream.bs).OpenProtocol)(stream.device_handle,
                &EFI_TCP4_PROTOCOL_GUID,
                mem::transmute(&stream.protocol),
                image_handle(),
                ptr::null() as EFI_HANDLE,
                EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL)); // TODO: BY_HANDLE is used for applications. Drivers should use GET. Will we ever support drivers?
        
            let status = ((*stream.protocol).Configure)(stream.protocol, &config_data);

            if status == EFI_NO_MAPPING { // Wait until the IP configuration process (probably DHCP) has finished
                let mut ip_mode_data = EFI_IP4_MODE_DATA::new();
                loop {
                    // TODO: This becomes an infinite loop on some firmeware such as Hyper-v
                    // Figure out why and fix it.
                    ret_on_err!(((*stream.protocol).GetModeData)(stream.protocol, ptr::null_mut(), ptr::null_mut(), &mut ip_mode_data, ptr::null_mut(), ptr::null_mut()));
                    if ip_mode_data.IsConfigured == TRUE { break }
                }

                ret_on_err!(((*stream.protocol).Configure)(stream.protocol, &config_data));
            } else {
                ret_on_err!(status);
            }

            ret_on_err!(((*stream.protocol).Connect)(stream.protocol, &mut stream.connect_token));
            stream.wait_for_evt(&stream.connect_token.CompletionToken.Event)?;
            ret_on_err!(stream.connect_token.CompletionToken.Status);
            stream.is_connected = true;
        }

        // TODO: We should try to close all events that have been created if we're returning early

        Ok(stream)
    }

    fn peer_addr(&self) -> Result<SocketAddrV4> {
        let mut config_data = EFI_TCP4_CONFIG_DATA::default();
        self.get_config_data(&mut config_data)?;
        Ok(SocketAddrV4::new(config_data.AccessPoint.RemoteAddress.into(), config_data.AccessPoint.RemotePort))
    }

    fn local_addr(&self) -> Result<SocketAddrV4> {
        let mut config_data = EFI_TCP4_CONFIG_DATA::default();
        self.get_config_data(&mut config_data)?;
        Ok(SocketAddrV4::new(config_data.AccessPoint.StationAddress.into(), config_data.AccessPoint.StationPort))
    }

    fn get_config_data(&self, config_data: &mut EFI_TCP4_CONFIG_DATA) -> Result<()> {
        unsafe {
            ret_on_err!(((*self.protocol).GetModeData)(self.protocol, 
                ptr::null_mut(),
                config_data,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut()));
        }
        Ok(())
    }

    unsafe fn wait_for_evt(&self, event: *const EFI_EVENT) -> Result<()> {
        let mut _index: UINTN = 0;;
        let status = ((*self.bs).WaitForEvent)(1, event, &mut _index);
        to_res((), status)
    }

    fn read_buf(&mut self, buf: &mut [u8]) -> Result<usize> {
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


        unsafe { RECV_DONE = false };
        self.recv_token.Packet.RxData =  &recv_data;
        ret_on_err!(unsafe { ((*self.protocol).Receive)(self.protocol, &self.recv_token) });

        // TODO: add a read timeout. Can be done by setting a timer for the length of the timeout
        while unsafe { !RECV_DONE } {
            ret_on_err!(unsafe { ((*self.protocol).Poll)(self.protocol) });
        }

        to_res(recv_data.DataLength as usize, self.recv_token.CompletionToken.Status)
    }

    fn write_buf(&mut self, buf: &[u8]) -> Result<usize> {
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

        // TODO: Add polling here to make transmit fast just like we do in read_buf above.
        unsafe { self.wait_for_evt(&self.send_token.CompletionToken.Event)? }; // TODO: Make sure we also check the status on the Event.Status field
        // TODO: is it okay to return buf len below? Would UEFI every tranmist part of the buffer. 
        // The documentation is unclear about this. Check this with experimentation
        to_res(buf.len(), self.send_token.CompletionToken.Status)
    }
}

impl Drop for Tcp4Stream {
    fn drop(&mut self) {
        // TODO: add the code to panic when any of the below calls fail. (Could be difficult) but maybe we can trace something when we do that.
        unsafe {
            ((*self.bs).CloseEvent)(self.connect_token.CompletionToken.Event);
            ((*self.bs).CloseEvent)(self.send_token.CompletionToken.Event);
            ((*self.bs).CloseEvent)(self.recv_token.CompletionToken.Event);

            self.close_token.AbortOnClose = FALSE;

            ((*self.protocol).Close)(self.protocol, &self.close_token);
            if self.is_connected { // We don't want want to wait if we weren't connected because then we end up waiting forever
                if let Err(_) = self.wait_for_evt(&self.close_token.CompletionToken.Event) { // Blocking until the connection is closed for certain
                     return; // Don't do anything further since we failed to close the connection safely.
                }
            }

            // This Configure call and the comment about the bug is copied verbatim from FastBoot protocol in tianocore:
            // Possible bug in EDK2 TCP4 driver: closing a connection doesn't remove its
            // PCB from the list of live connections. Subsequent attempts to Configure()
            // a TCP instance with the same local port will fail with INVALID_PARAMETER.
            // Calling Configure with NULL is a workaround for this issue.
            ((*self.protocol).Configure)(self.protocol, ptr::null());

            ((*self.bs).CloseEvent)(self.close_token.CompletionToken.Event);
            ((*self.binding_protocol).DestroyChild)(self.binding_protocol, &mut self.device_handle);
        }
    }
}

impl Read for Tcp4Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.read_buf(buf).map_err(|e| {
            match e.kind() {
                // Handling errors that indicate connection closed specially so the caller can retry
                EfiErrorKind::ConnectionReset => io::ErrorKind::ConnectionReset.into(),
                EfiErrorKind::ConnectionFin => io::ErrorKind::ConnectionAborted.into(),
                EfiErrorKind::AccessDenied => io::ErrorKind::NotConnected.into(), // As per UEFI spec we get access denied error when the connection has been closed
                EfiErrorKind::Timeout => io::ErrorKind::TimedOut.into(),
                _ => io::ErrorKind::Other.into(),
            }
        })
    }
}

impl Write for Tcp4Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_buf(buf).map_err(|e| {
            match e.kind() {
                // Handling errors that indicate connection closed specially so the caller can retry
                EfiErrorKind::ConnectionReset => io::ErrorKind::ConnectionReset.into(),
                EfiErrorKind::ConnectionFin => io::ErrorKind::ConnectionAborted.into(),
                EfiErrorKind::AccessDenied => io::ErrorKind::NotConnected.into(), // As per UEFI spec we get access denied error when the connection has been closed
                EfiErrorKind::Timeout => io::ErrorKind::TimedOut.into(),
                _ => io::ErrorKind::Other.into(),
            }
        })

    }


    fn flush(&mut self) -> io::Result<()> {
        // Does nothing. There's nothing in the underlying UEFI APIs to support this.
        Ok(())
    }
}


pub struct UdpSocket {
    udp4_socket: Udp4Socket,
}

impl UdpSocket {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        Ok(Self {udp4_socket: for_ip4_only(addr, |addr| Udp4Socket::connect(addr))? })
    }

    pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.udp4_socket.recv_buf(buf)
    }

    pub fn send(&mut self, buf: &[u8]) -> Result<usize> {
        self.udp4_socket.send_buf(buf)
    }
}

struct Udp4Socket {
    bs: *const EFI_BOOT_SERVICES,
    binding_protocol: *const EFI_SERVICE_BINDING_PROTOCOL,
    protocol: *const EFI_UDP4_PROTOCOL,
    device_handle: EFI_HANDLE,
    recv_token: EFI_UDP4_COMPLETION_TOKEN,
    send_token: EFI_UDP4_COMPLETION_TOKEN,
}

impl Udp4Socket {
    fn new() -> Self {
        Udp4Socket {
            bs: system_table().BootServices,
            binding_protocol: ptr::null() as *const EFI_SERVICE_BINDING_PROTOCOL,
            protocol: ptr::null() as *const EFI_UDP4_PROTOCOL,
            device_handle: ptr::null() as EFI_HANDLE,
            recv_token: EFI_UDP4_COMPLETION_TOKEN::default(),
            send_token: EFI_UDP4_COMPLETION_TOKEN::default(),
        }
    }

    fn connect(addr: SocketAddrV4) -> Result<Self> {
        let ip: EFI_IPv4_ADDRESS = (*addr.ip()).into();
        let dhcp_config = dhcp::cached_dhcp_config()?
                .ok_or_else(|| ::EfiError::from(::EfiErrorKind::DeviceError))?;

        let station_ip = if let IpAddr::V4(ip) = dhcp_config.ip() { ip.into() } else { EFI_IPv4_ADDRESS::zero() };
        let subnet_mask = if let IpAddr::V4(ip) = dhcp_config.subnet_mask() { ip.into() } else { EFI_IPv4_ADDRESS::zero() };
        let config_data = EFI_UDP4_CONFIG_DATA {
            AcceptBroadcast: FALSE,
            AcceptPromiscuous: FALSE,
            AcceptAnyPort: FALSE,
            AllowDuplicatePort: FALSE,
            TypeOfService: 0,
            TimeToLive: 255,
            DoNotFragment: TRUE,
            ReceiveTimeout: 0,
            TransmitTimeout: 0,
            UseDefaultAddress: FALSE,
            // TODO: make the use of DefaultAddress here vs using the addr from the DHCP config 
            // configurable via some settings on this class similar to Rust std lib
            StationAddress: station_ip, //EFI_IPv4_ADDRESS::zero(),
            SubnetMask: subnet_mask, //EFI_IPv4_ADDRESS::zero(),
            StationPort: 0,
            RemoteAddress: ip,
            RemotePort: addr.port(),
        };
        let mut socket = Self::new();

        unsafe {
            ret_on_err!(((*socket.bs).CreateEvent)(EVT_NOTIFY_WAIT, TPL_CALLBACK, empty_cb, ptr::null(), &mut socket.send_token.Event));
            ret_on_err!(((*socket.bs).CreateEvent)(EVT_NOTIFY_WAIT, TPL_CALLBACK, empty_cb, ptr::null(), &mut socket.recv_token.Event));

            ret_on_err!(((*socket.bs).LocateProtocol)(&EFI_UDP4_SERVICE_BINDING_PROTOCOL_GUID, ptr::null() as *const VOID, mem::transmute(&socket.binding_protocol)));
            ret_on_err!(((*socket.binding_protocol).CreateChild)(socket.binding_protocol, &mut socket.device_handle));
            ret_on_err!(((*socket.bs).OpenProtocol)(socket.device_handle,
                &EFI_UDP4_PROTOCOL_GUID,
                mem::transmute(&socket.protocol),
                image_handle(),
                ptr::null() as EFI_HANDLE,
                EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL)); // TODO: BY_HANDLE is used for applications. Drivers should use GET. Will we ever support drivers?
            let status = ((*socket.protocol).Configure)(socket.protocol, &config_data);
            if status == EFI_NO_MAPPING { // Wait until the IP configuration process (probably DHCP) has finished
                let mut ip_mode_data = EFI_IP4_MODE_DATA::new();
                loop {
                    // TODO: This becomes an infinite loop on some firmeware such as Hyper-v
                    // Figure out why and fix it.
                    ret_on_err!(((*socket.protocol).GetModeData)(socket.protocol, ptr::null_mut(), &mut ip_mode_data, ptr::null_mut(), ptr::null_mut()));
                    if ip_mode_data.IsConfigured == TRUE { break }
                }

                ret_on_err!(((*socket.protocol).Configure)(socket.protocol, &config_data));
            } else {
                ret_on_err!(status);
            }
        }

        // TODO: We should try to close all events that have been created if we're returning early

        Ok(socket)
    }

    unsafe fn wait_for_evt(&self, event: *const EFI_EVENT) -> Result<()> {
        let mut _index: UINTN = 0;;
        let status = ((*self.bs).WaitForEvent)(1, event, &mut _index);
        to_res((), status)
    }

    fn recv_buf(&mut self, buf: &mut [u8]) -> Result<usize> {
        ret_on_err!(unsafe { ((*self.protocol).Receive)(self.protocol, &self.recv_token) });

        let buffer_length: usize;
        unsafe {
            self.wait_for_evt(&self.recv_token.Event)?;
            let buffer = (*self.recv_token.Packet.RxData).FragmentTable[0].FragmentBuffer as *const u8;
            buffer_length = (*self.recv_token.Packet.RxData).FragmentTable[0].FragmentLength as usize;
            if buf.len() < buffer_length {
                return Err(EfiError::from(::ffi::EFI_INVALID_PARAMETER));
            }
            //TODO:Get rid of this copy
            ptr::copy(buffer, buf.as_mut_ptr(), buffer_length);
        }

        to_res(buffer_length, self.recv_token.Status)
    }

    fn send_buf(&mut self, buf: &[u8]) -> Result<usize> {
        let fragment_data = EFI_UDP4_FRAGMENT_DATA {
            FragmentLength: buf.len() as UINT32,
            FragmentBuffer: buf.as_ptr() as *const VOID
        };

        let send_data = EFI_UDP4_TRANSMIT_DATA {
            UdpSessionData: ptr::null() as *const EFI_UDP4_SESSION_DATA,
            GatewayAddress: ptr::null() as *const EFI_IPv4_ADDRESS,
            DataLength: buf.len() as UINT32,
            FragmentCount: 1,
            FragmentTable: [fragment_data] // TODO: will this result in a copy? Should be init fragment data in place here?
        };

        self.send_token.Packet.TxData =  &send_data;
        ret_on_err!(unsafe { ((*self.protocol).Transmit)(self.protocol, &self.send_token) });

        unsafe { self.wait_for_evt(&self.send_token.Event)? }; // TODO: Make sure we also check the status on the Event.Status field
        to_res(buf.len(), self.send_token.Status)
    }
}

impl Drop for Udp4Socket {
    fn drop(&mut self) {
        // TODO: add the code to panic when any of the below calls fail. (Could be difficult) but maybe we can trace something when we do that.
        unsafe {
            ((*self.bs).CloseEvent)(self.send_token.Event);
            ((*self.bs).CloseEvent)(self.recv_token.Event);
            ((*self.binding_protocol).DestroyChild)(self.binding_protocol, &mut self.device_handle);
        }
    }
}
