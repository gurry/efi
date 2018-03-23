// TODO; can we use AsRef and Borrow traits somehow during conversions across FFI boundary? 
// Or at least can we take inspiration from it?
use ::{
    Guid,
    Result,
    to_res,
    to_boolean,
    IpAddress,
    OpaqueEvent
};

use protocols::{
    simple_network::SimpleNetworkMode,
    managed_network::ManagedNetworkConfigData,
    ip4::Ip4ModeData
};

use ffi;
use ffi::{tcp4, udp4};
use ffi::{
    EFI_SUCCESS,
    EFI_EVENT,
    EFI_IPv4_ADDRESS,
    VOID,
    tcp4::{
        EFI_TCP4_PROTOCOL,
        EFI_TCP4_PROTOCOL_GUID,
        EFI_TCP4_CONNECTION_STATE,
        EFI_TCP4_CONNECTION_TOKEN, 
        EFI_TCP4_COMPLETION_TOKEN,
        EFI_TCP4_CLOSE_TOKEN,
        EFI_TCP4_IO_TOKEN,
        EFI_TCP4_CONFIG_DATA,
        EFI_TCP4_ACCESS_POINT,
        EFI_TCP4_OPTION,
        EFI_TCP4_TRANSMIT_DATA,
        EFI_TCP4_RECEIVE_DATA,
        EFI_TCP4_FRAGMENT_DATA,
        PacketUnion
    }
};


use core::{mem, ptr, slice, marker::PhantomData};
use protocols::Protocol;
use utils::{to_ptr, to_ptr_mut, Wrapper};

pub struct Tcp4Protocol<'a> {
    inner: EFI_TCP4_PROTOCOL,
    _p: PhantomData<&'a i32>
}

impl<'a> Protocol for Tcp4Protocol<'a> {
    type FfiType = EFI_TCP4_PROTOCOL;
    fn guid() -> Guid {
        EFI_TCP4_PROTOCOL_GUID
    }
}

impl<'a> Tcp4Protocol<'a> {
    // pub fn get_mode_data(&self,
    //     tcp4_state: Option<&mut Tcp4ConnectionState>,
    //     tcp4_config_data: Option<&mut Tcp4ConfigData>,
    //     ip4_mode_data: Option<&mut Ip4ModeData>,
    //     mnp_config_data: Option<&mut ManagedNetworkConfigData>,
    //     snp_mode_data: Option<&mut SimpleNetworkMode>) -> Result<()> {
    //         let status = (self.inner.GetModeData)(&self.inner, to_ptr_mut(tcp4_state), to_ptr_mut(tcp4_config_data), to_ptr_mut(ip4_mode_data), to_ptr_mut(mnp_config_data), to_ptr_mut(snp_mode_data));
    //         to_res((), status)
    // }


    fn get_config_data(&self) ->  Result<EFI_TCP4_CONFIG_DATA> {
        let mut data = EFI_TCP4_CONFIG_DATA {
            TypeOfService: 0,
            TimeToLive: 0,
            AccessPoint: EFI_TCP4_ACCESS_POINT {
                UseDefaultAddress: 1,
                StationAddress: EFI_IPv4_ADDRESS { Addr: [0, 0, 0, 0] },
                SubnetMask: EFI_IPv4_ADDRESS { Addr: [0, 0, 0, 0] },
                StationPort: 0,
                RemoteAddress: EFI_IPv4_ADDRESS { Addr: [0, 0, 0, 0] },
                RemotePort: 0,
                ActiveFlag: 1,
            },
            ControlOption: ptr::null()
        };

        let status = (self.inner.GetModeData)(&self.inner, ptr::null_mut(), &mut data, ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
        to_res(data, status)
    }

    pub fn configure(&mut self) -> Result<Tcp4ConfigBuilder> { // Todo: Returning a result causes awkwardness to user. Fix this
        Tcp4ConfigBuilder::init(self)
    }

    // pub fn routes(&self) -> EFI_TCP4_ROUTES {
    // }

    // TODO: Double check ownership works right
    pub fn connect(&mut self, token: &'a mut Tcp4ConnectionToken) -> Result<()> {
        let status = unsafe {
            (self.inner.Connect)(&self.inner, mem::transmute(token)) 
        };

        to_res((), status)
    }

    // pub fn accept(&self) -> EFI_TCP4_ACCEPT {
    // }

    // TODO: Double check ownership works right
    pub fn transmit(&mut self, token: &'a mut Tcp4IoTokenTx<'a>) -> Result<()> {
        let status = unsafe {
            (self.inner.Transmit)(&self.inner, mem::transmute(token)) 
        };

        to_res((), status)
    }

    pub fn receive(&mut self, token: &'a mut Tcp4IoTokenTx<'a>) -> Result<()> {
        let status = unsafe {
            (self.inner.Receive)(&self.inner, mem::transmute(token)) 
        };

        to_res((), status)
    }

    pub fn close(&mut self, close_token: &'a mut Tcp4CloseToken) -> Result<()> {
        let status = unsafe {
            (self.inner.Close)(&self.inner, mem::transmute(close_token)) 
        };

        to_res((), status)
    }

    pub fn cancel(&mut self, token: &'a mut Tcp4CompletionToken) -> Result<()> {
        let status = unsafe {
            (self.inner.Cancel)(&self.inner, mem::transmute(token)) 
        };

        to_res((), status)
    }

    pub fn poll(&mut self) -> Result<()> {
        let status =  (self.inner.Poll)(&self.inner);
        
        to_res((), status)
    }
}

pub struct Tcp4ConfigBuilder<'a> {
    config: EFI_TCP4_CONFIG_DATA,
    proto: &'a EFI_TCP4_PROTOCOL
}

impl<'a> Tcp4ConfigBuilder<'a> {
    // TODO: There's a flaw in init. What happens if the user inits the builder from teh current config
    // and then changes the current config somehow (maybe via another builder) and then tries to use this builder
    // It will probably cause problems because this builder has the old config data. Fix this.
    fn init(proto: &'a Tcp4Protocol) -> Result<Self> {
        let config = proto.get_config_data()?;
        Ok(Self { config, proto: &proto.inner })
    }
   
    pub fn type_of_service(&mut self, val: u8) -> &mut Self {
        self.config.TypeOfService = val;
        self
    }

    pub fn ttl(&mut self, val: u8) -> &mut Self {
        self.config.TimeToLive = val;
        self
    }

    pub fn use_default_address(&mut self, val: bool) -> &mut Self {
        self.config.AccessPoint.UseDefaultAddress = if val { 1 } else { 0 };
        self
    }

    // TODO: we must have have different types for IP4 and IP6 addresses and use IP4 address type here
    pub fn station_address(&mut self, val: IpAddress) -> &mut Self {
        self.config.AccessPoint.StationAddress = unsafe { val.v4 };
        self
    }

    pub fn subnet_mask(&mut self, val: IpAddress) -> &mut Self { // TODO: Don't use IpAddress here. Use a dedicated type to denote a subnet mask
        self.config.AccessPoint.SubnetMask = unsafe { val.v4 };
        self
    }

    pub fn station_port(&mut self, val: u16) -> &mut Self {
        self.config.AccessPoint.StationPort = val;
        self
    }

    pub fn remote_address(&mut self, val: IpAddress) -> &mut Self {
        self.config.AccessPoint.RemoteAddress = unsafe { val.v4 };
        self
    }

    pub fn remote_port(&mut self, val: u16) -> &mut Self {
        self.config.AccessPoint.RemotePort = val;
        self
    }

    pub fn active_flag(&mut self, val: bool) -> &mut Self {
        self.config.AccessPoint.ActiveFlag = if val { 1 } else { 0 };
        self
    }

    pub fn control_option(&mut self, val: &'a Tcp4Option) -> &mut Self {
        self.config.ControlOption = val as *const EFI_TCP4_OPTION;
        self
    }

    pub fn apply(&mut self) -> Result<()> {
        let status = (self.proto.Configure)(self.proto, &self.config);
        to_res((), status)
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(usize)]
pub enum Tcp4ConnectionState {
    Closed = EFI_TCP4_CONNECTION_STATE::Tcp4StateClosed as usize,
    Listen = EFI_TCP4_CONNECTION_STATE::Tcp4StateListen as usize,
    SynSent = EFI_TCP4_CONNECTION_STATE::Tcp4StateSynSent as usize,
    SynReceived = EFI_TCP4_CONNECTION_STATE::Tcp4StateSynReceived as usize,
    Established = EFI_TCP4_CONNECTION_STATE::Tcp4StateEstablished as usize,
    FinWait1 = EFI_TCP4_CONNECTION_STATE::Tcp4StateFinWait1 as usize,
    FinWait2 = EFI_TCP4_CONNECTION_STATE::Tcp4StateFinWait2 as usize,
    Closing = EFI_TCP4_CONNECTION_STATE::Tcp4StateClosing as usize,
    TimeWait = EFI_TCP4_CONNECTION_STATE::Tcp4StateTimeWait as usize,
    CloseWait = EFI_TCP4_CONNECTION_STATE::Tcp4StateCloseWait as usize,
    LastAck = EFI_TCP4_CONNECTION_STATE::Tcp4StateLastAck as usize
}

// TODO: This is a temp situation. Figure out the right way to code the below two structs
// keeping in mind that the user may have to selectively set the fields of these struct.
pub type Tcp4AccessPoint = EFI_TCP4_ACCESS_POINT;
pub type Tcp4Option = EFI_TCP4_OPTION;

#[repr(C)]
pub struct Tcp4ConfigData<'a> {
    inner: EFI_TCP4_CONFIG_DATA,
    control_option: &'a Tcp4Option
}

impl<'a> Wrapper for Tcp4ConfigData<'a> {
    type Inner = EFI_TCP4_CONFIG_DATA;
    fn inner_ptr(&self) -> *const Self::Inner {
        &(self.inner) as *const EFI_TCP4_CONFIG_DATA
    }
}

impl<'a> Tcp4ConfigData<'a> {
    pub fn new(type_of_service: u8, time_to_live: u8, access_point: Tcp4AccessPoint, control_option: &'a Tcp4Option) -> Self { // TODO: Tcp4AccessPoint is being copied. Find a way to avoid this copy
        Self {
            inner: EFI_TCP4_CONFIG_DATA {
                TypeOfService: type_of_service,
                TimeToLive: time_to_live,
                AccessPoint: access_point,
                ControlOption: control_option as *const EFI_TCP4_OPTION
            },
            control_option
        }
    }

    pub fn type_of_service(&self) -> u8 {
        self.inner.TypeOfService
    }

    pub fn time_to_live(&self) -> u8 {
        self.inner.TimeToLive
    }

    pub fn access_point(&self) -> &Tcp4AccessPoint {
        unsafe { mem::transmute(&self.inner.AccessPoint) }
    }

    pub fn control_option(&self) -> &Tcp4Option {
        self.control_option
    }
}

#[repr(C)]
pub struct Tcp4ConnectionToken(EFI_TCP4_CONNECTION_TOKEN); 
impl_wrapper!(Tcp4ConnectionToken, EFI_TCP4_CONNECTION_TOKEN); 

impl Tcp4ConnectionToken {
    pub fn new() -> Self {
        Tcp4ConnectionToken(EFI_TCP4_CONNECTION_TOKEN {
            CompletionToken: EFI_TCP4_COMPLETION_TOKEN {
                Event: ptr::null() as EFI_EVENT,
                Status: EFI_SUCCESS
            }
        })
    }

    pub fn completion_token(&self) -> &Tcp4CompletionToken {
        unsafe { mem::transmute(&self.0.CompletionToken) }
    }
}

#[repr(C)]
pub struct Tcp4CompletionToken(EFI_TCP4_COMPLETION_TOKEN); 
impl_wrapper!(Tcp4CompletionToken, EFI_TCP4_COMPLETION_TOKEN); 

impl Tcp4CompletionToken {
    pub fn event(&self) -> &OpaqueEvent {
        unsafe { mem::transmute(self.0.Event) }
    }

    pub fn status(&self) -> Tcp4CompletionTokenStatus {
        unsafe { mem::transmute(self.0.Status) }
    }
}

#[derive(Debug, Fail, Copy, Clone)]
#[repr(usize)]
pub enum Tcp4CompletionTokenStatus {
    #[fail(display = "The active open succeeded and the instance is in Tcp4StateEstablished")]
    Success = ffi::EFI_SUCCESS,
    #[fail(display = "The connect failed because the connection was reset either by instance itself or communication peer")]
    ConnectionReset = ffi::tcp4::EFI_CONNECTION_RESET,
    #[fail(display = "The connect failed because this connection was initiated with an active open and the connection was refused")]
    ConnectionRefused = ffi::tcp4::EFI_CONNECTION_REFUSED,
    #[fail(display = "The active open was aborted")]
    Aborted = ffi::EFI_ABORTED,
    #[fail(display = "The connection establishment timer expired and no more specific information is available")]
    Timeout = ffi::EFI_TIMEOUT,
    #[fail(display = "The active open failed because an ICMP network unreachable error was received")]
    NetworkUnreachable = ffi::udp4::EFI_NETWORK_UNREACHABLE,
    #[fail(display = "The active open failed because an ICMP host unreachable error was received")]
    HostUnreachable = ffi::udp4::EFI_HOST_UNREACHABLE,
    #[fail(display = "The active open failed because an ICMP protocol unreachable error was received")]
    ProtocolUnreachable = ffi::udp4::EFI_PROTOCOL_UNREACHABLE,
    #[fail(display = "The connection establishment timer timed out and an ICMP port unreachable error was received")]
    PortUnreachable = ffi::udp4::EFI_PORT_UNREACHABLE,
    #[fail(display = "The connection establishment timer timed out and some other ICMP error was received")]
    IcmpError = ffi::EFI_ICMP_ERROR,
    #[fail(display = "An unexpected system or network error occurred")]
    DeviceError = ffi::EFI_DEVICE_ERROR
}

#[repr(C)]
pub struct Tcp4CloseToken(EFI_TCP4_CLOSE_TOKEN);

impl Tcp4CloseToken {
    pub fn new(completion_token: Tcp4CompletionToken, abort_on_close: bool) -> Self {
        Tcp4CloseToken(
            EFI_TCP4_CLOSE_TOKEN {
                CompletionToken:  unsafe { mem::transmute(completion_token) },
                AbortOnClose: if abort_on_close { 1 } else { 0 }
            })
    }
    pub fn completion_token(&self) -> &Tcp4CompletionToken {
        unsafe { mem::transmute(&self.0.CompletionToken) }
    }

    pub fn abort_on_close(&self) -> bool {
        self.0.AbortOnClose == 1
    }
}

#[repr(C)]
pub struct Tcp4IoTokenTx<'a>
{
    inner: EFI_TCP4_IO_TOKEN,
    tx_data: Tcp4TransmitData<'a>
}

impl<'a> Wrapper for Tcp4IoTokenTx<'a> {
    type Inner = EFI_TCP4_IO_TOKEN;
    fn inner_ptr(&self) -> *const Self::Inner {
        &(self.inner) as *const EFI_TCP4_IO_TOKEN
    }
}

impl<'a> Tcp4IoTokenTx<'a> {
    pub fn new(completion_token: Tcp4CompletionToken, tx_data: Tcp4TransmitData<'a>) -> Self {
        Tcp4IoTokenTx {
            inner: EFI_TCP4_IO_TOKEN {
                CompletionToken: unsafe { mem::transmute(completion_token) },
                Packet: PacketUnion { TxData: tx_data.inner_ptr() }
            },
            tx_data
        }
    }
}

#[repr(C)]
pub struct Tcp4TransmitData<'a>
{
    inner: EFI_TCP4_TRANSMIT_DATA,
    frag_table: &'a [Tcp4FragmentData<'a>] 
}

impl<'a> Wrapper for Tcp4TransmitData<'a> {
    type Inner = EFI_TCP4_TRANSMIT_DATA;
    fn inner_ptr(&self) -> *const Self::Inner {
        &(self.inner) as *const EFI_TCP4_TRANSMIT_DATA
    }
}

impl<'a> Tcp4TransmitData<'a> {
    pub fn new(push: bool, urgent: bool, frag_table: &'a[Tcp4FragmentData]) -> Self {
        Self {
            inner: EFI_TCP4_TRANSMIT_DATA {
                    Push: to_boolean(push),
                    Urgent: to_boolean(urgent),
                    DataLength: frag_table.iter().map(|f| f.fragment_buffer().len() as u32).sum(),
                    FragmentCount: frag_table.len() as u32, // TODO: is this cast safe?
                    FragmentTable: unsafe { mem::transmute(frag_table.as_ptr()) }
            },
            frag_table
        }
    }

    pub fn push(&self) -> bool {
        self.inner.Push == 1
    }
    
    pub fn urgent(&self) -> bool {
        self.inner.Urgent == 1
    }

    pub fn data_length(&self) -> u32 {
        self.inner.DataLength
    }

    pub fn fragment_table(&self) -> &[Tcp4FragmentData] {
        self.frag_table
    }
}

#[repr(C)]
pub struct Tcp4IoTokenRx<'a>
{
    inner: EFI_TCP4_IO_TOKEN,
    rx_data: Tcp4ReceiveData<'a>
}

impl<'a> Wrapper for Tcp4IoTokenRx<'a> {
    type Inner = EFI_TCP4_IO_TOKEN;
    fn inner_ptr(&self) -> *const Self::Inner {
        &(self.inner) as *const EFI_TCP4_IO_TOKEN
    }
}

impl<'a> Tcp4IoTokenRx<'a> {
    pub fn new(completion_token: Tcp4CompletionToken, rx_data: Tcp4ReceiveData<'a>) -> Self {
        Tcp4IoTokenRx {
            inner: EFI_TCP4_IO_TOKEN {
                CompletionToken: unsafe { mem::transmute(completion_token) },
                Packet: PacketUnion { RxData: rx_data.inner_ptr() }
            },
            rx_data
        }
    }
}

#[repr(C)]
pub struct Tcp4ReceiveData<'a>
{
    inner: EFI_TCP4_RECEIVE_DATA,
    frag_table: &'a [Tcp4FragmentData<'a>] 
}

impl<'a> Wrapper for Tcp4ReceiveData<'a> {
    type Inner = EFI_TCP4_RECEIVE_DATA;
    fn inner_ptr(&self) -> *const Self::Inner {
        &(self.inner) as *const EFI_TCP4_RECEIVE_DATA
    }
}

impl<'a> Tcp4ReceiveData<'a> {
    pub fn new(urgent: bool, frag_table: &'a[Tcp4FragmentData]) -> Self {
        Self {
            inner: EFI_TCP4_RECEIVE_DATA {
                    UrgentFlag: to_boolean(urgent),
                    DataLength: frag_table.iter().map(|f| f.fragment_buffer().len() as u32).sum(),
                    FragmentCount: frag_table.len() as u32, // TODO: is this cast safe?
                    FragmentTable: unsafe { mem::transmute(frag_table.as_ptr()) }
            },
            frag_table
        }
    }
    
    pub fn urgent(&self) -> bool {
        self.inner.UrgentFlag == 1
    }

    pub fn data_length(&self) -> u32 {
        self.inner.DataLength
    }

    pub fn fragment_table(&self) -> &[Tcp4FragmentData] {
        self.frag_table
    }
}

#[repr(C)]
pub struct Tcp4FragmentData<'a> {
    inner: EFI_TCP4_FRAGMENT_DATA,
    _p: PhantomData<&'a i32>
}

impl<'a> Wrapper for Tcp4FragmentData<'a> {
    type Inner = EFI_TCP4_FRAGMENT_DATA;
    fn inner_ptr(&self) -> *const Self::Inner {
        &(self.inner) as *const EFI_TCP4_FRAGMENT_DATA
    }
}


impl<'a> Tcp4FragmentData<'a> {
    pub fn new(fragment_buffer: &'a [u8]) -> Self {
        Self { 
            inner: EFI_TCP4_FRAGMENT_DATA { FragmentLength: fragment_buffer.len() as u32, FragmentBuffer: fragment_buffer.as_ptr() as *const VOID },
            _p: PhantomData
        }
    }

    pub fn fragment_buffer(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.inner.FragmentBuffer as *const u8, self.inner.FragmentLength as usize) }
    }
}