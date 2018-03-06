
use ::{Result, Guid, IpAddress, to_boolean, from_boolean, to_res};
use protocols::Protocol;
use ffi::{UINT16, BOOLEAN};
use core::{mem, ptr, default::Default};


use ::ffi::pxe::{
    EFI_PXE_BASE_CODE_PROTOCOL, 
    EFI_PXE_BASE_CODE_PROTOCOL_GUID, 
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
};

// TODO: This is a lot of boilerplate. Can we find a way to generate this code?
#[repr(C)]
pub struct PxeBaseCodeProtocol(EFI_PXE_BASE_CODE_PROTOCOL);

impl Protocol for PxeBaseCodeProtocol {
    type FfiType = EFI_PXE_BASE_CODE_PROTOCOL;
    fn guid() -> Guid {
        EFI_PXE_BASE_CODE_PROTOCOL_GUID
    }
}

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
        let info_ptr = if let Some(info) = info { unsafe { info.ffi_type() } } else { ptr::null() };

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
            let map_packet_opt = |b: Option<&Packet>| b.map_or(ptr::null(), |v| unsafe { mem::transmute(v) });

            let status = (self.0.SetPackets)(&self.0,
                                map_bool_opt(new_dhcp_discover_valid),
                                map_bool_opt(new_dhcp_ack_received),
                                map_bool_opt(new_proxy_offer_received),
                                map_bool_opt(new_pxe_discover_valid),
                                map_bool_opt(new_pxe_reply_received),
                                map_bool_opt(new_pxe_bis_reply_received),
                                map_packet_opt(new_dhcp_discover),
                                map_packet_opt(new_dhcp_ack),
                                map_packet_opt(new_proxy_offer),
                                map_packet_opt(new_pxe_discover),
                                map_packet_opt(new_pxe_reply),
                                map_packet_opt(new_pxe_bis_reply));
            to_res((), status)
        } 

    // TODO: some missing methods here
    pub fn mode(&self) -> &Mode {
        unsafe { mem::transmute(&self.0.Mode) }
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


// TODO: it seems SrvList as per UEFI must contain at least one parameter. Not documented anywhere but the OVMF code seems to expect it.
// So we may have to create a new type that enforces at least one element requirement instead of taking a ref to a plain array.
impl<'a> DiscoverInfo<'a> {
    pub fn new(use_mcast: bool, use_bcast: bool, use_ucast: bool, must_use_list: bool, server_mcast_ip: IpAddress, srvlist: Option<&'a[SrvListEntry]>) -> Self {
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

    pub fn server_mcast_ip(&self) -> IpAddress {
        self.inner.ServerMCastIp
    }

    pub fn srvlist(&self) -> Option<&'a[SrvListEntry]> {
        self.srvlist
    }

    unsafe fn ffi_type(&self) -> *const EFI_PXE_BASE_CODE_DISCOVER_INFO {
        &(self.inner) as *const EFI_PXE_BASE_CODE_DISCOVER_INFO 
    }
}

impl<'a> Default for DiscoverInfo<'a> {
    fn default() -> Self {
        DiscoverInfo::new(false, true, false, false, IpAddress::zero(), None)
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct SrvListEntry(EFI_PXE_BASE_CODE_SRVLIST);

impl SrvListEntry {
    pub fn new(type_: u16, accept_any_response: bool, reserved: u8, ip_addr: IpAddress) -> Self {
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

    pub fn ip_addr(&self) -> IpAddress {
        self.0.IpAddr
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Mode(EFI_PXE_BASE_CODE_MODE);

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

    pub fn station_ip(&self) -> IpAddress {
        self.0.StationIp
    }

    pub fn subnet_mask(&self) -> IpAddress {
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
        unsafe { mem::transmute(&self.0.IpFilter)}
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

impl Packet {
    pub fn raw(&self) -> &[u8; 1472] {
        unsafe { &self.0.Raw }
    }

    pub fn as_dhcpv4(&self) -> &Dhcpv4Packet {
        unsafe { mem::transmute(&self.0.Dhcpv4) }
    }

    pub fn as_dhcpv6(&self) -> &Dhcpv6Packet {
        unsafe { mem::transmute(&self.0.Dhcpv6) }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Dhcpv4Packet(EFI_PXE_BASE_CODE_DHCPV4_PACKET);

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
    
    pub fn dhcp_options(&self) -> &[u8; 56] {
        &self.0.DhcpOptions
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Dhcpv6Packet(EFI_PXE_BASE_CODE_DHCPV6_PACKET);

impl Dhcpv6Packet {
    pub fn bit_field(&self) -> u32 { // Contains both MessageType and TransactionId as bit fields
        self.0.BitField
    }
    
    pub fn dhcp_options(&self) -> &[u8; 1024] {
        &self.0.DhcpOptions
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct IpFilter(EFI_PXE_BASE_CODE_IP_FILTER);

impl IpFilter {
    pub fn filters(&self) -> u8 {
        self.0.Filters
    }
    
    pub fn reserved(&self)  -> u16 {
        self.0.reserved
    }

    pub fn ip_list(&self) -> &[IpAddress] {
        &self.0.IpList[..self.0.IpCnt as usize]
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct ArpEntry(EFI_PXE_BASE_CODE_ARP_ENTRY);

#[derive(Debug)]
#[repr(C)]
pub struct RouteEntry(EFI_PXE_BASE_CODE_ROUTE_ENTRY);

#[derive(Debug)]
#[repr(C)]
pub struct IcpmError(EFI_PXE_BASE_CODE_ICMP_ERROR);

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

impl TftpError {
    pub fn error_code(&self) -> u8 {
        self.0.ErrorCode
    }

    pub fn error_string(&self) -> &[i8; 127] {
        &self.0.ErrorString
    }
}