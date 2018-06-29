use ffi::{
    pxe::{
        EFI_PXE_BASE_CODE_PROTOCOL_GUID,
        EFI_PXE_BASE_CODE_PROTOCOL,
        EFI_PXE_BASE_CODE_MODE,
        EFI_PXE_BASE_CODE_DISCOVER_INFO, 
        EFI_PXE_BASE_CODE_SRVLIST,
        EFI_PXE_BASE_CODE_PACKET,
        EFI_PXE_BASE_CODE_DHCPV4_PACKET,
        EFI_PXE_BASE_CODE_DHCPV6_PACKET,
        EFI_PXE_BASE_CODE_IP_FILTER,
        EFI_PXE_BASE_CODE_ARP_ENTRY,
        EFI_PXE_BASE_CODE_ROUTE_ENTRY,
        EFI_PXE_BASE_CODE_ICMP_ERROR,
        EFI_PXE_BASE_CODE_TFTP_ERROR,
    },
    EFI_IP_ADDRESS,
    UINT16,
    BOOLEAN,
    VOID,
};

use {
    EfiError,
    EfiErrorKind,
    Result,
    to_boolean,
    from_boolean,
    to_res,
    system_table,
    net::{IpAddr, Ipv4Addr},
};

use core::{mem, ptr, default::Default};
use utils::{to_ptr, Wrapper, to_opt};
use alloc::String;

// TODO: THIS WHOLE MODULE NEEDS A COMPLETE OVERHAUL. 
// The API surface area needs to be complete redesigned including things like:
// - exposing dhcp and pxe input params wherever needed
// - renaming publicly visible to types to make them less awkward and truer to the nature of dhcp/pxe
// Also need to remove needless thin wrapper types like Mode etc. Will use raw types in their stead.


// TODO: eventually use more specific errors like below instead of 
// blindly returning that EfiError shit
// enum DhcpErrorKind {
//     NoOffer,
//     NoAck,
//     NoProxyOffer,
//     NetworkError,
//     Other
// }

// TODO: should we expose other packets like DhcpDiscover as well?
pub struct DhcpConfig<'a> {
    // TODO: DANGEROUS. We are carrying a ref to mode around which is backed by a raw ptr.
    // This ptr could change underneath our feet. Intead of returning references
    // in this struct's methods (like &Dhcpv4Packet) returned owned copies (i.e. Dhcpv4Packet).
    mode: &'a Mode,
}

impl<'a> DhcpConfig<'a> {
    pub fn ip(&self) -> IpAddr {
        IpAddr::V4(unsafe { self.mode.station_ip().v4 }.into())
    }

    pub fn subnet_mask(&self) -> IpAddr {
        IpAddr::V4(unsafe { self.mode.subnet_mask().v4}.into())
    }

    pub fn server_ip(&self) -> Option<IpAddr> {
        const DHCP_SERVER_IDENTFIER_OPTION: u8 = 54;
        let server_id_option = self.mode.dhcp_ack().as_dhcpv4().dhcp_options()
            .find(|o| o.code() == DHCP_SERVER_IDENTFIER_OPTION)?;

        let val = server_id_option.value()?;
        Some(Ipv4Addr::new(val[0], val[1], val[2], val[3]).into()) 
    } 

    pub fn dhcp_ack_packet(&self) -> &Dhcpv4Packet {
        self.mode.dhcp_ack().as_dhcpv4()
    }

    pub fn proxy_offer_packet(&self) -> Option<&Dhcpv4Packet> {
        if self.mode.proxy_offer_received() {
            Some(self.mode.proxy_offer().as_dhcpv4())
        } else {
            None
        }
    }
}

pub struct BootServerConfig<'a> {
    // TODO: DANGEROUS. We are carrying a ref to mode around which is backed by a raw ptr.
    // This ptr could change underneath our feet. Intead of returning references
    // in this struct's methods (like &Dhcpv4Packet) returned owned copies (i.e. Dhcpv4Packet).
    mode: &'a Mode,
    boot_file: String,
}

// TODO: should we expose other packets like PxeDiscover as well?
impl<'a> BootServerConfig<'a> {
    fn new (mode: &'a Mode) -> Result<Self> {
        // TODO: Is it safe to rely on proxy_offer() for getting boot file? Some question:
        // 1. If there are multiple proxy offers received, which one is recorded by UEFI in this field?
        // 2. What if there are zero proxy offers received and the bootfile was sent in the DHCP offer?
        if !mode.proxy_offer_received() {
            return Err(EfiErrorKind::ProtocolError.into());
        }

        let boot_file = String::from_utf8_lossy(mode.proxy_offer().as_dhcpv4().bootp_boot_file()).into_owned();
        Ok(Self { mode, boot_file })
    }

    pub fn boot_server_ip(&self) -> IpAddr {
        IpAddr::V4((*self.mode.pxe_reply().as_dhcpv4().bootp_si_addr()).into())
    } 

    pub fn boot_file(&self) -> &str {
        &self.boot_file
    }

    pub fn pxe_ack_packet(&self) -> &Dhcpv4Packet {
        self.mode.pxe_reply().as_dhcpv4()
    }
}

// TODO expose public apis to check if DHCP has already happned or not.
// Same for PXE
pub fn cached_dhcp_config<'a>() -> Result<Option<DhcpConfig<'a>>> {
    let pxe = locate_pxe_protocol()?;
    let mode = pxe.mode().ok_or_else::<EfiError, _>(|| EfiErrorKind::ProtocolError.into())?;

    if !mode.dhcp_ack_received() {
        return Ok(None)
    }

    Ok(Some(DhcpConfig { mode } ))
}

pub fn run_dhcp<'a>() -> Result<DhcpConfig<'a>> {
    // TODO: see tianocore-edk2\NetworkPkg\UefiPxeBcDxe\PxeBcBoot.c file to know to implement PXE sequence especially the method PxeBcDiscoverBootFile

    // TODO: we're using PxeBaseCodeProtocol for now for expediency,
    // but we want to get rid of it and its associated types below
    // and use raw ffi types here (except for packet wrapper types etc. we can keep thos)
    // It's too much work to maintain a the set of thin wrappers like this.
    let pxe = locate_pxe_protocol()?;

    let mode = pxe.mode().ok_or_else::<EfiError, _>(|| EfiErrorKind::ProtocolError.into())?;

    if !mode.started(){
        let use_ipv6 = false;
        pxe.start(use_ipv6)?;
    }

    let sort_offers = false; // TODO: may want to expose this out to the caller
    pxe.dhcp(sort_offers)?;

    if !mode.dhcp_ack_received() {
        return Err(EfiErrorKind::ProtocolError.into());
    }

    Ok(DhcpConfig { mode } )
}

// TODO: allow user to specify discovery options such as whether to do unicast, broadcast or multicast 
// and list of boot servers to use for unicast etc.
pub fn run_boot_server_discovery<'a>(_dhcp_config: &DhcpConfig) -> Result<BootServerConfig<'a>> {
    // We're requring the '_dhcp_config' argument above only to enforce the fact that user should've run DHCP first before calling this method.
    let info = DiscoverInfo::default();

    // TODO: we're using PxeBaseCodeProtocol for now for expediency,
    // but we want to get rid of it and its associated types below
    // and use raw ffi types here (except for packet wrapper types etc. we can keep thos)
    // It's too much work to maintain a the set of thin wrappers like this.
    let pxe = locate_pxe_protocol()?;
    pxe.discover(BootType::Bootstrap, BOOT_LAYER_INITIAL, false, Some(&info))?; 

    let mode = pxe.mode().ok_or_else::<EfiError, _>(|| EfiErrorKind::ProtocolError.into())?;

    Ok(BootServerConfig::new(mode)?)
}


fn locate_pxe_protocol<'a>() -> Result<&'a PxeBaseCodeProtocol> {
    let bs = (*system_table()).BootServices;
    let mut pxe_protocol: *const EFI_PXE_BASE_CODE_PROTOCOL = ptr::null_mut();
    unsafe {
        ret_on_err!(((*bs).LocateProtocol)(&EFI_PXE_BASE_CODE_PROTOCOL_GUID, ptr::null_mut() as *mut VOID, mem::transmute(&mut pxe_protocol)));
        Ok(mem::transmute(pxe_protocol))
    }
}

// TODO: This is a lot of boilerplate. Can we find a way to generate this code?
#[repr(C)]
pub struct PxeBaseCodeProtocol(EFI_PXE_BASE_CODE_PROTOCOL);
impl_wrapper!(PxeBaseCodeProtocol, EFI_PXE_BASE_CODE_PROTOCOL);

impl From<EFI_PXE_BASE_CODE_PROTOCOL> for PxeBaseCodeProtocol {
    fn from(raw_protocol: EFI_PXE_BASE_CODE_PROTOCOL) -> Self {
        PxeBaseCodeProtocol(raw_protocol)
    }
}

impl PxeBaseCodeProtocol {
    pub fn start(&self, use_ipv6: bool) -> Result<()> {
        let status = (self.0.Start)(&self.0, to_boolean(use_ipv6));
        to_res((), status)
    }

    pub fn stop(&self) -> Result<()> {
        let status = (self.0.Stop)(&self.0);
        to_res((), status)
    }

    pub fn dhcp(&self, sort_offers: bool) -> Result<()> {
        let status = (self.0.Dhcp)(&self.0, to_boolean(sort_offers));
        to_res((), status)
    }

    pub fn discover(&self, boot_type: BootType, layer: u16, use_bis: bool, info: Option<&DiscoverInfo>) -> Result<u16> {
        let layer_ptr = &layer as *const UINT16;
        let info_ptr = if let Some(info) = info { info.inner_ptr() } else { ptr::null() };

        let status = (self.0.Discover)(&self.0, unsafe { mem::transmute(boot_type) }, layer_ptr, to_boolean(use_bis), info_ptr);
        to_res(layer, status)
    }

    pub fn mtftp() -> Result<()> {
        unimplemented!()
    }

    pub fn set_packets(&self, 
        new_dhcp_discover_valid: Option<bool>, 
        new_dhcp_ack_received: Option<bool>, 
        new_proxy_offer_received: Option<bool>,
        new_pxe_discover_valid: Option<bool>,
        new_pxe_reply_received: Option<bool>,
        new_pxe_bis_reply_received: Option<bool>,
        new_dhcp_discover: Option<&Packet>,
        new_dhcp_ack:  Option<&Packet>,
        new_proxy_offer:  Option<&Packet>,
        new_pxe_discover:  Option<&Packet>,
        new_pxe_reply:  Option<&Packet>,
        new_pxe_bis_reply: Option<&Packet>) -> Result<()> {
            let true_ptr: *const BOOLEAN = &1;
            let false_ptr: *const BOOLEAN = &0;
            let map_bool_opt = |b: Option<bool>| b.map_or(ptr::null(), |v| if v { true_ptr } else { false_ptr });

            let status = (self.0.SetPackets)(&self.0,
                                map_bool_opt(new_dhcp_discover_valid),
                                map_bool_opt(new_dhcp_ack_received),
                                map_bool_opt(new_proxy_offer_received),
                                map_bool_opt(new_pxe_discover_valid),
                                map_bool_opt(new_pxe_reply_received),
                                map_bool_opt(new_pxe_bis_reply_received),
                                to_ptr(new_dhcp_discover),
                                to_ptr(new_dhcp_ack),
                                to_ptr(new_proxy_offer),
                                to_ptr(new_pxe_discover),
                                to_ptr(new_pxe_reply),
                                to_ptr(new_pxe_bis_reply));
            to_res((), status)
        } 

    // TODO: some missing methods here
    pub fn mode(&self) -> Option<&Mode> {
        to_opt(self.0.Mode)
    }
}

pub const BOOT_LAYER_INITIAL: u16 = 0;

#[derive(Debug)]
#[repr(u16)]
pub enum BootType {
    Bootstrap = 0,
    MsWinntRis = 1,
    IntelLcm = 2,
    Dosundi = 3,
    NecEsmpro = 4,
    IbmWsod = 5,
    IbmLccm = 6,
    CaUnicenterTng = 7,
    HpOpenview = 8,
    Altiris9 = 9,
    Altiris10 = 10,
    Altiris11 = 11,
    NotUsed12 = 12,
    RedhatInstall = 13,
    RedhatBoot = 14,
    Rembo = 15,
    Beoboot = 16,
    //
    // Values 17 through 32767 are reserved.
    // Values 32768 through 65279 are for vendor use.
    // Values 65280 through 65534 are reserved.
    //
    Pxetest = 65535,
}

#[derive(Debug)]
pub struct DiscoverInfo<'a> {
    inner: EFI_PXE_BASE_CODE_DISCOVER_INFO,
    srvlist: Option<&'a[SrvListEntry]>
}

impl<'a> Wrapper for DiscoverInfo<'a> {
    type Inner = EFI_PXE_BASE_CODE_DISCOVER_INFO;
    fn inner_ptr(&self) -> *const Self::Inner {
        &(self.inner) as *const EFI_PXE_BASE_CODE_DISCOVER_INFO
    }
}

// TODO: it seems SrvList as per UEFI must contain at least one parameter. Not documented anywhere but the OVMF code seems to expect it.
// So we may have to create a new type that enforces at least one element requirement instead of taking a ref to a plain array.
impl<'a> DiscoverInfo<'a> {
    pub fn new(use_mcast: bool, use_bcast: bool, use_ucast: bool, must_use_list: bool, server_mcast_ip: EFI_IP_ADDRESS, srvlist: Option<&'a[SrvListEntry]>) -> Self {
        Self { 
            inner: EFI_PXE_BASE_CODE_DISCOVER_INFO {
                UseMCast: to_boolean(use_mcast), 
                UseBCast: to_boolean(use_bcast), 
                UseUCast: to_boolean(use_ucast), 
                MustUseList: to_boolean(must_use_list), 
                ServerMCastIp: server_mcast_ip, 
                IpCnt: if let Some(slist) = srvlist { slist.len() as u16 } else { 0 }, // TODO: can we replace this cast with something safer?
                SrvList: unsafe { if let Some(slist) = srvlist { mem::transmute(slist.as_ptr()) } else { ptr::null()} } // Here be dragons
            },
            srvlist
        }
    }

    pub fn use_mcast(&self) -> bool {
        from_boolean(self.inner.UseMCast)
    }

    pub fn use_bcast(&self) -> bool {
        from_boolean(self.inner.UseBCast)
    }

    pub fn use_ucast(&self) -> bool {
        from_boolean(self.inner.UseUCast)
    }

    pub fn must_use_list(&self) -> bool {
        from_boolean(self.inner.MustUseList)
    }

    pub fn server_mcast_ip(&self) -> EFI_IP_ADDRESS {
        self.inner.ServerMCastIp
    }

    pub fn srvlist(&self) -> Option<&'a[SrvListEntry]> {
        self.srvlist
    }
}

impl<'a> Default for DiscoverInfo<'a> {
    fn default() -> Self {
        DiscoverInfo::new(false, true, false, false, EFI_IP_ADDRESS::zero(), Some(&DEFAULT_SRV_LIST_ENTRY)) // By default UEFI expects at least one srvlistentry. That's why we couldn't have used None for last parameter
    }
}

// Should've implemented Default trait for SrvListEntry and used that here intead of explicitly constructing SrvListEntry but function calls are not allowed on const expressions unfortunately :(. Not yet anyway.
const DEFAULT_SRV_LIST_ENTRY: [SrvListEntry; 1] = [SrvListEntry(EFI_PXE_BASE_CODE_SRVLIST { Type: 0, AcceptAnyResponse: 1, reserved: 0, IpAddr: EFI_IP_ADDRESS{ Addr: [0, 0, 0, 0]}})];

#[derive(Debug)]
#[repr(C)]
pub struct SrvListEntry(EFI_PXE_BASE_CODE_SRVLIST);
impl_wrapper!(SrvListEntry, EFI_PXE_BASE_CODE_SRVLIST);

impl SrvListEntry {
    pub fn new(type_: u16, accept_any_response: bool, reserved: u8, ip_addr: EFI_IP_ADDRESS) -> Self {
        SrvListEntry ( 
            EFI_PXE_BASE_CODE_SRVLIST { 
                Type: type_,
                AcceptAnyResponse: to_boolean(accept_any_response),
                reserved,
                IpAddr: ip_addr
            }
        )
    }
    // Had to append underscore in the name because 'type' is a Rust keyword
    pub fn type_(&self) -> u16 {
        self.0.Type
    }

    pub fn accept_any_response(&self) -> bool {
        from_boolean(self.0.AcceptAnyResponse)
    }

    pub fn reserved(&self) -> u8 {
        self.0.reserved
    }

    pub fn ip_addr(&self) -> EFI_IP_ADDRESS {
        self.0.IpAddr
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Mode(EFI_PXE_BASE_CODE_MODE);
impl_wrapper!(Mode, EFI_PXE_BASE_CODE_MODE);

impl Mode {
    pub fn started(&self) -> bool {
        self.0.Started == 1
    }

    pub fn ipv6_available(&self) -> bool {
        self.0.Ipv6Available == 1
    }

    pub fn ipv6_supported(&self) -> bool {
        self.0.Ipv6Supported == 1
    }

    pub fn using_ipv6(&self) -> bool {
        self.0.UsingIpv6 == 1
    }

    pub fn bis_supported(&self) -> bool {
        self.0.BisSupported == 1
    }

    pub fn bis_detected(&self) -> bool {
        self.0.BisDetected == 1
    }

    pub fn auto_arp(&self) -> bool {
        self.0.AutoArp == 1
    }

    pub fn send_guid(&self) -> bool {
        self.0.SendGUID == 1
    }

    pub fn dhcp_discover_valid(&self) -> bool {
        self.0.DhcpDiscoverValid == 1
    }

    pub fn dhcp_ack_received(&self) -> bool {
        self.0.DhcpAckReceived == 1
    }

    pub fn proxy_offer_received(&self) -> bool {
        self.0.ProxyOfferReceived == 1
    }

    pub fn pxe_discover_valid(&self) -> bool {
        self.0.PxeDiscoverValid == 1
    }

    pub fn pxe_reply_received(&self) -> bool {
        self.0.PxeReplyReceived == 1
    }

    pub fn pxe_bis_reply_received(&self) -> bool {
        self.0.PxeBisReplyReceived == 1
    }

    pub fn icmp_error_received(&self) -> bool {
        self.0.IcmpErrorReceived == 1
    }

    pub fn tftp_error_received(&self) -> bool {
        self.0.TftpErrorReceived == 1
    }

    pub fn make_callbacks(&self) -> bool {
        self.0.MakeCallbacks == 1
    }

    pub fn ttl(&self) -> u8 {
        self.0.TTL
    }

    pub fn tos(&self) -> u8 {
        self.0.ToS
    }

    pub fn station_ip(&self) -> EFI_IP_ADDRESS {
        self.0.StationIp
    }

    pub fn subnet_mask(&self) -> EFI_IP_ADDRESS {
        self.0.SubnetMask
    }
    
    pub fn dhcp_discover(&self) -> &Packet {
        unsafe { mem::transmute(&self.0.DhcpDiscover) }
    }

    pub fn dhcp_ack(&self) -> &Packet {
        unsafe { mem::transmute(&self.0.DhcpAck) }
    }

    pub fn proxy_offer(&self) -> &Packet {
        unsafe { mem::transmute(&self.0.ProxyOffer) }
    }

    pub fn pxe_discover(&self) -> &Packet {
        unsafe { mem::transmute(&self.0.PxeDiscover) }
    }
    
    pub fn pxe_reply(&self) -> &Packet {
        unsafe { mem::transmute(&self.0.PxeReply) }
    }
    
    pub fn pxe_bis_reply(&self) -> &Packet {
        unsafe { mem::transmute(&self.0.PxeBisReply) }
    }
    
    pub fn ip_filter(&self) -> &IpFilter {
        unsafe { mem::transmute(&self.0.IpFilter) }
    }
   
    pub fn arp_cache(&self) -> &[EFI_PXE_BASE_CODE_ARP_ENTRY] {
        &self.0.ArpCache[..self.0.ArpCacheEntries as usize] // TODO: is this cast to usize safe. Take another look
    }

    pub fn route_table(&self) -> &[EFI_PXE_BASE_CODE_ROUTE_ENTRY] {
        &self.0.RouteTable[..self.0.RouteTableEntries as usize] // TODO: is this cast to usize safe. Take another look
    }

    pub fn icmp_error(&self) -> &IcpmError {
        unsafe { mem::transmute(&self.0.IcmpError) }
    }

    pub fn tftp_error(&self) -> &TftpError {
        unsafe { mem::transmute(&self.0.TftpError) }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Packet(EFI_PXE_BASE_CODE_PACKET);
impl_wrapper!(Packet, EFI_PXE_BASE_CODE_PACKET);

impl Packet {
    pub fn raw(&self) -> &[u8; 1472] {
        unsafe { &self.0.Raw }
    }

    pub fn as_dhcpv4(&self) -> &Dhcpv4Packet {
        unsafe { mem::transmute(&self.0.Dhcpv4) }
    }

    pub fn as_dhcpv6(&self) -> Option<&Dhcpv6Packet> {
        unsafe { mem::transmute(&self.0.Dhcpv6) }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Dhcpv4Packet(EFI_PXE_BASE_CODE_DHCPV4_PACKET);
impl_wrapper!(Dhcpv4Packet, EFI_PXE_BASE_CODE_DHCPV4_PACKET);

impl Dhcpv4Packet {
    pub fn bootp_opcode(&self) -> u8 {
        self.0.BootpOpcode
    }

    pub fn bootp_hw_type(&self) -> u8 {
        self.0.BootpHwType
    }
    
    pub fn bootp_hw_addr_len(&self) -> u8 {
        self.0.BootpHwAddrLen
    }
    
    pub fn bootp_gate_hops(&self) -> u8 {
        self.0.BootpGateHops
    }
    
    pub fn bootp_ident(&self) -> u32 {
        self.0.BootpIdent
    }
    
    pub fn bootp_seconds(&self) -> u16 {
        self.0.BootpSeconds
    }
    
    pub fn bootp_flags(&self) -> u16 {
        self.0.BootpFlags
    }
    
    pub fn bootp_ci_addr(&self) -> &[u8; 4] {
        &self.0.BootpCiAddr
    }
    
    pub fn bootp_yi_addr(&self) -> &[u8; 4] {
        &self.0.BootpYiAddr
    }
    
    pub fn bootp_si_addr(&self) -> &[u8; 4] {
        &self.0.BootpSiAddr
    }
    
    pub fn bootp_gi_addr(&self) -> &[u8; 4] {
        &self.0.BootpGiAddr
    }
    
    pub fn bootp_hw_addr(&self) -> &[u8; 16] {
        &self.0.BootpHwAddr
    }
    
    pub fn bootp_srv_name(&self) -> &[u8; 64] {
        &self.0.BootpSrvName
    }
    
    pub fn bootp_boot_file(&self) -> &[u8; 128] {
        &self.0.BootpBootFile
    }
    
    pub fn dhcp_magik(&self) -> u32 {
        self.0.DhcpMagik
    }
    
    pub fn dhcp_options<'a>(&'a self) -> impl Iterator<Item=DhcpOption<'a>> { //&[u8; 56] {
        DhcpOptionIter { buf: &self.0.DhcpOptions }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Dhcpv6Packet(EFI_PXE_BASE_CODE_DHCPV6_PACKET);
impl_wrapper!(Dhcpv6Packet, EFI_PXE_BASE_CODE_DHCPV6_PACKET);

impl Dhcpv6Packet {
    pub fn bit_field(&self) -> u32 { // Contains both MessageType and TransactionId as bit fields
        self.0.BitField
    }
    
    // TODO: Do DHCPv6 options have the same format as DHCPv4 and therefore is it safe to use the same parsing code for them?
    pub fn dhcp_options<'a>(&'a self) -> impl Iterator<Item=DhcpOption<'a>> {
        DhcpOptionIter { buf: &self.0.DhcpOptions }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct IpFilter(EFI_PXE_BASE_CODE_IP_FILTER);
impl_wrapper!(IpFilter, EFI_PXE_BASE_CODE_IP_FILTER);

impl IpFilter {
    pub fn filters(&self) -> u8 {
        self.0.Filters
    }
    
    pub fn reserved(&self)  -> u16 {
        self.0.reserved
    }

    pub fn ip_list(&self) -> &[EFI_IP_ADDRESS] {
        &self.0.IpList[..self.0.IpCnt as usize]
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct ArpEntry(EFI_PXE_BASE_CODE_ARP_ENTRY);
impl_wrapper!(ArpEntry, EFI_PXE_BASE_CODE_ARP_ENTRY);

#[derive(Debug)]
#[repr(C)]
pub struct RouteEntry(EFI_PXE_BASE_CODE_ROUTE_ENTRY);
impl_wrapper!(RouteEntry, EFI_PXE_BASE_CODE_ROUTE_ENTRY);

#[derive(Debug)]
#[repr(C)]
pub struct IcpmError(EFI_PXE_BASE_CODE_ICMP_ERROR);
impl_wrapper!(IcpmError, EFI_PXE_BASE_CODE_ICMP_ERROR);

impl IcpmError {
    pub fn type_(&self) -> u8 {
        self.0.Type
    }

    pub fn code(&self) -> u8 {
        self.0.Code
    }

    pub fn checksum(&self) -> u16 {
        self.0.Checksum
    }

    // TODO: will do this later
    // pub fn u(&self) -> TempUnionIcmpErr {
    //     (*self.0).u
    // }

    pub fn data(&self) -> &[u8; 494] {
        &self.0.Data
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TftpError(EFI_PXE_BASE_CODE_TFTP_ERROR);
impl_wrapper!(TftpError, EFI_PXE_BASE_CODE_TFTP_ERROR);

impl TftpError {
    pub fn error_code(&self) -> u8 {
        self.0.ErrorCode
    }

    pub fn error_string(&self) -> &[i8; 127] {
        &self.0.ErrorString
    }
}
 
// TODO: Move all of this DHCP parsing code into a separate crate (called dhcparse) 
// so other applications, such as those for testing, can use it as well.
pub struct DhcpOption<'a> {
    code: u8,
    val: Option<&'a[u8]>,
}

impl<'a> DhcpOption<'a> {
    pub fn code(&self) -> u8 {
        self.code
    }

    pub fn value(&self) -> Option<&[u8]> {
        self.val
    }
}

pub struct DhcpOptionIter<'a> {
    buf: &'a[u8],
}

impl<'a> Iterator for DhcpOptionIter<'a> {
    type Item = DhcpOption<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let next_option_start_index = self.buf.iter().position(|b| *b != 0); // Skipping padding bytes
        self.buf = match next_option_start_index {
            Some(index) => &self.buf[index..],
            None => &[]
        };

        // The below would've been so simple with slice patterns, but they aren't close to stable yet :(
        const OPTION_END_CODE: u8 = 255;
        let (code, len, val) = {
            // as per RFC2132 a valid option must have code and length fields. 
            // Therefore it must have at least two elements otherwise we end here.
            // We also end if we have reached option end code
            if self.buf.len() < 2 || self.buf[0] == OPTION_END_CODE { 
                self.buf = &[]; // Assign to empty slice so that subsequent calls to this method also end up here
                return None;
            }

            let code = self.buf[0];
            let len = self.buf[1] as usize;
            let remaining = &self.buf[2..];
            if remaining.len() < len { // Length of remaining must be at least the value in the length field (otherwise how can we ready the value of the option)
                self.buf = &[]; // Assign to empty slice so that subsequent calls to this method also end up here
                return None;
            }
            
            let val = match remaining.len() {
                0 => None,
                _ => Some(&remaining[..len])
            };
            (code, len, val)
        };

        self.buf = &self.buf[(len + 2)..];

        Some(DhcpOption { code, val })
    }
}