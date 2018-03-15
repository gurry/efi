use::ffi::simple_network::EFI_SIMPLE_NETWORK_MODE;

#[repr(C)]
pub struct SimpleNetworkMode(EFI_SIMPLE_NETWORK_MODE); 
impl_wrapper!(SimpleNetworkMode, EFI_SIMPLE_NETWORK_MODE); 