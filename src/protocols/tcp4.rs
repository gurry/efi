use ::{
    Guid,
    Result,
    to_res,
};

use protocols::{
    simple_network::SimpleNetworkMode,
    managed_network::ManagedNetworkConfigData,
    ip4::Ip4ModeData
};

use ffi::tcp4::{
    EFI_TCP4_PROTOCOL,
    EFI_TCP4_PROTOCOL_GUID,
    EFI_TCP4_CONNECTION_STATE,
    EFI_TCP4_CONFIG_DATA
};

use core::{mem, ptr};
use protocols::Protocol;
use utils::{to_ptr, to_ptr_mut};

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
            let status = unsafe {
                (self.0.GetModeData)(mem::transmute(self), to_ptr_mut(tcp4_state), to_ptr_mut(tcp4_config_data), to_ptr_mut(ip4_mode_data), to_ptr_mut(mnp_config_data), to_ptr_mut(snp_mode_data))
            };

            to_res((), status)
    }

    pub fn configure(&self, tcp_config_data: Option<&Tcp4ConfigData>) -> Result<()> {
        let status = unsafe {
            (self.0.Configure)(mem::transmute(self), to_ptr(tcp_config_data)) 
        };

        to_res((), status)
    }

    // pub fn routes(&self) -> EFI_TCP4_ROUTES {
    // }

    // pub fn connect(&self) -> EFI_TCP4_CONNECT {
    // }

    // pub fn accept(&self) -> EFI_TCP4_ACCEPT {
    // }

    // pub fn transmit(&self) -> EFI_TCP4_TRANSMIT {
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