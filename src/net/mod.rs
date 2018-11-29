pub mod addr;
pub mod dns;
pub mod pxebc;
pub mod ifconfig;
mod parser;

use ::{
    Result,
    system_table,
    image_handle,
    EfiError,
    EfiErrorKind,
    to_res,
    io::{self, Read, Write},
    events::{self, TimerSchedule, TimerState, EventTpl, Wait},
};
use self::pxebc::DhcpConfig;
use ffi::{
    TRUE,
    FALSE,
    EFI_EVENT,
    EFI_HANDLE,
    EFI_STATUS,
    EFI_SUCCESS,
    EFI_NOT_READY,
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

use core::{ptr, mem, ops::Drop, time::Duration};
pub use self::addr::*;

// TODO: There are no timeouts anywhere (e.g. connect, read, write etc.). Add timeouts at all those places
pub struct TcpStream {
    tcp4_stream: Tcp4Stream,
}

fn for_ip4_only<A: ToSocketAddrs, F: FnMut(SocketAddrV4) -> Result<S>, S>(addr: A, mut callback: F) -> Result<S> {
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

static mut OP_DONE: bool = false;
extern "win64" fn common_cb(_event: EFI_EVENT, _context: *const VOID) -> EFI_STATUS {
    unsafe { OP_DONE = true };
    EFI_SUCCESS
}

fn reset_op_done() {
    unsafe { OP_DONE = false }
}

fn op_done() -> bool {
    unsafe { OP_DONE }
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

        let dhcp_config = pxebc::PxeBaseCodeProtocol::get_any()?
            .ok_or_else(|| ::EfiError::from(::EfiErrorKind::DeviceError))?
            .cached_dhcp_config()?
            .ok_or_else(|| ::EfiError::from(::EfiErrorKind::DeviceError))?;

        let station_ip = if let IpAddr::V4(ip) = dhcp_config.ip() { ip.into() } else { EFI_IPv4_ADDRESS::zero() };
        let subnet_mask = if let IpAddr::V4(ip) = dhcp_config.subnet_mask() { ip.into() } else { EFI_IPv4_ADDRESS::zero() };
        let config_data = EFI_TCP4_CONFIG_DATA {
            TypeOfService: 0,
            TimeToLive: 255,
            AccessPoint: EFI_TCP4_ACCESS_POINT {
                UseDefaultAddress: FALSE,
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
            ret_on_err!(((*stream.bs).CreateEvent)(EVT_NOTIFY_WAIT, TPL_CALLBACK, Some(empty_cb), ptr::null(), &mut stream.connect_token.CompletionToken.Event));
            ret_on_err!(((*stream.bs).CreateEvent)(EVT_NOTIFY_WAIT, TPL_CALLBACK, Some(empty_cb), ptr::null(), &mut stream.send_token.CompletionToken.Event));
            ret_on_err!(((*stream.bs).CreateEvent)(EVT_NOTIFY_SIGNAL, TPL_NOTIFY, Some(common_cb), ptr::null(), &mut stream.recv_token.CompletionToken.Event));
            ret_on_err!(((*stream.bs).CreateEvent)(EVT_NOTIFY_WAIT, TPL_CALLBACK, Some(empty_cb), ptr::null(), &mut stream.close_token.CompletionToken.Event));

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

        }

        // Copy in all routes from the DHCP config
        // TODO: This is faulty. Get the dhcp config specifically of the interface we're binding on
        let (subnet_addr, subnet_mask, gateway_addr) = form_default_route(&dhcp_config)?;
        unsafe {
            ret_on_err!(((*stream.protocol).Routes)(stream.protocol, FALSE, &subnet_addr, &subnet_mask, &gateway_addr));

            ret_on_err!(((*stream.protocol).Connect)(stream.protocol, &mut stream.connect_token));
            stream.wait_for_evt(&stream.connect_token.CompletionToken.Event)?;
            ret_on_err!(stream.connect_token.CompletionToken.Status);
            stream.is_connected = true;
        }

        // TODO: We should try to close all events that have been created if we're returning early

        Ok(stream)
    }

    fn peer_addr(&self) -> Result<SocketAddrV4> {
        let config_data = self.get_config_data()?;
        Ok(SocketAddrV4::new(config_data.AccessPoint.RemoteAddress.into(), config_data.AccessPoint.RemotePort))
    }

    fn local_addr(&self) -> Result<SocketAddrV4> {
        let config_data = self.get_config_data()?;
        Ok(SocketAddrV4::new(config_data.AccessPoint.StationAddress.into(), config_data.AccessPoint.StationPort))
    }

    fn get_config_data(&self) -> Result<EFI_TCP4_CONFIG_DATA> {
        let mut config_data = EFI_TCP4_CONFIG_DATA::default();
        unsafe {
            ret_on_err!(((*self.protocol).GetModeData)(self.protocol, 
                ptr::null_mut(),
                &mut config_data,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut()));
        }
        Ok(config_data)
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


        reset_op_done();
        self.recv_token.Packet.RxData =  &recv_data;
        ret_on_err!(unsafe { ((*self.protocol).Receive)(self.protocol, &self.recv_token) });

        // TODO: add a read timeout. Can be done by setting a timer for the length of the timeout
        while !op_done() {
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
    pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        Ok(Self {udp4_socket: for_ip4_only(addr, |addr| Udp4Socket::bind(addr))? })
    }

    // TODO: Fix this bullshit around how we're creating a new socket on every connect
    // (we're doing this because UEFI doesn't allow us to change the address of an already created UDP protocol)
    pub fn connect<A: ToSocketAddrs>(&mut self, addr: A) -> Result<()> {
        for_ip4_only(addr, |addr| {
            let bound_addr = self.udp4_socket.bound_addr;
            self.udp4_socket = Udp4Socket::bind_and_connect(bound_addr, addr)?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.udp4_socket.recv_buf(buf)
    }

    // TODO: need to make self non-mut just like in the std lib
    pub fn send(&mut self, buf: &[u8]) -> Result<usize> {
        self.udp4_socket.send_buf(buf, None)
    }

    // TODO: implement recv_from() as well

    // TODO: need to make self non-mut just like in the std lib
    pub fn send_to<A: ToSocketAddrs>(&mut self, buf: &[u8], addr: A) -> Result<usize> {
        let mut last_error = EfiError::from(EfiErrorKind::InvalidParameter);
        let socket_addrs = addr.to_socket_addrs().map_err(|_| ::EfiError::from(::EfiErrorKind::DeviceError))?; // Not just doing into() on EfiErrKind because compiler wants type annotations
        for addr in socket_addrs {
            if let SocketAddr::V4(addr) = addr {
                let session_data = EFI_UDP4_SESSION_DATA{
                    SourceAddress: Ipv4Addr::unspecified().into(), // Unspecified to use the socket's configured addr
                    SourcePort: 0, // zero to use the socket's configured port
                    DestinationAddress: (*addr.ip()).into(),
                    DestinationPort: addr.port(),
                };
                match self.udp4_socket.send_buf(buf, Some(&session_data)) {
                    Ok(s) => return Ok(s),
                    Err(e) => {
                        last_error = e;
                        continue;
                    },
                };
            }
        }

        Err(last_error)
    }

    pub fn set_read_timeout(&mut self, dur: Option<Duration>) -> Result<()> {
        self.udp4_socket.set_read_timeout(dur)
    }

    pub fn set_write_timeout(&mut self, dur: Option<Duration>) -> Result<()> {
        self.udp4_socket.set_write_timeout(dur)
    }

    pub fn read_timeout(&self) -> Result<Option<Duration>> {
        self.udp4_socket.read_timeout()
    }

    pub fn write_timeout(&self) -> Result<Option<Duration>> {
        self.udp4_socket.write_timeout()
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.udp4_socket.local_addr().map(|a| SocketAddr::V4(a))
    }

}

struct Timer {
    timeout: Option<Duration>,
    timer: Option<events::Timer>,
}

impl Timer {
    fn infinite() -> Self {
        Self { timeout: None, timer: None}
    }

    fn set_timeout(&mut self, timeout: Option<Duration>) -> Result<()> {
        match timeout {
            Some(timeout) => {
                    self.timeout = Some(timeout);
                    self.timer = Some(events::Timer::create(timeout, TimerSchedule::Relative, TimerState::Inactive, EventTpl::Notify)?);
            },
            None => {
                self.timeout = None;
                self.timer = None;
            },
        };
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        if let Some(ref mut timer) = self.timer {
            if let Some(timeout) = self.timeout {
                timer.set(timeout, TimerSchedule::Relative)?;
            }
        }
        Ok(())
    }

    fn is_expired(&self) -> Result<bool> {
        if let Some(ref timer) = self.timer {
            timer.is_signaled()
        } else {
            Ok(false)
        }
    }

    fn timeout(&self) -> Option<Duration> {
        self.timeout
    }
}

struct Udp4Socket {
    bs: *const EFI_BOOT_SERVICES,
    binding_protocol: *const EFI_SERVICE_BINDING_PROTOCOL,
    protocol: *const EFI_UDP4_PROTOCOL,
    device_handle: EFI_HANDLE,
    recv_token: EFI_UDP4_COMPLETION_TOKEN,
    send_token: EFI_UDP4_COMPLETION_TOKEN,
    read_timer: Timer,
    write_timer: Timer,
    bound_addr: SocketAddrV4, // This is the address that was passed to us to bind to. It's different from local_addr() because the OS might choose arbitrary port if 0 is passed in bound_addr
}

impl Udp4Socket {
    pub fn bind(addr: SocketAddrV4) -> Result<Self> {
        // Using unspecified remote IpAddr to indicate we're not connecting to any remote addr
        // Using 0 remote port to indicate we're not connecting to any remote port
        let remote_addr = SocketAddrV4::new(Ipv4Addr::unspecified(), 0);
        Self::bind_and_connect(addr, remote_addr)
    }

    fn bind_and_connect(local_addr: SocketAddrV4, remote_addr: SocketAddrV4) -> Result<Self> {
        // TODO: THIS IS A TEMPORARY HACK. WE ACTUALLY WANT TO MAKE THE COMMENTED OUT CODE BELOW WORK.
        let dhcp_config = pxebc::PxeBaseCodeProtocol::get_any()? // TODO: this is bullshit. We should use the PXE BC on the exact interface corresponding to supplied IP
                .ok_or_else(|| ::EfiError::from(::EfiErrorKind::DeviceError))?
                .cached_dhcp_config()?
                .ok_or_else(|| ::EfiError::from(::EfiErrorKind::DeviceError))?;
        let station_addr = if let IpAddr::V4(ip) = dhcp_config.ip() { ip.into() } else { EFI_IPv4_ADDRESS::zero() };
        let subnet_mask = if let IpAddr::V4(ip) = dhcp_config.subnet_mask() { ip.into() } else { EFI_IPv4_ADDRESS::zero() };

        // TODO this code is not working because:
        // a. We want to use UseDefaultAddress when the ip to bound to is unspecified.
        // b. However on hyper v using UseDefaultAddress leads to a never-terminating configure loop.
        // So we have to fix the hyper v problem before we can make this code work
        // let station_addr = *local_addr.ip();
        // let (subnet_mask, use_default_addr) = if station_addr.is_unspecified() {
        //     // If the station addr is unspecified then the subnet mask is unspecified as well
        //     (Ipv4Addr::unspecified(), true)
        // } else {
        //     // If not station addr is not unspecified then we locate the interface associated with this IP
        //     // and get its subnet mask
        //     // TODO: this shit doesn't work at all. Fix it.
        //     let matching_interface = ifconfig::interfaces()?.into_iter()
        //                                 .find(|i| i.station_address_ipv4() == station_addr)
        //                                 .ok_or_else(|| ::EfiError::from(::EfiErrorKind::DeviceError))?;
        //     (matching_interface.subnet_mask_ipv4(), false)
        // };

        let config = EFI_UDP4_CONFIG_DATA {
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
            StationAddress: station_addr.into(),
            SubnetMask: subnet_mask.into(),
            StationPort: local_addr.port(),
            RemoteAddress: (*remote_addr.ip()).into(),
            RemotePort: remote_addr.port(),
        };

        let mut socket = Udp4Socket {
            bs: system_table().BootServices,
            binding_protocol: ptr::null() as *const EFI_SERVICE_BINDING_PROTOCOL,
            protocol: ptr::null() as *const EFI_UDP4_PROTOCOL,
            device_handle: ptr::null() as EFI_HANDLE,
            recv_token: EFI_UDP4_COMPLETION_TOKEN::default(),
            send_token: EFI_UDP4_COMPLETION_TOKEN::default(),
            read_timer: Timer::infinite(),
            write_timer: Timer::infinite(),
            bound_addr: local_addr,
        };

        unsafe {
            ret_on_err!(((*socket.bs).CreateEvent)(EVT_NOTIFY_WAIT, TPL_CALLBACK, Some(empty_cb), ptr::null(), &mut socket.send_token.Event));
            ret_on_err!(((*socket.bs).CreateEvent)(EVT_NOTIFY_SIGNAL, TPL_NOTIFY, Some(common_cb), ptr::null(), &mut socket.recv_token.Event));

            ret_on_err!(((*socket.bs).LocateProtocol)(&EFI_UDP4_SERVICE_BINDING_PROTOCOL_GUID, ptr::null() as *const VOID, mem::transmute(&socket.binding_protocol)));
            ret_on_err!(((*socket.binding_protocol).CreateChild)(socket.binding_protocol, &mut socket.device_handle));
            ret_on_err!(((*socket.bs).OpenProtocol)(socket.device_handle,
                &EFI_UDP4_PROTOCOL_GUID,
                mem::transmute(&socket.protocol),
                image_handle(),
                ptr::null() as EFI_HANDLE,
                EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL)); // TODO: BY_HANDLE is used for applications. Drivers should use GET. Will we ever support drivers?
            let status = ((*socket.protocol).Configure)(socket.protocol, &config);
            if status == EFI_NO_MAPPING { // Wait until the IP configuration process (probably DHCP) has finished
                let mut ip_mode_data = EFI_IP4_MODE_DATA::new();
                loop {
                    // TODO: This becomes an infinite loop on some firmeware such as Hyper-v
                    // Figure out why and fix it.
                    ret_on_err!(((*socket.protocol).GetModeData)(socket.protocol, ptr::null_mut(), &mut ip_mode_data, ptr::null_mut(), ptr::null_mut()));
                    if ip_mode_data.IsConfigured == TRUE { break }
                }

                ret_on_err!(((*socket.protocol).Configure)(socket.protocol, &config));
            } else {
                ret_on_err!(status);
            }
        }

        // Copy in all routes from the DHCP config
        // TODO: This is faulty. Get the dhcp config specifically of the interface we're binding on
        let dhcp_config = pxebc::PxeBaseCodeProtocol::get_any()? // TODO: this is bullshit. We should use the PXE BC on the exact interface corresponding to supplied IP
                .ok_or_else(|| ::EfiError::from(::EfiErrorKind::DeviceError))?
                .cached_dhcp_config()?
                .ok_or_else(|| ::EfiError::from(::EfiErrorKind::DeviceError))?;
        let (subnet_addr, subnet_mask, gateway_addr) = form_default_route(&dhcp_config)?;
        unsafe {
            ret_on_err!(((*socket.protocol).Routes)(socket.protocol, FALSE, &subnet_addr, &subnet_mask, &gateway_addr));
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
        reset_op_done();
        ret_on_err!(unsafe { ((*self.protocol).Receive)(self.protocol, &self.recv_token) });

        self.read_timer.start()?;
        let read_succeeded = loop {
            let status = unsafe { ((*self.protocol).Poll)(self.protocol) };
            if status != EFI_SUCCESS  && status != EFI_NOT_READY { // EFI_NOT_READY merely means there's not data received on the socket yet. It does not indicate any kind of failure.
                return Err(status.into());
            }

            if op_done() {
                break true;
            } else if self.read_timer.is_expired()? {
                break false;
            }
        }; 

        if read_succeeded {
            let read_len: usize;
            unsafe {
                let read_data = (*self.recv_token.Packet.RxData).FragmentTable[0].FragmentBuffer as *const u8;
                read_len = (*self.recv_token.Packet.RxData).FragmentTable[0].FragmentLength as usize;
                if buf.len() < read_len {
                    return Err(EfiError::from(::ffi::EFI_INVALID_PARAMETER));
                }
                //TODO:Get rid of this copy
                ptr::copy(read_data, buf.as_mut_ptr(), read_len);
            }
            to_res(read_len, self.recv_token.Status)
        } else {
            ret_on_err!(unsafe { ((*self.protocol).Cancel)(self.protocol, &self.recv_token) }); // Must cancel the token. Otherwise the next read fails with ACCESS_DENIED
            Err(::EfiErrorKind::Timeout.into()) // TODO: check whether the std::UdpSocket returns a timeout error in this case or just Ok(0) and mimic its behaviour.
        }
    }

    fn send_buf(&mut self, buf: &[u8], session_data: Option<&EFI_UDP4_SESSION_DATA>) -> Result<usize> {
        let fragment_data = EFI_UDP4_FRAGMENT_DATA {
            FragmentLength: buf.len() as UINT32,
            FragmentBuffer: buf.as_ptr() as *const VOID
        };

        let send_data = EFI_UDP4_TRANSMIT_DATA {
            UdpSessionData: if session_data.is_some() { session_data.unwrap() } else { ptr::null() as *const EFI_UDP4_SESSION_DATA },
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

    pub fn set_read_timeout(&mut self, dur: Option<Duration>) -> Result<()> {
        self.read_timer.set_timeout(dur)
    }

    pub fn set_write_timeout(&mut self, dur: Option<Duration>) -> Result<()> {
        self.write_timer.set_timeout(dur)
    }

    pub fn read_timeout(&self) -> Result<Option<Duration>> {
        Ok(self.read_timer.timeout())
    }

    pub fn write_timeout(&self) -> Result<Option<Duration>> {
        Ok(self.write_timer.timeout())
    }

    pub fn local_addr(&self) -> Result<SocketAddrV4> {
        let config = self.get_config_data()?;
        Ok(SocketAddrV4::new(config.StationAddress.into(), config.StationPort))
    }

    fn get_config_data(&self) -> Result<EFI_UDP4_CONFIG_DATA> {
        let mut config_data = EFI_UDP4_CONFIG_DATA::default();
        unsafe {
            ret_on_err!(((*self.protocol).GetModeData)(self.protocol, 
                &mut config_data,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut()));
        }
        Ok(config_data)
    }
}

impl Drop for Udp4Socket {
    fn drop(&mut self) {
        // TODO: add the code to panic when any of the below calls fail. (Could be difficult) but maybe we can trace something when we do that.
        unsafe {
            ((*self.protocol).Configure)(self.protocol, ptr::null());
            ((*self.bs).CloseEvent)(self.send_token.Event);
            ((*self.bs).CloseEvent)(self.recv_token.Event);
            ((*self.binding_protocol).DestroyChild)(self.binding_protocol, &mut self.device_handle);
        }
    }
}

fn extract_router_opt(dhcp_config: &DhcpConfig) -> Result<Ipv4Addr> {
    let ack_pkt = dhcp_config.dhcp_ack_packet().ok_or_else(|| ::EfiError::from(::EfiErrorKind::NotFound))?;
    let router_option = ack_pkt.dhcp_option(3)
        .ok_or_else(|| ::EfiError::from(::EfiErrorKind::DeviceError))?;
    let router_ip_buf = router_option.value()
        .ok_or_else(|| ::EfiError::from(::EfiErrorKind::DeviceError))?;

    let router_ip = Ipv4Addr::new(router_ip_buf[0], router_ip_buf[1], router_ip_buf[2], router_ip_buf[3]);
    Ok(router_ip)
}

fn form_default_route(dhcp_config: &DhcpConfig) -> Result<(EFI_IPv4_ADDRESS, EFI_IPv4_ADDRESS, EFI_IPv4_ADDRESS)> {
    let router_ip = extract_router_opt(&dhcp_config)?;

    // Unspecified subnet and subnet mask means this is a default route.
    let subnet_addr: EFI_IPv4_ADDRESS = Ipv4Addr::unspecified().into();
    let subnet_mask: EFI_IPv4_ADDRESS = Ipv4Addr::unspecified().into();
    let gateway_addr: EFI_IPv4_ADDRESS = router_ip.into();

    Ok((subnet_addr, subnet_mask, gateway_addr))
}