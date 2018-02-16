use ::{Result, Guid, EfiError, IpAddress, to_boolean, from_boolean};
use protocols::Protocol;
use ffi::{EFI_SUCCESS, UINT16};
use core::{slice, marker::PhantomData, mem};


use ::ffi::pxe::{
    EFI_PXE_BASE_CODE_PROTOCOL, 
    EFI_PXE_BASE_CODE_PROTOCOL_GUID, 
    EFI_PXE_BASE_CODE_MODE,
    EFI_PXE_BASE_CODE_DISCOVER_INFO, 
    EFI_PXE_BASE_CODE_SRVLIST
};

// pub struct EFI_PXE_BASE_CODE_PROTOCOL {
//     Revision: UINT64,
//     Start: EFI_PXE_BASE_CODE_START,
//     Stop: EFI_PXE_BASE_CODE_STOP,
//     Dhcp: EFI_PXE_BASE_CODE_DHCP,
//     Discover: EFI_PXE_BASE_CODE_DISCOVER,
//     Mtftp: EFI_PXE_BASE_CODE_MTFTP,
//     UdpWrite: EFI_PXE_BASE_CODE_UDP_WRITE,
//     UdpRead: EFI_PXE_BASE_CODE_UDP_READ,
//     SetIpFilter: EFI_PXE_BASE_CODE_SET_IP_FILTER,
//     Arp: EFI_PXE_BASE_CODE_ARP,
//     SetParameters: EFI_PXE_BASE_CODE_SET_PARAMETERS,
//     SetStationIp: EFI_PXE_BASE_CODE_SET_STATION_IP,
//     SetPackets: EFI_PXE_BASE_CODE_SET_PACKETS,
//     Mode: *const EFI_PXE_BASE_CODE_MODE,
// }

pub struct PxeBaseCodeProtocol(*const EFI_PXE_BASE_CODE_PROTOCOL);

impl Protocol for PxeBaseCodeProtocol {
    type FfiType = EFI_PXE_BASE_CODE_PROTOCOL;
    fn guid() -> Guid {
        EFI_PXE_BASE_CODE_PROTOCOL_GUID
    }
}

impl From<*const EFI_PXE_BASE_CODE_PROTOCOL> for PxeBaseCodeProtocol {
    fn from(raw_protocol: *const EFI_PXE_BASE_CODE_PROTOCOL) -> Self {
        PxeBaseCodeProtocol(raw_protocol)
    }
}

impl PxeBaseCodeProtocol {
    pub fn start(&self, use_ipv6: bool) -> Result<()> {
        let status = unsafe { ((*self.0).Start)(self.0, to_boolean(use_ipv6)) };

        match status {
            EFI_SUCCESS => Ok(()),
            s => Err(EfiError::from(s))
        }
    }

    pub fn stop(&self) -> Result<()> {
        let status = unsafe { ((*self.0).Stop)(self.0) };

        match status {
            EFI_SUCCESS => Ok(()),
            s => Err(EfiError::from(s))
        }
    }

    pub fn dhcp(&self, sort_offers: bool) -> Result<()> {
        let status = unsafe { ((*self.0).Dhcp)(self.0, to_boolean(sort_offers)) };

        match status {
            EFI_SUCCESS => Ok(()),
            s => Err(EfiError::from(s))
        }

    }

    pub fn discover(&self, boot_type: BootType, layer: u16, use_bis: bool, info: &DiscoverInfo) -> Result<()> {
        let layer_ptr = &layer as *const UINT16;
        let status = unsafe { ((*self.0).Discover)(self.0, mem::transmute(boot_type), layer_ptr, to_boolean(use_bis), info.ffi_type()) };

        match status {
            EFI_SUCCESS => Ok(()),
            s => Err(EfiError::from(s))
        }
    }

    pub fn mtftp() -> Result<()> {
        unimplemented!()
    }

    // TODO: some missing methods here
    pub fn mode() -> Result<()> {
        unimplemented!()
    }
}

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

pub struct DiscoverInfo<'a> {
    inner: EFI_PXE_BASE_CODE_DISCOVER_INFO,
    phantom: PhantomData<&'a EFI_PXE_BASE_CODE_SRVLIST>
}

impl<'a> DiscoverInfo<'a> {
    pub fn new(use_mcast: bool, use_bcast: bool, use_ucast: bool, must_use_list: bool, server_mcast_ip: IpAddress, srvlist: &'a[SrvListEntry]) -> Self {
        Self { 
            inner: EFI_PXE_BASE_CODE_DISCOVER_INFO {
                UseMCast: to_boolean(use_mcast), 
                UseBCast: to_boolean(use_bcast), 
                UseUCast: to_boolean(use_ucast), 
                MustUseList: to_boolean(must_use_list), 
                ServerMCastIp: server_mcast_ip, 
                IpCnt: srvlist.len() as u16, // TODO: can we replace this cast with something safer?
                SrvList: unsafe { mem::transmute(srvlist.as_ptr()) } // Here be dragons
            },
            phantom: PhantomData
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

    pub fn ip_cnt(&self) -> u16 {
        self.inner.IpCnt
    }

    pub fn srvlist(&self) -> &'a[SrvListEntry] {
        // TODO: can we replace the cast to usize in the line below with something safer?
        unsafe { slice::from_raw_parts(mem::transmute(self.inner.SrvList), self.inner.IpCnt as usize) } // Broken glass all over. Tread carefully!
    }

    unsafe fn ffi_type(&self) -> *const EFI_PXE_BASE_CODE_DISCOVER_INFO {
        &(self.inner) as *const EFI_PXE_BASE_CODE_DISCOVER_INFO 
    }
}

#[repr(C)]
pub struct SrvList {
    ptr: *const EFI_PXE_BASE_CODE_SRVLIST, 
    count: u32,
    curr_pos: u32
}

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

pub struct Mode(*const EFI_PXE_BASE_CODE_MODE);

impl Mode {
    pub fn started(&self) -> bool {
        unsafe { (*self.0).Started == 1 }
    }
    pub fn ipv6_available(&self) -> bool {
        unsafe { (*self.0).Ipv6Available == 1 }
    }
    pub fn ipv6_supported(&self) -> bool {
        unsafe { (*self.0).Ipv6Supported == 1 }
    }
    pub fn using_ipv6(&self) -> bool {
        unsafe { (*self.0).UsingIpv6 == 1 }
    }
    pub fn bis_supported(&self) -> bool {
        unsafe { (*self.0).BisSupported == 1 }
    }
    pub fn bis_detected(&self) -> bool {
        unsafe { (*self.0).BisDetected == 1 }
    }
    pub fn auto_arp(&self) -> bool {
        unsafe { (*self.0).AutoArp == 1 }
    }
    pub fn send_guid(&self) -> bool {
        unsafe { (*self.0).SendGUID == 1 }
    }
    pub fn dhcp_discover_valid(&self) -> bool {
        unsafe { (*self.0).DhcpDiscoverValid == 1 }
    }
    pub fn dhcp_ack_received(&self) -> bool {
        unsafe { (*self.0).DhcpAckReceived == 1 }
    }
    pub fn proxy_offer_received(&self) -> bool {
        unsafe { (*self.0).ProxyOfferReceived == 1 }
    }
    pub fn pxe_discover_valid(&self) -> bool {
        unsafe { (*self.0).PxeDiscoverValid == 1 }
    }
    pub fn pxe_reply_received(&self) -> bool {
        unsafe { (*self.0).PxeReplyReceived == 1 }
    }
    pub fn pxe_bis_reply_received(&self) -> bool {
        unsafe { (*self.0).PxeBisReplyReceived == 1 }
    }
    pub fn icmp_error_received(&self) -> bool {
        unsafe { (*self.0).IcmpErrorReceived == 1 }
    }
    pub fn tftp_error_received(&self) -> bool {
        unsafe { (*self.0).TftpErrorReceived == 1 }
    }
    pub fn make_callbacks(&self) -> bool {
        unsafe { (*self.0).MakeCallbacks == 1 }
    }
    pub fn ttl(&self) -> u8 {
        unsafe { (*self.0).TTL }
    }
    pub fn tos(&self) -> u8 {
        unsafe { (*self.0).ToS }
    }

    // pub fn StationIp(&self) -> IpAddress {
    //     unimplemented!()
    // }
    // pub fn SubnetMask(&self) -> IpAddress {
    //     unimplemented!()
    // }
    // pub fn DhcpDiscover(&self) -> EFI_PXE_BASE_CODE_PACKET {
    //     unimplemented!()
    // }
    // pub fn DhcpAck(&self) -> EFI_PXE_BASE_CODE_PACKET {
    //     unimplemented!()
    // }
    // pub fn ProxyOffer(&self) -> EFI_PXE_BASE_CODE_PACKET {
    //     unimplemented!()
    // }
    // pub fn PxeDiscover(&self) -> EFI_PXE_BASE_CODE_PACKET {
    //     unimplemented!()
    // }
    // pub fn PxeReply(&self) -> EFI_PXE_BASE_CODE_PACKET {
    //     unimplemented!()
    // }
    // pub fn PxeBisReply(&self) -> EFI_PXE_BASE_CODE_PACKET {
    //     unimplemented!()
    // }
    // pub fn IpFilter(&self) -> EFI_PXE_BASE_CODE_IP_FILTER {
    //     unimplemented!()
    // }
    // pub fn ArpCacheEntries(&self) -> u32 {
    //     unimplemented!()
    // }
    // pub fn ArpCache(&self) -> [EFI_PXE_BASE_CODE_ARP_ENTRY; EFI_PXE_BASE_CODE_MAX_ARP_ENTRIES] {
    //     unimplemented!()
    // }
    // pub fn RouteTableEntries(&self) -> u32 {
    //     unimplemented!()
    // }
    // pub fn RouteTable(&self) -> [EFI_PXE_BASE_CODE_ROUTE_ENTRY; EFI_PXE_BASE_CODE_MAX_ROUTE_ENTRIES] {
    //     unimplemented!()
    // }
    // pub fn IcmpError(&self) -> EFI_PXE_BASE_CODE_ICMP_ERROR {
    //     unimplemented!()
    // }
    // pub fn TftpError(&self) -> EFI_PXE_BASE_CODE_TFTP_ERROR {
    //     unimplemented!()
    // }
}
