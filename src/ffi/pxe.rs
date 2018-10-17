///  EFI PXE Base Code Protocol definitions, which is used to access PXE-compatible 
///  devices for network access and network booting.
///  This Protocol is introduced in EFI Specification 1.10.           

use core::fmt;

use ffi::base::{
    EFI_GUID, 
    EFI_IP_ADDRESS, 
    EFI_MAC_ADDRESS, 
    EFI_STATUS, 
    UINT8,
    UINT16,
    UINT32,
    UINT64,
    INT8,
    UINTN,
    BOOLEAN,
    VOID
};

pub const EFI_PXE_BASE_CODE_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0x03c4e603, 0xac28, 0x11d3, [0x9a, 0x2d, 0x00, 0x90, 0x27, 0x3f, 0xc1, 0x4d]);

#[repr(C)]
pub struct EFI_PXE_BASE_CODE_PROTOCOL {
    pub Revision: UINT64,
    pub Start: EFI_PXE_BASE_CODE_START,
    pub Stop: EFI_PXE_BASE_CODE_STOP,
    pub Dhcp: EFI_PXE_BASE_CODE_DHCP,
    pub Discover: EFI_PXE_BASE_CODE_DISCOVER,
    pub Mtftp: EFI_PXE_BASE_CODE_MTFTP,
    pub UdpWrite: EFI_PXE_BASE_CODE_UDP_WRITE,
    pub UdpRead: EFI_PXE_BASE_CODE_UDP_READ,
    pub SetIpFilter: EFI_PXE_BASE_CODE_SET_IP_FILTER,
    pub Arp: EFI_PXE_BASE_CODE_ARP,
    pub SetParameters: EFI_PXE_BASE_CODE_SET_PARAMETERS,
    pub SetStationIp: EFI_PXE_BASE_CODE_SET_STATION_IP,
    pub SetPackets: EFI_PXE_BASE_CODE_SET_PACKETS,
    pub Mode: *const EFI_PXE_BASE_CODE_MODE,
}

pub const EFI_PXE_BASE_CODE_MAX_ARP_ENTRIES: UINTN = 8;
pub const EFI_PXE_BASE_CODE_MAX_ROUTE_ENTRIES: UINTN = 8;

/// The data values in this structure are read-only and
/// are updated by the code that produces the
/// EFI_PXE_BASE_CODE_PROTOCOL functions. 
#[derive(Debug)]
#[repr(C)]
pub struct EFI_PXE_BASE_CODE_MODE {
    pub Started: BOOLEAN,
    pub Ipv6Available: BOOLEAN,
    pub Ipv6Supported: BOOLEAN,
    pub UsingIpv6: BOOLEAN,
    pub BisSupported: BOOLEAN,
    pub BisDetected: BOOLEAN,
    pub AutoArp: BOOLEAN,
    pub SendGUID: BOOLEAN,
    pub DhcpDiscoverValid: BOOLEAN,
    pub DhcpAckReceived: BOOLEAN,
    pub ProxyOfferReceived: BOOLEAN,
    pub PxeDiscoverValid: BOOLEAN,
    pub PxeReplyReceived: BOOLEAN,
    pub PxeBisReplyReceived: BOOLEAN,
    pub IcmpErrorReceived: BOOLEAN,
    pub TftpErrorReceived: BOOLEAN,
    pub MakeCallbacks: BOOLEAN,
    pub TTL: UINT8,
    pub ToS: UINT8,
    pub StationIp: EFI_IP_ADDRESS,
    pub SubnetMask: EFI_IP_ADDRESS,
    pub DhcpDiscover: EFI_PXE_BASE_CODE_PACKET,
    pub DhcpAck: EFI_PXE_BASE_CODE_PACKET,
    pub ProxyOffer: EFI_PXE_BASE_CODE_PACKET,
    pub PxeDiscover: EFI_PXE_BASE_CODE_PACKET,
    pub PxeReply: EFI_PXE_BASE_CODE_PACKET,
    pub PxeBisReply: EFI_PXE_BASE_CODE_PACKET,
    pub IpFilter: EFI_PXE_BASE_CODE_IP_FILTER,
    pub ArpCacheEntries: UINT32,
    pub ArpCache: [EFI_PXE_BASE_CODE_ARP_ENTRY; EFI_PXE_BASE_CODE_MAX_ARP_ENTRIES],
    pub RouteTableEntries: UINT32,
    pub RouteTable: [EFI_PXE_BASE_CODE_ROUTE_ENTRY; EFI_PXE_BASE_CODE_MAX_ROUTE_ENTRIES],
    pub IcmpError: EFI_PXE_BASE_CODE_ICMP_ERROR,
    pub TftpError: EFI_PXE_BASE_CODE_TFTP_ERROR,
}

pub type EFI_PXE_BASE_CODE_UDP_PORT = UINT16;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct EFI_PXE_BASE_CODE_DHCPV4_PACKET {
    pub BootpOpcode: UINT8,
    pub BootpHwType: UINT8,
    pub BootpHwAddrLen: UINT8,
    pub BootpGateHops: UINT8,
    pub BootpIdent: UINT32,
    pub BootpSeconds: UINT16,
    pub BootpFlags: UINT16,
    pub BootpCiAddr: [UINT8; 4],
    pub BootpYiAddr: [UINT8; 4],
    pub BootpSiAddr: [UINT8; 4],
    pub BootpGiAddr: [UINT8; 4],
    pub BootpHwAddr: [UINT8; 16],
    pub BootpSrvName: [UINT8; 64],
    pub BootpBootFile: [UINT8; 128],
    pub DhcpMagik: UINT32,
    //TODO:DhcpOptions is defined as [UINT8; 56] in the spec, but options don't fit in 56 bytes,
    //setting it to EFI_PXE_BASE_CODE_PACKET raw size (1472) minus sum of other fields (240)
    pub DhcpOptions: [UINT8; 1232],
}

impl fmt::Debug for EFI_PXE_BASE_CODE_DHCPV4_PACKET {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EFI_PXE_BASE_CODE_DHCPV4_PACKET: {{ ")?;
        write!(f, "BootpOpcode: {:?}, BootpHwType: {:?}, BootpHwAddrLen: {:?}, BootpGateHops: {:?}, BootpIdent: {:?}, BootpSeconds: {:?}, BootpFlags: {:?}", self.BootpOpcode, self.BootpHwType, self.BootpHwAddrLen, self.BootpGateHops, self.BootpIdent, self.BootpSeconds, self.BootpFlags)?;
        write!(f, ", BootpCiAddr: ")?;
        self.BootpCiAddr.fmt(f)?;
        write!(f, ", BootpYiAddr: ")?;
        self.BootpYiAddr.fmt(f)?;
        write!(f, ", BootpSiAddr: ")?;
        self.BootpSiAddr.fmt(f)?;
        write!(f, ", BootpGiAddr: ")?;
        self.BootpGiAddr.fmt(f)?;
        write!(f, ", BootpHwAddr: ")?;
        self.BootpHwAddr.fmt(f)?;
        write!(f, ", BootpSrvName: ")?;
        self.BootpSrvName.fmt(f)?;
        write!(f, ", BootpBootFile: ")?;
        self.BootpBootFile.fmt(f)?;
        write!(f, ", DhcpMagik: ")?;
        self.DhcpMagik.fmt(f)?;
        write!(f, ", DhcpOptions: ")?;
        self.DhcpOptions.fmt(f)?;
        write!(f, " }}")

    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct EFI_PXE_BASE_CODE_DHCPV6_PACKET {
    pub BitField: UINT32, // Contains both MessageType and TransactionId as bit fields
    pub DhcpOptions: [UINT8; 1024]
}

impl fmt::Debug for EFI_PXE_BASE_CODE_DHCPV6_PACKET {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EFI_PXE_BASE_CODE_DHCPV6_PACKET : {{ ")?;
        write!(f, "BitField: {:?}, DhcpOptions: ", self.BitField)?;
        self.DhcpOptions.fmt(f)?;
        write!(f, " }}")
    }
}

impl EFI_PXE_BASE_CODE_DHCPV6_PACKET {
    pub fn MessageType(&self) -> UINT8 {
        (self.BitField & 0xFF) as UINT8
    }

    pub fn TransactionId(&self) -> UINT32 {
        (self.BitField >> 8)
    }
}

#[repr(C)]
pub union EFI_PXE_BASE_CODE_PACKET {
    pub Raw: [UINT8; 1472],
    pub Dhcpv4: EFI_PXE_BASE_CODE_DHCPV4_PACKET,
    pub Dhcpv6: EFI_PXE_BASE_CODE_DHCPV6_PACKET,
}

impl fmt::Debug for EFI_PXE_BASE_CODE_PACKET {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { self.Raw.fmt(f) }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct TempStructIcmpErr {
    pub Identifier: UINT16,
    pub Sequence: UINT16,
}

#[repr(C)]
pub union TempUnionIcmpErr {
    pub reserved: UINT32,
    pub Mtu: UINT32,
    pub Pointer: UINT32,
    pub Echo: TempStructIcmpErr,
}

impl fmt::Debug for TempUnionIcmpErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TempUnionIcmpErr: {{ {:?} }}", unsafe { self.reserved })
    }
}


#[repr(C)]
pub struct EFI_PXE_BASE_CODE_ICMP_ERROR {
    pub Type: UINT8,
    pub Code: UINT8,
    pub Checksum: UINT16,
    pub u: TempUnionIcmpErr,
    pub Data: [UINT8; 494],
}

impl fmt::Debug for EFI_PXE_BASE_CODE_ICMP_ERROR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EFI_PXE_BASE_CODE_ICMP_ERROR: {{ Type: {:?}, Code: {:?}, Checksum: {:?}, u: {:?}, Data: ", self.Type, self.Code, self.Checksum, self.u)?;
        self.Data.fmt(f)?;
        write!(f, " }}")
    }
}

#[repr(C)]
pub struct EFI_PXE_BASE_CODE_TFTP_ERROR {
    pub ErrorCode: UINT8,
    pub ErrorString: [INT8; 127]
}

impl fmt::Debug for EFI_PXE_BASE_CODE_TFTP_ERROR   {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EFI_PXE_BASE_CODE_TFTP_ERROR: {{ ErrorCode: {:?}, ErrorString: ", self.ErrorCode)?;
        self.ErrorString.fmt(f)?;
        write!(f, " }}")
    }
}

pub const EFI_PXE_BASE_CODE_MAX_IPCNT: UINTN = 8;

#[derive(Debug)]
#[repr(C)]
pub struct EFI_PXE_BASE_CODE_IP_FILTER {
    pub Filters: UINT8,
    pub IpCnt: UINT8,
    pub reserved: UINT16,
    pub IpList: [EFI_IP_ADDRESS; EFI_PXE_BASE_CODE_MAX_IPCNT],
}

pub const EFI_PXE_BASE_CODE_IP_FILTER_STATION_IP: UINT32 = 0x0001;
pub const EFI_PXE_BASE_CODE_IP_FILTER_BROADCAST: UINT32 = 0x0002;
pub const EFI_PXE_BASE_CODE_IP_FILTER_PROMISCUOUS: UINT32 = 0x0004;
pub const EFI_PXE_BASE_CODE_IP_FILTER_PROMISCUOUS_MULTICAST: UINT32 = 0x0008;

#[derive(Debug)]
#[repr(C)]
pub struct EFI_PXE_BASE_CODE_ARP_ENTRY {
    pub IpAddr: EFI_IP_ADDRESS,
    pub MacAddr: EFI_MAC_ADDRESS,
}

#[derive(Debug)]
#[repr(C)]
pub struct EFI_PXE_BASE_CODE_ROUTE_ENTRY {
    pub IpAddr: EFI_IP_ADDRESS,
    pub SubnetMask: EFI_IP_ADDRESS,
    pub GwAddr: EFI_IP_ADDRESS,
}


pub const EFI_PXE_BASE_CODE_UDP_OPFLAGS_ANY_SRC_IP: UINT32 = 0x0001;
pub const EFI_PXE_BASE_CODE_UDP_OPFLAGS_ANY_SRC_PORT: UINT32 = 0x0002;
pub const EFI_PXE_BASE_CODE_UDP_OPFLAGS_ANY_DEST_IP: UINT32 = 0x0004;
pub const EFI_PXE_BASE_CODE_UDP_OPFLAGS_ANY_DEST_PORT: UINT32 = 0x0008;
pub const EFI_PXE_BASE_CODE_UDP_OPFLAGS_USE_FILTER: UINT32 = 0x0010;
pub const EFI_PXE_BASE_CODE_UDP_OPFLAGS_MAY_FRAGMENT: UINT32 = 0x0020;
#[allow(non_upper_case_globals)]
pub const DEFAULT_TTL: UINT32 = 16;
#[allow(non_upper_case_globals)]
pub const DEFAULT_ToS: UINT32 = 0;


pub type EFI_PXE_BASE_CODE_START = extern "win64" fn(
    This: *const EFI_PXE_BASE_CODE_PROTOCOL,
    UseIpv6: BOOLEAN) -> EFI_STATUS;
pub type EFI_PXE_BASE_CODE_STOP = extern "win64" fn(
    This: *const EFI_PXE_BASE_CODE_PROTOCOL) -> EFI_STATUS;
pub type EFI_PXE_BASE_CODE_DHCP = extern "win64" fn(
    This: *const EFI_PXE_BASE_CODE_PROTOCOL,
    SortOffers: BOOLEAN) -> EFI_STATUS;
pub type EFI_PXE_BASE_CODE_DISCOVER = extern "win64" fn(
    This: *const EFI_PXE_BASE_CODE_PROTOCOL,
    Type: UINT16,
    Layer: *const UINT16,
    UseBis: BOOLEAN,
    Info: *const EFI_PXE_BASE_CODE_DISCOVER_INFO) -> EFI_STATUS;


// Bootstrap Types
pub const EFI_PXE_BASE_CODE_BOOT_TYPE_BOOTSTRAP: UINT16 = 0;
pub const EFI_PXE_BASE_CODE_BOOT_TYPE_MS_WINNT_RIS: UINT16 = 1;
pub const EFI_PXE_BASE_CODE_BOOT_TYPE_INTEL_LCM: UINT16 = 2;
pub const EFI_PXE_BASE_CODE_BOOT_TYPE_DOSUNDI: UINT16 = 3;
pub const EFI_PXE_BASE_CODE_BOOT_TYPE_NEC_ESMPRO: UINT16 = 4;
#[allow(non_upper_case_globals)]
pub const EFI_PXE_BASE_CODE_BOOT_TYPE_IBM_WSoD: UINT16 = 5;
pub const EFI_PXE_BASE_CODE_BOOT_TYPE_IBM_LCCM: UINT16 = 6;
pub const EFI_PXE_BASE_CODE_BOOT_TYPE_CA_UNICENTER_TNG: UINT16 = 7;
pub const EFI_PXE_BASE_CODE_BOOT_TYPE_HP_OPENVIEW: UINT16 = 8;
pub const EFI_PXE_BASE_CODE_BOOT_TYPE_ALTIRIS_9: UINT16 = 9;
pub const EFI_PXE_BASE_CODE_BOOT_TYPE_ALTIRIS_10 : UINT16 = 10;
pub const EFI_PXE_BASE_CODE_BOOT_TYPE_ALTIRIS_11 : UINT16 = 11;
pub const EFI_PXE_BASE_CODE_BOOT_TYPE_NOT_USED_12 : UINT16 = 12;
pub const EFI_PXE_BASE_CODE_BOOT_TYPE_REDHAT_INSTALL : UINT16 = 13;
pub const EFI_PXE_BASE_CODE_BOOT_TYPE_REDHAT_BOOT : UINT16 = 14;
pub const EFI_PXE_BASE_CODE_BOOT_TYPE_REMBO : UINT16 = 15;
pub const EFI_PXE_BASE_CODE_BOOT_TYPE_BEOBOOT : UINT16 = 16;
//
// Values 17 through 32767 are reserved.
// Values 32768 through 65279 are for vendor use.
// Values 65280 through 65534 are reserved.
//
pub const EFI_PXE_BASE_CODE_BOOT_TYPE_PXETEST: UINT16 = 65535;
pub const EFI_PXE_BASE_CODE_BOOT_LAYER_MASK: UINT16 = 0x7FFF;
pub const EFI_PXE_BASE_CODE_BOOT_LAYER_INITIAL: UINT16 = 0x0000;


pub type EFI_PXE_BASE_CODE_MTFTP = extern "win64" fn(
    This: *const EFI_PXE_BASE_CODE_PROTOCOL,
    Operation: EFI_PXE_BASE_CODE_TFTP_OPCODE, 
    BufferPtr: *const VOID,
    Overwrite: BOOLEAN,
    BufferSize: *const UINT64,
    BlockSize: *const UINTN,
    ServerIp: *const EFI_IP_ADDRESS,
    Filename: *const UINT8,
    Info: *const EFI_PXE_BASE_CODE_MTFTP_INFO,
    DontUseBuffe: BOOLEAN,
) -> EFI_STATUS;

#[derive(Debug)]
#[repr(C)]
pub struct EFI_PXE_BASE_CODE_DISCOVER_INFO {
    pub UseMCast: BOOLEAN, 
    pub UseBCast: BOOLEAN, 
    pub UseUCast: BOOLEAN, 
    pub MustUseList: BOOLEAN, 
    pub ServerMCastIp: EFI_IP_ADDRESS, 
    pub IpCnt: UINT16, 
    // The actual definition of the SrvList field is like this -> SrvList: [EFI_PXE_BASE_CODE_SRVLIST; 1],
    // but we changed it to the pointer definition below because it's easier to deal with during FFI.
    pub SrvList: *const EFI_PXE_BASE_CODE_SRVLIST
}

#[derive(Debug)]
#[repr(C)]
pub struct EFI_PXE_BASE_CODE_SRVLIST {
   pub Type: UINT16,
   pub AcceptAnyResponse: BOOLEAN,
   pub reserved: UINT8,
   pub IpAddr: EFI_IP_ADDRESS 
}


#[derive(Debug)]
#[repr(C)]
pub enum EFI_PXE_BASE_CODE_TFTP_OPCODE {
    EFI_PXE_BASE_CODE_TFTP_FIRST,
    EFI_PXE_BASE_CODE_TFTP_GET_FILE_SIZE,
    EFI_PXE_BASE_CODE_TFTP_READ_FILE,
    EFI_PXE_BASE_CODE_TFTP_WRITE_FILE,
    EFI_PXE_BASE_CODE_TFTP_READ_DIRECTORY,
    EFI_PXE_BASE_CODE_MTFTP_GET_FILE_SIZE,
    EFI_PXE_BASE_CODE_MTFTP_READ_FILE,
    EFI_PXE_BASE_CODE_MTFTP_READ_DIRECTORY,
    EFI_PXE_BASE_CODE_MTFTP_LAST
}

#[derive(Debug)]
#[repr(C)]
pub struct EFI_PXE_BASE_CODE_MTFTP_INFO {
    pub MCastIp: EFI_IP_ADDRESS, 
    pub CPort: EFI_PXE_BASE_CODE_UDP_PORT, 
    pub SPort: EFI_PXE_BASE_CODE_UDP_PORT,
    pub ListenTimeout: UINT16,
    pub TransmitTimeout: UINT16,
}


pub type EFI_PXE_BASE_CODE_UDP_WRITE = extern "win64" fn(
    This: *const EFI_PXE_BASE_CODE_PROTOCOL, 
    OpFlags: UINT16, 
    DestIp: *const EFI_IP_ADDRESS,
    DestPort: *const EFI_PXE_BASE_CODE_UDP_PORT,
    GatewayIp: *const EFI_IP_ADDRESS, 
    SrcIp: *const EFI_IP_ADDRESS, 
    SrcPort: *const EFI_PXE_BASE_CODE_UDP_PORT, 
    HeaderSize: *const UINTN, 
    HeaderPtr: *const VOID, 
    BufferSize: *const UINTN, 
    BufferPt: *const VOID 
) -> EFI_STATUS;

pub type EFI_PXE_BASE_CODE_UDP_READ = extern "win64" fn(
    This: *const EFI_PXE_BASE_CODE_PROTOCOL, 
    OpFlags: UINT16, 
    DestIp: *const EFI_IP_ADDRESS,
    DestPort: *const EFI_PXE_BASE_CODE_UDP_PORT,
    SrcIp: *const EFI_IP_ADDRESS, 
    SrcPort: *const EFI_PXE_BASE_CODE_UDP_PORT, 
    HeaderSize: *const UINTN, 
    HeaderPtr: *const VOID, 
    BufferSize: *const UINTN, 
    BufferPtr: *const VOID
) -> EFI_STATUS;


pub type EFI_PXE_BASE_CODE_SET_IP_FILTER = extern "win64" fn(
    This: *const EFI_PXE_BASE_CODE_PROTOCOL, 
    NewFilter: *const EFI_PXE_BASE_CODE_IP_FILTER
) -> EFI_STATUS;

pub type EFI_PXE_BASE_CODE_ARP = extern "win64" fn(
    This: *const EFI_PXE_BASE_CODE_PROTOCOL, 
    IpAddr: *const EFI_IP_ADDRESS, 
    MacAddr: *const EFI_MAC_ADDRESS
) -> EFI_STATUS;


pub type EFI_PXE_BASE_CODE_SET_PARAMETERS = extern "win64" fn(
    This: *const EFI_PXE_BASE_CODE_PROTOCOL, 
    NewAutoArp: *const BOOLEAN, 
    NewSendGUID: *const BOOLEAN,
    NewTTL: *const UINT8, 
    NewToS: *const UINT8, 
    NewMakeCallback: *const BOOLEAN 
) -> EFI_STATUS; 


pub type EFI_PXE_BASE_CODE_SET_STATION_IP = extern "win64" fn(
    This: *const EFI_PXE_BASE_CODE_PROTOCOL, 
    NewStationIp: *const EFI_IP_ADDRESS,
    NewSubnetMask: *const EFI_IP_ADDRESS 
) -> EFI_STATUS;


pub type EFI_PXE_BASE_CODE_SET_PACKETS = extern "win64" fn(
    This: *const EFI_PXE_BASE_CODE_PROTOCOL, 
    NewDhcpDiscoverValid: *const BOOLEAN,
    NewDhcpAckReceived: *const BOOLEAN,
    NewProxyOfferReceived: *const BOOLEAN,
    NewPxeDiscoverValid: *const BOOLEAN, 
    NewPxeReplyReceived: *const BOOLEAN,
    NewPxeBisReplyReceived: *const BOOLEAN,
    NewDhcpDiscover: *const EFI_PXE_BASE_CODE_PACKET,
    NewDhcpAck: *const EFI_PXE_BASE_CODE_PACKET,
    NewProxyOffer: *const EFI_PXE_BASE_CODE_PACKET,
    NewPxeDiscover: *const EFI_PXE_BASE_CODE_PACKET,
    NewPxeReply: *const EFI_PXE_BASE_CODE_PACKET,
    NewPxeBisReply: *const EFI_PXE_BASE_CODE_PACKET
) -> EFI_STATUS;