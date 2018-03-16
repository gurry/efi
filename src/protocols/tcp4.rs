// TODO; can we use AsRef and Borrow traits somehow during conversions across FFI boundary? 
// Or at least can we take inspiration from it?
use ::{
    Guid,
    Result,
    to_res,
    to_boolean,
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
    tcp4::{
        EFI_TCP4_PROTOCOL,
        EFI_TCP4_PROTOCOL_GUID,
        EFI_TCP4_CONNECTION_STATE,
        EFI_TCP4_CONNECTION_TOKEN, 
        EFI_TCP4_COMPLETION_TOKEN,
        EFI_TCP4_IO_TOKEN,
        EFI_TCP4_CONFIG_DATA,
        EFI_TCP4_TRANSMIT_DATA,
        PacketUnion
    }
};


use core::{mem, ptr, slice};
use protocols::Protocol;
use utils::{to_ptr, to_ptr_mut, Wrapper};

pub struct Tcp4Protocol(EFI_TCP4_PROTOCOL);
impl_wrapper!(Tcp4Protocol, EFI_TCP4_PROTOCOL);

impl Protocol for Tcp4Protocol {
    type FfiType = EFI_TCP4_PROTOCOL;
    fn guid() -> Guid {
        EFI_TCP4_PROTOCOL_GUID
    }
}

impl Tcp4Protocol {
    pub fn get_mode_data( &self,
        tcp4_state: Option<&mut Tcp4ConnectionState>,
        tcp4_config_data: Option<&mut Tcp4ConfigData>,
        ip4_mode_data: Option<&mut Ip4ModeData>,
        mnp_config_data: Option<&mut ManagedNetworkConfigData>,
        snp_mode_data: Option<&mut SimpleNetworkMode>) -> Result<()> {
            let status = (self.0.GetModeData)(&self.0, to_ptr_mut(tcp4_state), to_ptr_mut(tcp4_config_data), to_ptr_mut(ip4_mode_data), to_ptr_mut(mnp_config_data), to_ptr_mut(snp_mode_data));
            to_res((), status)
    }

    pub fn configure(&self, tcp_config_data: Option<&Tcp4ConfigData>) -> Result<()> {
        let status = (self.0.Configure)(&self.0, to_ptr(tcp_config_data)) ;
        to_res((), status)
    }

    // pub fn routes(&self) -> EFI_TCP4_ROUTES {
    // }

    // // TODO: Questions around ownership. What if the caller destroys the token as soon as calling this method?
    // // Won't the EFI system try to write to the token after words?
    // pub fn connect(&self, token: &mut Tcp4ConnectionToken) -> Result<()> {
    //     let status = unsafe {
    //         (self.0.Connect)(&self.0, mem::transmute(token)) 
    //     };

    //     to_res((), status)
    // }

    // pub fn accept(&self) -> EFI_TCP4_ACCEPT {
    // }

    // // TODO: Major questions around ownership. What if the caller destroys the token as soon as calling this method?
    // // Won't the EFI system try to write to the token after words? How do we prevent this?
    // // There are also subfields hanging off the token (such as buffer pointers) which also raise the same questions, only more vehemently so.
    // pub fn transmit(&self,  token: &mut Tcp4IoToken) -> Result<()> {
    //     let status = unsafe {
    //         (self.0.Transmit)(&self.0, mem::transmute(token)) 
    //     };

    //     to_res((), status)
    // }

    // pub fn receive(&self) -> EFI_TCP4_RECEIVE {
    // }

    // pub fn close(&self) -> EFI_TCP4_CLOSE {
    // }

    // pub fn cancel(&self) -> EFI_TCP4_CANCEL {
    // }

    // pub fn poll(&self) -> EFI_TCP4_POLL {
    // }
}

#[repr(C)]
pub struct Tcp4ConnectionState(EFI_TCP4_CONNECTION_STATE); 
impl_wrapper!(Tcp4ConnectionState, EFI_TCP4_CONNECTION_STATE);

#[repr(C)]
pub struct Tcp4ConfigData(EFI_TCP4_CONFIG_DATA); 
impl_wrapper!(Tcp4ConfigData, EFI_TCP4_CONFIG_DATA); 

#[repr(C)]
pub struct Tcp4ConnectionToken(EFI_TCP4_CONNECTION_TOKEN); 
impl_wrapper!(Tcp4ConnectionToken, EFI_TCP4_CONNECTION_TOKEN); 

// impl Tcp4ConnectionToken {
//     pub fn new() -> Self {
//         Tcp4ConnectionToken(EFI_TCP4_CONNECTION_TOKEN {
//             CompletionToken: EFI_TCP4_COMPLETION_TOKEN {
//                 Event: ptr::null() as EFI_EVENT,
//                 Status: EFI_SUCCESS
//             }
//         })
//     }

//     pub fn completion_token(&self) -> &Tcp4CompletionToken {
//         unsafe { mem::transmute(&self.0.CompletionToken) }
//     }
// }

// #[repr(C)]
// pub struct Tcp4CompletionToken(EFI_TCP4_COMPLETION_TOKEN); 
// impl_wrapper!(Tcp4CompletionToken, EFI_TCP4_COMPLETION_TOKEN); 

// impl Tcp4CompletionToken {
//     pub fn event(&self) -> &OpaqueEvent {
//         unsafe { mem::transmute(self.0.Event) }
//     }

//     pub fn status(&self) -> Tcp4CompletionTokenStatus {
//         unsafe { mem::transmute(self.0.Status) }
//     }
// }

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

// #[repr(C)]
// pub struct Tcp4IoToken<'a>
// {
//     inner: EFI_TCP4_IO_TOKEN,
//     tx_data: Tcp4TransmitData<'a>
// }

// impl<'a> Wrapper for Tcp4IoToken<'a> {
//     type Inner = EFI_TCP4_IO_TOKEN;
//     fn inner_ptr(&self) -> *const Self::Inner {
//         &(self.inner) as *const EFI_TCP4_IO_TOKEN
//     }
// }

// impl<'a> Tcp4IoToken<'a> {
//     pub fn new(completion_token: Tcp4CompletionToken, tx_data: Tcp4TransmitData<'a>) -> Self {
//         Tcp4IoToken {
//             inner: EFI_TCP4_IO_TOKEN {
//                 CompletionToken: EFI_TCP4_COMPLETION_TOKEN {
//                     Event: ptr::null() as EFI_EVENT,
//                     Status: EFI_SUCCESS
//                 },
//                 Packet: PacketUnion { TxData: tx_data.inner_ptr() }
//             },
//             tx_data
//         }
//     }
// }

// #[repr(C)]
// pub struct Tcp4TransmitData<'a>
// {
//     inner: EFI_TCP4_TRANSMIT_DATA,
//     frag_table: &'a[&'a[u8]] 
// }

// impl<'a> Wrapper for Tcp4TransmitData<'a> {
//     type Inner = EFI_TCP4_TRANSMIT_DATA;
//     fn inner_ptr(&self) -> *const Self::Inner {
//         &(self.inner) as *const EFI_TCP4_TRANSMIT_DATA
//     }
// }

// impl<'a> Tcp4TransmitData<'a> {
//     pub fn new(push: bool, urgent: bool, frag_table: &'a[&[u8]]) -> Self {
//         Self {
//             inner: EFI_TCP4_TRANSMIT_DATA {
//                     Push: to_boolean(push),
//                     Urgent: to_boolean(urgent),
//                     DataLength: frag_table.iter().map(|f| f.len()).sum(),
//                     FragmentCount: frag_table.len() as u32, // TODO: is this cast safe?
//                     FragmentTable: frag_table.
//             },
//             frag_table
//         }
//     }

//     pub fn push(&self) -> bool {
//         self.0.Push == 1
//     }
    
//     pub fn urgent(&self) -> bool {
//         self.0.Urgent == 1
//     }

//     pub fn data_length(&self) -> u32 {
//         self.0.DataLength
//     }

//     pub fn fragment_table(&self) -> &[&[u8]] {
//         self.frag_table
//     }
// }

// pub struct Tcp4FragmentData(EFI_TCP4_FRAGMENT_DATA); 

// impl<'a> Wrapper for Tcp4FragmentData<'a> {
//     type Inner = EFI_TCP4_FRAGMENT_DATA;
//     fn inner_ptr(&self) -> *const Self::Inner {
//         &(self.inner) as *const EFI_TCP4_FRAGMENT_DATA
//     }
// }


// impl Tcp4FragmentData {
//     pub fn new(fragment_buffer: &[u8]) -> Self {
//         Self(EFI_TCP4_FRAGMENT_DATA { FragmentLength: fragment_buffer.len(), Frag})
//     }
//     pub fn fragment_buffer(&self) -> &[u8] {
//         unsafe { slice::from_raw_parts(self.0.FragmentBuffer, self.0.FragmentLength as usize) }
//     }
// }