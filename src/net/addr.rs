use ffi::{EFI_IPv4_ADDRESS, EFI_IPv6_ADDRESS, EFI_IP_ADDRESS};
use core::{mem, fmt, iter, slice, option, cmp::Ordering};
use io;
use alloc::{string::String, vec::{self, Vec}};
use super::dns::lookup_host;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Ipv4Addr(EFI_IPv4_ADDRESS);

impl Ipv4Addr {
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Ipv4Addr(EFI_IPv4_ADDRESS {
            Addr: [a, b, c, d]
        })
    }

    pub fn localhost() -> Ipv4Addr {
        Ipv4Addr::new(127, 0, 0, 1)
    }

    pub fn unspecified() -> Ipv4Addr {
        Ipv4Addr::new(0, 0, 0, 0)
    }

    pub fn octets(&self) -> [u8; 4] {
        self.0.Addr
    }

    /// Returns [`true`] for the special 'unspecified' address (0.0.0.0).
    ///
    /// This property is defined in _UNIX Network Programming, Second Edition_,
    /// W. Richard Stevens, p. 891; see also [ip7].
    ///
    /// [ip7]: http://man7.org/linux/man-pages/man7/ip.7.html
    pub fn is_unspecified(&self) -> bool {
        self.0.Addr == [0, 0, 0, 0]
    }

    pub fn is_loopback(&self) -> bool {
        self.octets()[0] == 127
    }

    /// Returns [`true`] if this is a private address.
    ///
    /// The private address ranges are defined in [IETF RFC 1918] and include:
    ///
    ///  - 10.0.0.0/8
    ///  - 172.16.0.0/12
    ///  - 192.168.0.0/16
    ///
    /// [IETF RFC 1918]: https://tools.ietf.org/html/rfc1918
    pub fn is_private(&self) -> bool {
        match (self.octets()[0], self.octets()[1]) {
            (10, _) => true,
            (172, b) if b >= 16 && b <= 31 => true,
            (192, 168) => true,
            _ => false
        }
    }

    pub fn is_link_local(&self) -> bool {
        self.octets()[0] == 169 && self.octets()[1] == 254
    }

    /// Returns [`true`] if the address appears to be globally routable.
    /// See [iana-ipv4-special-registry][ipv4-sr].
    ///
    /// The following return false:
    ///
    /// - private address (10.0.0.0/8, 172.16.0.0/12 and 192.168.0.0/16)
    /// - the loopback address (127.0.0.0/8)
    /// - the link-local address (169.254.0.0/16)
    /// - the broadcast address (255.255.255.255/32)
    /// - test addresses used for documentation (192.0.2.0/24, 198.51.100.0/24 and 203.0.113.0/24)
    /// - the unspecified address (0.0.0.0)
    ///
    /// [ipv4-sr]: https://www.iana.org/assignments/iana-ipv4-special-registry/iana-ipv4-special-registry.xhtml
    pub fn is_global(&self) -> bool {
        !self.is_private() && !self.is_loopback() && !self.is_link_local() &&
        !self.is_broadcast() && !self.is_documentation() && !self.is_unspecified()
    }

    /// Returns [`true`] if this is a multicast address (224.0.0.0/4).
    ///
    /// Multicast addresses have a most significant octet between 224 and 239,
    /// and is defined by [IETF RFC 5771].
    ///
    /// [IETF RFC 5771]: https://tools.ietf.org/html/rfc5771
    pub fn is_multicast(&self) -> bool {
        self.octets()[0] >= 224 && self.octets()[0] <= 239
    }

    /// Returns [`true`] if this is a broadcast address (255.255.255.255).
    ///
    /// A broadcast address has all octets set to 255 as defined in [IETF RFC 919].
    ///
    /// [IETF RFC 919]: https://tools.ietf.org/html/rfc919
    pub fn is_broadcast(&self) -> bool {
        self.octets()[0] == 255 && self.octets()[1] == 255 &&
        self.octets()[2] == 255 && self.octets()[3] == 255
    }

    /// Returns [`true`] if this address is in a range designated for documentation.
    ///
    /// This is defined in [IETF RFC 5737]:
    ///
    /// - 192.0.2.0/24 (TEST-NET-1)
    /// - 198.51.100.0/24 (TEST-NET-2)
    /// - 203.0.113.0/24 (TEST-NET-3)
    ///
    /// [IETF RFC 5737]: https://tools.ietf.org/html/rfc5737
    pub fn is_documentation(&self) -> bool {
        match(self.octets()[0], self.octets()[1], self.octets()[2], self.octets()[3]) {
            (192, 0, 2, _) => true,
            (198, 51, 100, _) => true,
            (203, 0, 113, _) => true,
            _ => false
        }
    }

    /// Converts this address to an IPv4-compatible [IPv6 address].
    ///
    /// a.b.c.d becomes ::a.b.c.d
    pub fn to_ipv6_compatible(&self) -> Ipv6Addr {
        Ipv6Addr::new(0, 0, 0, 0, 0, 0,
                      ((self.octets()[0] as u16) << 8) | self.octets()[1] as u16,
                      ((self.octets()[2] as u16) << 8) | self.octets()[3] as u16)
    }

    /// Converts this address to an IPv4-mapped [IPv6 address].
    ///
    /// a.b.c.d becomes ::ffff:a.b.c.d
    ///
    pub fn to_ipv6_mapped(&self) -> Ipv6Addr {
        Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff,
                      ((self.octets()[0] as u16) << 8) | self.octets()[1] as u16,
                      ((self.octets()[2] as u16) << 8) | self.octets()[3] as u16)
    }
}

impl From<EFI_IPv4_ADDRESS> for Ipv4Addr {
    fn from(val: EFI_IPv4_ADDRESS) -> Self {
        Ipv4Addr(val)
    }
}

impl From<Ipv4Addr > for EFI_IPv4_ADDRESS {
    fn from(val: Ipv4Addr) -> Self {
        val.0
    }
}

impl From<u32> for Ipv4Addr {
    fn from(ip: u32) -> Ipv4Addr {
        Ipv4Addr::new((ip >> 24) as u8, (ip >> 16) as u8, (ip >> 8) as u8, ip as u8)
    }
}

impl From<Ipv4Addr> for u32 {
    fn from(ip: Ipv4Addr) -> u32 {
        let ip = ip.octets();
        ((ip[0] as u32) << 24) + ((ip[1] as u32) << 16) + ((ip[2] as u32) << 8) + (ip[3] as u32)
    }
}

impl From<[u8; 4]> for Ipv4Addr {
    fn from(octets: [u8; 4]) -> Ipv4Addr {
        Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3])
    }
}

impl PartialOrd for Ipv4Addr {
    fn partial_cmp(&self, other: &Ipv4Addr) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ipv4Addr {
    fn cmp(&self, other: &Ipv4Addr) -> Ordering {
        let self_u32 = u32::from(*self);
        let other_u32 = u32::from(*other);
        self_u32.cmp(&other_u32)
    }
}

impl PartialOrd<Ipv4Addr> for IpAddr {
    fn partial_cmp(&self, other: &Ipv4Addr) -> Option<Ordering> {
        match *self {
            IpAddr::V4(ref v4) => v4.partial_cmp(other),
            IpAddr::V6(_) => Some(Ordering::Greater),
        }
    }
}

impl PartialOrd<IpAddr> for Ipv4Addr {
    fn partial_cmp(&self, other: &IpAddr) -> Option<Ordering> {
        match *other {
            IpAddr::V4(ref v4) => self.partial_cmp(v4),
            IpAddr::V6(_) => Some(Ordering::Less),
        }
    }
}

impl PartialEq<Ipv4Addr> for IpAddr {
    fn eq(&self, other: &Ipv4Addr) -> bool {
        match *self {
            IpAddr::V4(ref v4) => v4 == other,
            IpAddr::V6(_) => false,
        }
    }
}

impl PartialEq<IpAddr> for Ipv4Addr {
    fn eq(&self, other: &IpAddr) -> bool {
        match *other {
            IpAddr::V4(ref v4) => self == v4,
            IpAddr::V6(_) => false,
        }
    }
}

impl fmt::Display for Ipv4Addr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let octets = self.octets();
        write!(fmt, "{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3])
    }
}

#[derive(Copy, PartialEq, Eq, Clone, Hash, Debug)]
pub enum Ipv6MulticastScope {
    InterfaceLocal,
    LinkLocal,
    RealmLocal,
    AdminLocal,
    SiteLocal,
    OrganizationLocal,
    Global
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Ipv6Addr(EFI_IPv6_ADDRESS);

impl Ipv6Addr {
    pub fn new(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> Self {
        Ipv6Addr(EFI_IPv6_ADDRESS {
            Addr: unsafe { mem::transmute([a, b, c, d, e, f, g, h]) } // Transmuting from an 8 elem array of u16 to 16 elem array of UINT8
        })
    }

    pub fn localhost() -> Ipv6Addr {
        Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)
    }

    pub fn unspecified() -> Ipv6Addr {
        Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)
    }

    pub fn segments(&self) -> [u16; 8] {
        let arr = &self.0.Addr;
        [
            (arr[0] as u16) << 8 | (arr[1] as u16),
            (arr[2] as u16) << 8 | (arr[3] as u16),
            (arr[4] as u16) << 8 | (arr[5] as u16),
            (arr[6] as u16) << 8 | (arr[7] as u16),
            (arr[8] as u16) << 8 | (arr[9] as u16),
            (arr[10] as u16) << 8 | (arr[11] as u16),
            (arr[12] as u16) << 8 | (arr[13] as u16),
            (arr[14] as u16) << 8 | (arr[15] as u16),
        ]
    }

    /// Returns [`true`] for the special 'unspecified' address (::).
    ///
    /// This property is defined in [IETF RFC 4291].
    ///
    /// [IETF RFC 4291]: https://tools.ietf.org/html/rfc4291
    pub fn is_unspecified(&self) -> bool {
        self.segments() == [0, 0, 0, 0, 0, 0, 0, 0]
    }

    /// Returns [`true`] if this is a loopback address (::1).
    ///
    /// This property is defined in [IETF RFC 4291].
    ///
    /// [IETF RFC 4291]: https://tools.ietf.org/html/rfc4291
    pub fn is_loopback(&self) -> bool {
        self.segments() == [0, 0, 0, 0, 0, 0, 0, 1]
    }

    /// Returns [`true`] if the address appears to be globally routable.
    ///
    /// The following return [`false`]:
    ///
    /// - the loopback address
    /// - link-local, site-local, and unique local unicast addresses
    /// - interface-, link-, realm-, admin- and site-local multicast addresses
    pub fn is_global(&self) -> bool {
        match self.multicast_scope() {
            Some(Ipv6MulticastScope::Global) => true,
            None => self.is_unicast_global(),
            _ => false
        }
    }

    /// Returns [`true`] if this is a unique local address (fc00::/7).
    ///
    /// This property is defined in [IETF RFC 4193].
    ///
    /// [IETF RFC 4193]: https://tools.ietf.org/html/rfc4193
    pub fn is_unique_local(&self) -> bool {
        (self.segments()[0] & 0xfe00) == 0xfc00
    }

    /// Returns [`true`] if the address is unicast and link-local (fe80::/10).
    ///
    /// This property is defined in [IETF RFC 4291].
    ///
    /// [IETF RFC 4291]: https://tools.ietf.org/html/rfc4291
    pub fn is_unicast_link_local(&self) -> bool {
        (self.segments()[0] & 0xffc0) == 0xfe80
    }

    /// Returns [`true`] if this is a deprecated unicast site-local address
    /// (fec0::/10).
    pub fn is_unicast_site_local(&self) -> bool {
        (self.segments()[0] & 0xffc0) == 0xfec0
    }

    /// Returns [`true`] if this is an address reserved for documentation
    /// (2001:db8::/32).
    ///
    /// This property is defined in [IETF RFC 3849].
    ///
    /// [IETF RFC 3849]: https://tools.ietf.org/html/rfc3849
    pub fn is_documentation(&self) -> bool {
        (self.segments()[0] == 0x2001) && (self.segments()[1] == 0xdb8)
    }

    /// Returns [`true`] if the address is a globally routable unicast address.
    ///
    /// The following return false:
    ///
    /// - the loopback address
    /// - the link-local addresses
    /// - the (deprecated) site-local addresses
    /// - unique local addresses
    /// - the unspecified address
    /// - the address range reserved for documentation
    ///
    pub fn is_unicast_global(&self) -> bool {
        !self.is_multicast()
            && !self.is_loopback() && !self.is_unicast_link_local()
            && !self.is_unicast_site_local() && !self.is_unique_local()
            && !self.is_unspecified() && !self.is_documentation()
    }

    /// Returns the address's multicast scope if the address is multicast.
    pub fn multicast_scope(&self) -> Option<Ipv6MulticastScope> {
        if self.is_multicast() {
            match self.segments()[0] & 0x000f {
                1 => Some(Ipv6MulticastScope::InterfaceLocal),
                2 => Some(Ipv6MulticastScope::LinkLocal),
                3 => Some(Ipv6MulticastScope::RealmLocal),
                4 => Some(Ipv6MulticastScope::AdminLocal),
                5 => Some(Ipv6MulticastScope::SiteLocal),
                8 => Some(Ipv6MulticastScope::OrganizationLocal),
                14 => Some(Ipv6MulticastScope::Global),
                _ => None
            }
        } else {
            None
        }
    }

    /// Returns [`true`] if this is a multicast address (ff00::/8).
    ///
    /// This property is defined by [IETF RFC 4291].
    ///
    /// [IETF RFC 4291]: https://tools.ietf.org/html/rfc4291
    pub fn is_multicast(&self) -> bool {
        (self.segments()[0] & 0xff00) == 0xff00
    }

    /// Converts this address to an [IPv4 address]. Returns [`None`] if this address is
    /// neither IPv4-compatible or IPv4-mapped.
    ///
    /// ::a.b.c.d and ::ffff:a.b.c.d become a.b.c.d
    pub fn to_ipv4(&self) -> Option<Ipv4Addr> {
        match self.segments() {
            [0, 0, 0, 0, 0, f, g, h] if f == 0 || f == 0xffff => {
                Some(Ipv4Addr::new((g >> 8) as u8, g as u8,
                                   (h >> 8) as u8, h as u8))
            },
            _ => None
        }
    }

    // TODO: Implement this. Note you can't just transmute from segments() because of endianness
    // /// Returns the sixteen eight-bit integers the IPv6 address consists of.
    // pub fn octets(&self) -> [u8; 16] {
    //     self.inner.s6_addr
    // }
}

impl From<EFI_IPv6_ADDRESS> for Ipv6Addr {
    fn from(val: EFI_IPv6_ADDRESS) -> Self {
        Ipv6Addr(val)
    }
}

impl From<Ipv6Addr > for EFI_IPv6_ADDRESS {
    fn from(val: Ipv6Addr) -> Self {
        val.0
    }
}

impl From<Ipv6Addr> for u128 {
    fn from(ip: Ipv6Addr) -> u128 {
        let ip = ip.segments();
        ((ip[0] as u128) << 112) + ((ip[1] as u128) << 96) + ((ip[2] as u128) << 80) +
            ((ip[3] as u128) << 64) + ((ip[4] as u128) << 48) + ((ip[5] as u128) << 32) +
            ((ip[6] as u128) << 16) + (ip[7] as u128)
    }
}

impl From<u128> for Ipv6Addr {
    fn from(ip: u128) -> Ipv6Addr {
        Ipv6Addr::new(
            (ip >> 112) as u16, (ip >> 96) as u16, (ip >> 80) as u16,
            (ip >> 64) as u16, (ip >> 48) as u16, (ip >> 32) as u16,
            (ip >> 16) as u16, ip as u16,
        )
    }
}

impl From<[u8; 16]> for Ipv6Addr {
    fn from(octets: [u8; 16]) -> Ipv6Addr {
        (EFI_IPv6_ADDRESS { Addr: octets }).into()
    }
}

impl From<[u16; 8]> for Ipv6Addr {
    fn from(segments: [u16; 8]) -> Ipv6Addr {
        let [a, b, c, d, e, f, g, h] = segments;
        Ipv6Addr::new(a, b, c, d, e, f, g, h)
    }
}

impl PartialOrd for Ipv6Addr {
    fn partial_cmp(&self, other: &Ipv6Addr) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<Ipv6Addr> for IpAddr {
    fn partial_cmp(&self, other: &Ipv6Addr) -> Option<Ordering> {
        match *self {
            IpAddr::V4(_) => Some(Ordering::Less),
            IpAddr::V6(ref v6) => v6.partial_cmp(other),
        }
    }
}

impl PartialOrd<IpAddr> for Ipv6Addr {
    fn partial_cmp(&self, other: &IpAddr) -> Option<Ordering> {
        match *other {
            IpAddr::V4(_) => Some(Ordering::Greater),
            IpAddr::V6(ref v6) => self.partial_cmp(v6),
        }
    }
}

impl Ord for Ipv6Addr {
    fn cmp(&self, other: &Ipv6Addr) -> Ordering {
        self.segments().cmp(&other.segments())
    }
}

impl PartialEq<IpAddr> for Ipv6Addr {
    fn eq(&self, other: &IpAddr) -> bool {
        match *other {
            IpAddr::V4(_) => false,
            IpAddr::V6(ref v6) => self == v6,
        }
    }
}

impl PartialEq<Ipv6Addr> for IpAddr {
    fn eq(&self, other: &Ipv6Addr) -> bool {
        match *self {
            IpAddr::V4(_) => false,
            IpAddr::V6(ref v6) => v6 == other,
        }
    }
}

impl fmt::Display for Ipv6Addr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.segments() {
            // We need special cases for :: and ::1, otherwise they're formatted
            // as ::0.0.0.[01]
            [0, 0, 0, 0, 0, 0, 0, 0] => write!(fmt, "::"),
            [0, 0, 0, 0, 0, 0, 0, 1] => write!(fmt, "::1"),
            // Ipv4 Compatible address
            [0, 0, 0, 0, 0, 0, g, h] => {
                write!(fmt, "::{}.{}.{}.{}", (g >> 8) as u8, g as u8,
                       (h >> 8) as u8, h as u8)
            }
            // Ipv4-Mapped address
            [0, 0, 0, 0, 0, 0xffff, g, h] => {
                write!(fmt, "::ffff:{}.{}.{}.{}", (g >> 8) as u8, g as u8,
                       (h >> 8) as u8, h as u8)
            },
            _ => {
                fn find_zero_slice(segments: &[u16; 8]) -> (usize, usize) {
                    let mut longest_span_len = 0;
                    let mut longest_span_at = 0;
                    let mut cur_span_len = 0;
                    let mut cur_span_at = 0;

                    for i in 0..8 {
                        if segments[i] == 0 {
                            if cur_span_len == 0 {
                                cur_span_at = i;
                            }

                            cur_span_len += 1;

                            if cur_span_len > longest_span_len {
                                longest_span_len = cur_span_len;
                                longest_span_at = cur_span_at;
                            }
                        } else {
                            cur_span_len = 0;
                            cur_span_at = 0;
                        }
                    }

                    (longest_span_at, longest_span_len)
                }

                let (zeros_at, zeros_len) = find_zero_slice(&self.segments());

                if zeros_len > 1 {
                    fn fmt_subslice(segments: &[u16], fmt: &mut fmt::Formatter) -> fmt::Result {
                        if !segments.is_empty() {
                            write!(fmt, "{:x}", segments[0])?;
                            for &seg in &segments[1..] {
                                write!(fmt, ":{:x}", seg)?;
                            }
                        }
                        Ok(())
                    }

                    fmt_subslice(&self.segments()[..zeros_at], fmt)?;
                    fmt.write_str("::")?;
                    fmt_subslice(&self.segments()[zeros_at + zeros_len..], fmt)
                } else {
                    let &[a, b, c, d, e, f, g, h] = &self.segments();
                    write!(fmt, "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
                           a, b, c, d, e, f, g, h)
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IpAddr {
    V4(Ipv4Addr),
    V6(Ipv6Addr)
}

impl IpAddr {
    /// Returns [`true`] for the special 'unspecified' address.
    pub fn is_unspecified(&self) -> bool {
        match *self {
            IpAddr::V4(ref a) => a.is_unspecified(),
            IpAddr::V6(ref a) => a.is_unspecified(),
        }
    }

    /// Returns [`true`] if this is a loopback address.
    pub fn is_loopback(&self) -> bool {
        match *self {
            IpAddr::V4(ref a) => a.is_loopback(),
            IpAddr::V6(ref a) => a.is_loopback(),
        }
    }

    /// Returns [`true`] if the address appears to be globally routable.
    pub fn is_global(&self) -> bool {
        match *self {
            IpAddr::V4(ref a) => a.is_global(),
            IpAddr::V6(ref a) => a.is_global(),
        }
    }

    /// Returns [`true`] if this is a multicast address.
    pub fn is_multicast(&self) -> bool {
        match *self {
            IpAddr::V4(ref a) => a.is_multicast(),
            IpAddr::V6(ref a) => a.is_multicast(),
        }
    }

    /// Returns [`true`] if this address is in a range designated for documentation.
    pub fn is_documentation(&self) -> bool {
        match *self {
            IpAddr::V4(ref a) => a.is_documentation(),
            IpAddr::V6(ref a) => a.is_documentation(),
        }
    }
}

impl From<Ipv4Addr> for IpAddr {
    fn from(ipv4: Ipv4Addr) -> IpAddr {
        IpAddr::V4(ipv4)
    }
}

impl From<Ipv6Addr> for IpAddr {
    fn from(ipv6: Ipv6Addr) -> IpAddr {
        IpAddr::V6(ipv6)
    }
}

impl From<EFI_IPv4_ADDRESS> for IpAddr {
    fn from(ipv4: EFI_IPv4_ADDRESS) -> IpAddr {
        IpAddr::V4(ipv4.into())
    }
}

impl From<EFI_IPv6_ADDRESS> for IpAddr {
    fn from(ipv6: EFI_IPv6_ADDRESS) -> IpAddr {
        IpAddr::V6(ipv6.into())
    }
}

impl From<[u8; 16]> for IpAddr {
    fn from(octets: [u8; 16]) -> IpAddr {
        IpAddr::V6(Ipv6Addr::from(octets))
    }
}

impl From<[u16; 8]> for IpAddr {
    fn from(segments: [u16; 8]) -> IpAddr {
        IpAddr::V6(Ipv6Addr::from(segments))
    }
}

impl fmt::Display for IpAddr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IpAddr::V4(a) => a.fmt(fmt),
            IpAddr::V6(a) => a.fmt(fmt),
        }
    }
}

impl From<IpAddr> for EFI_IP_ADDRESS {
    fn from(ip: IpAddr) -> EFI_IP_ADDRESS {
        match ip {
            IpAddr::V4(a) => EFI_IP_ADDRESS { v4: a.into() },
            IpAddr::V6(a) => EFI_IP_ADDRESS { v6: a.into() },
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SocketAddrV4 {
    ip: Ipv4Addr,
    port: u16,
}

impl SocketAddrV4 {
    pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        Self { ip, port }
    }

    pub fn ip(&self) -> &Ipv4Addr {
        &self.ip
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl fmt::Display for SocketAddrV4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.ip(), self.port())
    }
}

// TODO: Should we implement flow info and scope id?
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SocketAddrV6 {
    ip: Ipv6Addr,
    port: u16,
}

impl SocketAddrV6 {
    pub fn new(ip: Ipv6Addr, port: u16) -> Self {
        Self { ip, port }
    }

    pub fn ip(&self) -> &Ipv6Addr {
        &self.ip
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl fmt::Display for SocketAddrV6 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]:{}", self.ip(), self.port())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SocketAddr {
    V4(SocketAddrV4),
    V6(SocketAddrV6)
}

impl SocketAddr {
    pub fn new(ip: IpAddr, port: u16) -> SocketAddr {
        match ip {
            IpAddr::V4(a) => SocketAddr::V4(SocketAddrV4::new(a, port)),
            IpAddr::V6(a) => SocketAddr::V6(SocketAddrV6::new(a, port)),
        }
    }

    pub fn ip(&self) -> IpAddr {
        match self {
            SocketAddr::V4(a) => IpAddr::V4(*a.ip()),
            SocketAddr::V6(a) => IpAddr::V6(*a.ip()),
        }
    }

    pub fn port(&self) -> u16 {
        match self {
            SocketAddr::V4(a) => a.port(),
            SocketAddr::V6(a) => a.port(),
        }
    }

    pub fn is_ipv4(&self) -> bool {
        match *self {
            SocketAddr::V4(_) => true,
            SocketAddr::V6(_) => false,
        }
    }

    pub fn is_ipv6(&self) -> bool {
        match *self {
            SocketAddr::V4(_) => false,
            SocketAddr::V6(_) => true,
        }
    }
}

impl fmt::Display for SocketAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SocketAddr::V4(a) => a.fmt(f),
            SocketAddr::V6(a) => a.fmt(f),
        }
    }
}

impl From<SocketAddrV4> for SocketAddr {
    fn from(sock4: SocketAddrV4) -> SocketAddr {
        SocketAddr::V4(sock4)
    }
}

impl From<SocketAddrV6> for SocketAddr {
    fn from(sock6: SocketAddrV6) -> SocketAddr {
        SocketAddr::V6(sock6)
    }
}

impl<I: Into<IpAddr>> From<(I, u16)> for SocketAddr {
    fn from(pieces: (I, u16)) -> SocketAddr {
        SocketAddr::new(pieces.0.into(), pieces.1)
    }
}


pub trait ToSocketAddrs {
    /// Returned iterator over socket addresses which this type may correspond
    /// to.
    type Iter: Iterator<Item=SocketAddr>;

    /// Converts this object to an iterator of resolved `SocketAddr`s.
    ///
    /// The returned iterator may not actually yield any values depending on the
    /// outcome of any resolution performed.
    ///
    /// Note that this function may block the current thread while resolution is
    /// performed.
    fn to_socket_addrs(&self) -> io::Result<Self::Iter>;
}

impl ToSocketAddrs for SocketAddr {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        Ok(Some(*self).into_iter())
    }
}

impl ToSocketAddrs for SocketAddrV4 {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        SocketAddr::V4(*self).to_socket_addrs()
    }
}

impl ToSocketAddrs for SocketAddrV6 {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        SocketAddr::V6(*self).to_socket_addrs()
    }
}

impl ToSocketAddrs for (IpAddr, u16) {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        match ip {
            IpAddr::V4(ref a) => (*a, port).to_socket_addrs(),
            IpAddr::V6(ref a) => (*a, port).to_socket_addrs(),
        }
    }
}

impl ToSocketAddrs for (Ipv4Addr, u16) {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        SocketAddrV4::new(ip, port).to_socket_addrs()
    }
}

impl ToSocketAddrs for (Ipv6Addr, u16) {
    type Iter = option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<option::IntoIter<SocketAddr>> {
        let (ip, port) = *self;
        SocketAddrV6::new(ip, port).to_socket_addrs()
    }
}

#[allow(deprecated)]
fn resolve_socket_addr(hostname: &str, port: u16) -> io::Result<vec::IntoIter<SocketAddr>> {
    let ip_addrs = lookup_host(hostname).map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to resolve host name"))?;
    let sock_addrs: Vec<_> = ip_addrs.into_iter().map(|ip| SocketAddr::new(ip, port)).collect();
    Ok(sock_addrs.into_iter())
}

impl<'a> ToSocketAddrs for (&'a str, u16) {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        let (host, port) = *self;

        // try to parse the host as a regular IP address first
        if let Ok(addr) = host.parse::<Ipv4Addr>() {
            let addr = SocketAddrV4::new(addr, port);
            return Ok(vec![SocketAddr::V4(addr)].into_iter())
        }
        if let Ok(addr) = host.parse::<Ipv6Addr>() {
            let addr = SocketAddrV6::new(addr, port);
            return Ok(vec![SocketAddr::V6(addr)].into_iter())
        }

        resolve_socket_addr(host, port)
    }
}

// accepts strings like 'localhost:12345'
impl ToSocketAddrs for str {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        // try to parse as a regular SocketAddr first
        if let Some(addr) = self.parse().ok() {
            return Ok(vec![addr].into_iter());
        }

        macro_rules! try_opt {
            ($e:expr, $msg:expr) => (
                match $e {
                    Some(r) => r,
                    None => return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                                      $msg)),
                }
            )
        }

        // split the string by ':' and convert the second part to u16
        let mut parts_iter = self.rsplitn(2, ':');
        let port_str = try_opt!(parts_iter.next(), "invalid socket address");
        let host = try_opt!(parts_iter.next(), "invalid socket address");
        let port: u16 = try_opt!(port_str.parse().ok(), "invalid port value");
        resolve_socket_addr(host, port)
    }
}

impl<'a> ToSocketAddrs for &'a [SocketAddr] {
    type Iter = iter::Cloned<slice::Iter<'a, SocketAddr>>;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(self.iter().cloned())
    }
}

impl<'a, T: ToSocketAddrs + ?Sized> ToSocketAddrs for &'a T {
    type Iter = T::Iter;
    fn to_socket_addrs(&self) -> io::Result<T::Iter> {
        (**self).to_socket_addrs()
    }
}

impl ToSocketAddrs for String {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<vec::IntoIter<SocketAddr>> {
        (&**self).to_socket_addrs()
    }
}

// Tests for this module
#[cfg(all(test, not(target_os = "emscripten")))]
mod tests {
    use net::*;
    use net::Ipv6MulticastScope::*;
    use net::test::{tsa, sa6, sa4};

    #[test]
    fn test_from_str_ipv4() {
        assert_eq!(Ok(Ipv4Addr::new(127, 0, 0, 1)), "127.0.0.1".parse());
        assert_eq!(Ok(Ipv4Addr::new(255, 255, 255, 255)), "255.255.255.255".parse());
        assert_eq!(Ok(Ipv4Addr::new(0, 0, 0, 0)), "0.0.0.0".parse());

        // out of range
        let none: Option<Ipv4Addr> = "256.0.0.1".parse().ok();
        assert_eq!(None, none);
        // too short
        let none: Option<Ipv4Addr> = "255.0.0".parse().ok();
        assert_eq!(None, none);
        // too long
        let none: Option<Ipv4Addr> = "255.0.0.1.2".parse().ok();
        assert_eq!(None, none);
        // no number between dots
        let none: Option<Ipv4Addr> = "255.0..1".parse().ok();
        assert_eq!(None, none);
    }

    #[test]
    fn test_from_str_ipv6() {
        assert_eq!(Ok(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), "0:0:0:0:0:0:0:0".parse());
        assert_eq!(Ok(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), "0:0:0:0:0:0:0:1".parse());

        assert_eq!(Ok(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), "::1".parse());
        assert_eq!(Ok(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), "::".parse());

        assert_eq!(Ok(Ipv6Addr::new(0x2a02, 0x6b8, 0, 0, 0, 0, 0x11, 0x11)),
                "2a02:6b8::11:11".parse());

        // too long group
        let none: Option<Ipv6Addr> = "::00000".parse().ok();
        assert_eq!(None, none);
        // too short
        let none: Option<Ipv6Addr> = "1:2:3:4:5:6:7".parse().ok();
        assert_eq!(None, none);
        // too long
        let none: Option<Ipv6Addr> = "1:2:3:4:5:6:7:8:9".parse().ok();
        assert_eq!(None, none);
        // triple colon
        let none: Option<Ipv6Addr> = "1:2:::6:7:8".parse().ok();
        assert_eq!(None, none);
        // two double colons
        let none: Option<Ipv6Addr> = "1:2::6::8".parse().ok();
        assert_eq!(None, none);
        // `::` indicating zero groups of zeros
        let none: Option<Ipv6Addr> = "1:2:3:4::5:6:7:8".parse().ok();
        assert_eq!(None, none);
    }

    #[test]
    fn test_from_str_ipv4_in_ipv6() {
        assert_eq!(Ok(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 49152, 545)),
                "::192.0.2.33".parse());
        assert_eq!(Ok(Ipv6Addr::new(0, 0, 0, 0, 0, 0xFFFF, 49152, 545)),
                "::FFFF:192.0.2.33".parse());
        assert_eq!(Ok(Ipv6Addr::new(0x64, 0xff9b, 0, 0, 0, 0, 49152, 545)),
                "64:ff9b::192.0.2.33".parse());
        assert_eq!(Ok(Ipv6Addr::new(0x2001, 0xdb8, 0x122, 0xc000, 0x2, 0x2100, 49152, 545)),
                "2001:db8:122:c000:2:2100:192.0.2.33".parse());

        // colon after v4
        let none: Option<Ipv4Addr> = "::127.0.0.1:".parse().ok();
        assert_eq!(None, none);
        // not enough groups
        let none: Option<Ipv6Addr> = "1.2.3.4.5:127.0.0.1".parse().ok();
        assert_eq!(None, none);
        // too many groups
        let none: Option<Ipv6Addr> = "1.2.3.4.5:6:7:127.0.0.1".parse().ok();
        assert_eq!(None, none);
    }

    #[test]
    fn test_from_str_socket_addr() {
        assert_eq!(Ok(sa4(Ipv4Addr::new(77, 88, 21, 11), 80)),
                   "77.88.21.11:80".parse());
        assert_eq!(Ok(SocketAddrV4::new(Ipv4Addr::new(77, 88, 21, 11), 80)),
                   "77.88.21.11:80".parse());
        assert_eq!(Ok(sa6(Ipv6Addr::new(0x2a02, 0x6b8, 0, 1, 0, 0, 0, 1), 53)),
                   "[2a02:6b8:0:1::1]:53".parse());
        assert_eq!(Ok(SocketAddrV6::new(Ipv6Addr::new(0x2a02, 0x6b8, 0, 1,
                                                      0, 0, 0, 1), 53, 0, 0)),
                   "[2a02:6b8:0:1::1]:53".parse());
        assert_eq!(Ok(sa6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0x7F00, 1), 22)),
                   "[::127.0.0.1]:22".parse());
        assert_eq!(Ok(SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0,
                                                      0x7F00, 1), 22, 0, 0)),
                   "[::127.0.0.1]:22".parse());

        // without port
        let none: Option<SocketAddr> = "127.0.0.1".parse().ok();
        assert_eq!(None, none);
        // without port
        let none: Option<SocketAddr> = "127.0.0.1:".parse().ok();
        assert_eq!(None, none);
        // wrong brackets around v4
        let none: Option<SocketAddr> = "[127.0.0.1]:22".parse().ok();
        assert_eq!(None, none);
        // port out of range
        let none: Option<SocketAddr> = "127.0.0.1:123456".parse().ok();
        assert_eq!(None, none);
    }

    #[test]
    fn ipv6_addr_to_string() {
        // ipv4-mapped address
        let a1 = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc000, 0x280);
        assert_eq!(a1.to_string(), "::ffff:192.0.2.128");

        // ipv4-compatible address
        let a1 = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0xc000, 0x280);
        assert_eq!(a1.to_string(), "::192.0.2.128");

        // v6 address with no zero segments
        assert_eq!(Ipv6Addr::new(8, 9, 10, 11, 12, 13, 14, 15).to_string(),
                   "8:9:a:b:c:d:e:f");

        // reduce a single run of zeros
        assert_eq!("ae::ffff:102:304",
                   Ipv6Addr::new(0xae, 0, 0, 0, 0, 0xffff, 0x0102, 0x0304).to_string());

        // don't reduce just a single zero segment
        assert_eq!("1:2:3:4:5:6:0:8",
                   Ipv6Addr::new(1, 2, 3, 4, 5, 6, 0, 8).to_string());

        // 'any' address
        assert_eq!("::", Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).to_string());

        // loopback address
        assert_eq!("::1", Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1).to_string());

        // ends in zeros
        assert_eq!("1::", Ipv6Addr::new(1, 0, 0, 0, 0, 0, 0, 0).to_string());

        // two runs of zeros, second one is longer
        assert_eq!("1:0:0:4::8", Ipv6Addr::new(1, 0, 0, 4, 0, 0, 0, 8).to_string());

        // two runs of zeros, equal length
        assert_eq!("1::4:5:0:0:8", Ipv6Addr::new(1, 0, 0, 4, 5, 0, 0, 8).to_string());
    }

    #[test]
    fn ipv4_to_ipv6() {
        assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x1234, 0x5678),
                   Ipv4Addr::new(0x12, 0x34, 0x56, 0x78).to_ipv6_mapped());
        assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0x1234, 0x5678),
                   Ipv4Addr::new(0x12, 0x34, 0x56, 0x78).to_ipv6_compatible());
    }

    #[test]
    fn ipv6_to_ipv4() {
        assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x1234, 0x5678).to_ipv4(),
                   Some(Ipv4Addr::new(0x12, 0x34, 0x56, 0x78)));
        assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0x1234, 0x5678).to_ipv4(),
                   Some(Ipv4Addr::new(0x12, 0x34, 0x56, 0x78)));
        assert_eq!(Ipv6Addr::new(0, 0, 1, 0, 0, 0, 0x1234, 0x5678).to_ipv4(),
                   None);
    }

    #[test]
    fn ip_properties() {
        fn check4(octets: &[u8; 4], unspec: bool, loopback: bool,
                  global: bool, multicast: bool, documentation: bool) {
            let ip = IpAddr::V4(Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]));
            assert_eq!(ip.is_unspecified(), unspec);
            assert_eq!(ip.is_loopback(), loopback);
            assert_eq!(ip.is_global(), global);
            assert_eq!(ip.is_multicast(), multicast);
            assert_eq!(ip.is_documentation(), documentation);
        }

        fn check6(str_addr: &str, unspec: bool, loopback: bool,
                  global: bool, u_doc: bool, mcast: bool) {
            let ip = IpAddr::V6(str_addr.parse().unwrap());
            assert_eq!(ip.is_unspecified(), unspec);
            assert_eq!(ip.is_loopback(), loopback);
            assert_eq!(ip.is_global(), global);
            assert_eq!(ip.is_documentation(), u_doc);
            assert_eq!(ip.is_multicast(), mcast);
        }

        //     address                unspec loopbk global multicast doc
        check4(&[0, 0, 0, 0],         true,  false, false,  false,   false);
        check4(&[0, 0, 0, 1],         false, false, true,   false,   false);
        check4(&[0, 1, 0, 0],         false, false, true,   false,   false);
        check4(&[10, 9, 8, 7],        false, false, false,  false,   false);
        check4(&[127, 1, 2, 3],       false, true,  false,  false,   false);
        check4(&[172, 31, 254, 253],  false, false, false,  false,   false);
        check4(&[169, 254, 253, 242], false, false, false,  false,   false);
        check4(&[192, 0, 2, 183],     false, false, false,  false,   true);
        check4(&[192, 1, 2, 183],     false, false, true,   false,   false);
        check4(&[192, 168, 254, 253], false, false, false,  false,   false);
        check4(&[198, 51, 100, 0],    false, false, false,  false,   true);
        check4(&[203, 0, 113, 0],     false, false, false,  false,   true);
        check4(&[203, 2, 113, 0],     false, false, true,   false,   false);
        check4(&[224, 0, 0, 0],       false, false, true,   true,    false);
        check4(&[239, 255, 255, 255], false, false, true,   true,    false);
        check4(&[255, 255, 255, 255], false, false, false,  false,   false);

        //     address                            unspec loopbk global doc    mcast
        check6("::",                              true,  false, false, false, false);
        check6("::1",                             false, true,  false, false, false);
        check6("::0.0.0.2",                       false, false, true,  false, false);
        check6("1::",                             false, false, true,  false, false);
        check6("fc00::",                          false, false, false, false, false);
        check6("fdff:ffff::",                     false, false, false, false, false);
        check6("fe80:ffff::",                     false, false, false, false, false);
        check6("febf:ffff::",                     false, false, false, false, false);
        check6("fec0::",                          false, false, false, false, false);
        check6("ff01::",                          false, false, false, false, true);
        check6("ff02::",                          false, false, false, false, true);
        check6("ff03::",                          false, false, false, false, true);
        check6("ff04::",                          false, false, false, false, true);
        check6("ff05::",                          false, false, false, false, true);
        check6("ff08::",                          false, false, false, false, true);
        check6("ff0e::",                          false, false, true,  false, true);
        check6("2001:db8:85a3::8a2e:370:7334",    false, false, false, true,  false);
        check6("102:304:506:708:90a:b0c:d0e:f10", false, false, true,  false, false);
    }

    #[test]
    fn ipv4_properties() {
        fn check(octets: &[u8; 4], unspec: bool, loopback: bool,
                 private: bool, link_local: bool, global: bool,
                 multicast: bool, broadcast: bool, documentation: bool) {
            let ip = Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]);
            assert_eq!(octets, &ip.octets());

            assert_eq!(ip.is_unspecified(), unspec);
            assert_eq!(ip.is_loopback(), loopback);
            assert_eq!(ip.is_private(), private);
            assert_eq!(ip.is_link_local(), link_local);
            assert_eq!(ip.is_global(), global);
            assert_eq!(ip.is_multicast(), multicast);
            assert_eq!(ip.is_broadcast(), broadcast);
            assert_eq!(ip.is_documentation(), documentation);
        }

        //    address                unspec loopbk privt  linloc global multicast brdcast doc
        check(&[0, 0, 0, 0],         true,  false, false, false, false,  false,    false,  false);
        check(&[0, 0, 0, 1],         false, false, false, false, true,   false,    false,  false);
        check(&[0, 1, 0, 0],         false, false, false, false, true,   false,    false,  false);
        check(&[10, 9, 8, 7],        false, false, true,  false, false,  false,    false,  false);
        check(&[127, 1, 2, 3],       false, true,  false, false, false,  false,    false,  false);
        check(&[172, 31, 254, 253],  false, false, true,  false, false,  false,    false,  false);
        check(&[169, 254, 253, 242], false, false, false, true,  false,  false,    false,  false);
        check(&[192, 0, 2, 183],     false, false, false, false, false,  false,    false,  true);
        check(&[192, 1, 2, 183],     false, false, false, false, true,   false,    false,  false);
        check(&[192, 168, 254, 253], false, false, true,  false, false,  false,    false,  false);
        check(&[198, 51, 100, 0],    false, false, false, false, false,  false,    false,  true);
        check(&[203, 0, 113, 0],     false, false, false, false, false,  false,    false,  true);
        check(&[203, 2, 113, 0],     false, false, false, false, true,   false,    false,  false);
        check(&[224, 0, 0, 0],       false, false, false, false, true,   true,     false,  false);
        check(&[239, 255, 255, 255], false, false, false, false, true,   true,     false,  false);
        check(&[255, 255, 255, 255], false, false, false, false, false,  false,    true,   false);
    }

    #[test]
    fn ipv6_properties() {
        fn check(str_addr: &str, octets: &[u8; 16], unspec: bool, loopback: bool,
                 unique_local: bool, global: bool,
                 u_link_local: bool, u_site_local: bool, u_global: bool, u_doc: bool,
                 m_scope: Option<Ipv6MulticastScope>) {
            let ip: Ipv6Addr = str_addr.parse().unwrap();
            assert_eq!(str_addr, ip.to_string());
            assert_eq!(&ip.octets(), octets);
            assert_eq!(Ipv6Addr::from(*octets), ip);

            assert_eq!(ip.is_unspecified(), unspec);
            assert_eq!(ip.is_loopback(), loopback);
            assert_eq!(ip.is_unique_local(), unique_local);
            assert_eq!(ip.is_global(), global);
            assert_eq!(ip.is_unicast_link_local(), u_link_local);
            assert_eq!(ip.is_unicast_site_local(), u_site_local);
            assert_eq!(ip.is_unicast_global(), u_global);
            assert_eq!(ip.is_documentation(), u_doc);
            assert_eq!(ip.multicast_scope(), m_scope);
            assert_eq!(ip.is_multicast(), m_scope.is_some());
        }

        //    unspec loopbk uniqlo global unill  unisl  uniglo doc    mscope
        check("::", &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
              true,  false, false, false, false, false, false, false, None);
        check("::1", &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
              false, true,  false, false, false, false, false, false, None);
        check("::0.0.0.2", &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
              false, false, false, true,  false, false, true,  false, None);
        check("1::", &[0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
              false, false, false, true,  false, false, true,  false, None);
        check("fc00::", &[0xfc, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
              false, false, true,  false, false, false, false, false, None);
        check("fdff:ffff::", &[0xfd, 0xff, 0xff, 0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
              false, false, true,  false, false, false, false, false, None);
        check("fe80:ffff::", &[0xfe, 0x80, 0xff, 0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
              false, false, false, false, true,  false, false, false, None);
        check("febf:ffff::", &[0xfe, 0xbf, 0xff, 0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
              false, false, false, false, true,  false, false, false, None);
        check("fec0::", &[0xfe, 0xc0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
              false, false, false, false, false, true,  false, false, None);
        check("ff01::", &[0xff, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
              false, false, false, false, false, false, false, false, Some(InterfaceLocal));
        check("ff02::", &[0xff, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
              false, false, false, false, false, false, false, false, Some(LinkLocal));
        check("ff03::", &[0xff, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
              false, false, false, false, false, false, false, false, Some(RealmLocal));
        check("ff04::", &[0xff, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
              false, false, false, false, false, false, false, false, Some(AdminLocal));
        check("ff05::", &[0xff, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
              false, false, false, false, false, false, false, false, Some(SiteLocal));
        check("ff08::", &[0xff, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
              false, false, false, false, false, false, false, false, Some(OrganizationLocal));
        check("ff0e::", &[0xff, 0xe, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
              false, false, false, true,  false, false, false, false, Some(Global));
        check("2001:db8:85a3::8a2e:370:7334",
              &[0x20, 1, 0xd, 0xb8, 0x85, 0xa3, 0, 0, 0, 0, 0x8a, 0x2e, 3, 0x70, 0x73, 0x34],
              false, false, false, false, false, false, false, true, None);
        check("102:304:506:708:90a:b0c:d0e:f10",
              &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
              false, false, false, true,  false, false, true,  false, None);
    }

    #[test]
    fn to_socket_addr_socketaddr() {
        let a = sa4(Ipv4Addr::new(77, 88, 21, 11), 12345);
        assert_eq!(Ok(vec![a]), tsa(a));
    }

    #[test]
    fn test_ipv4_to_int() {
        let a = Ipv4Addr::new(0x11, 0x22, 0x33, 0x44);
        assert_eq!(u32::from(a), 0x11223344);
    }

    #[test]
    fn test_int_to_ipv4() {
        let a = Ipv4Addr::new(0x11, 0x22, 0x33, 0x44);
        assert_eq!(Ipv4Addr::from(0x11223344), a);
    }

    #[test]
    fn test_ipv6_to_int() {
        let a = Ipv6Addr::new(0x1122, 0x3344, 0x5566, 0x7788, 0x99aa, 0xbbcc, 0xddee, 0xff11);
        assert_eq!(u128::from(a), 0x112233445566778899aabbccddeeff11u128);
    }

    #[test]
    fn test_int_to_ipv6() {
        let a = Ipv6Addr::new(0x1122, 0x3344, 0x5566, 0x7788, 0x99aa, 0xbbcc, 0xddee, 0xff11);
        assert_eq!(Ipv6Addr::from(0x112233445566778899aabbccddeeff11u128), a);
    }

    #[test]
    fn ipv4_from_constructors() {
        assert_eq!(Ipv4Addr::localhost(), Ipv4Addr::new(127, 0, 0, 1));
        assert!(Ipv4Addr::localhost().is_loopback());
        assert_eq!(Ipv4Addr::unspecified(), Ipv4Addr::new(0, 0, 0, 0));
        assert!(Ipv4Addr::unspecified().is_unspecified());
    }

    #[test]
    fn ipv6_from_contructors() {
        assert_eq!(Ipv6Addr::localhost(), Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        assert!(Ipv6Addr::localhost().is_loopback());
        assert_eq!(Ipv6Addr::unspecified(), Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0));
        assert!(Ipv6Addr::unspecified().is_unspecified());
    }

    #[test]
    fn ipv4_from_octets() {
        assert_eq!(Ipv4Addr::from([127, 0, 0, 1]), Ipv4Addr::new(127, 0, 0, 1))
    }

    #[test]
    fn ipv6_from_segments() {
        let from_u16s = Ipv6Addr::from([0x0011, 0x2233, 0x4455, 0x6677,
                                        0x8899, 0xaabb, 0xccdd, 0xeeff]);
        let new = Ipv6Addr::new(0x0011, 0x2233, 0x4455, 0x6677,
                                0x8899, 0xaabb, 0xccdd, 0xeeff);
        assert_eq!(new, from_u16s);
    }

    #[test]
    fn ipv6_from_octets() {
        let from_u16s = Ipv6Addr::from([0x0011, 0x2233, 0x4455, 0x6677,
                                        0x8899, 0xaabb, 0xccdd, 0xeeff]);
        let from_u8s = Ipv6Addr::from([0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
                                       0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
        assert_eq!(from_u16s, from_u8s);
    }

    #[test]
    fn cmp() {
        let v41 = Ipv4Addr::new(100, 64, 3, 3);
        let v42 = Ipv4Addr::new(192, 0, 2, 2);
        let v61 = "2001:db8:f00::1002".parse::<Ipv6Addr>().unwrap();
        let v62 = "2001:db8:f00::2001".parse::<Ipv6Addr>().unwrap();
        assert!(v41 < v42);
        assert!(v61 < v62);

        assert_eq!(v41, IpAddr::V4(v41));
        assert_eq!(v61, IpAddr::V6(v61));
        assert!(v41 != IpAddr::V4(v42));
        assert!(v61 != IpAddr::V6(v62));

        assert!(v41 < IpAddr::V4(v42));
        assert!(v61 < IpAddr::V6(v62));
        assert!(IpAddr::V4(v41) < v42);
        assert!(IpAddr::V6(v61) < v62);

        assert!(v41 < IpAddr::V6(v61));
        assert!(IpAddr::V4(v41) < v61);
    }

    #[test]
    fn is_v4() {
        let ip = IpAddr::V4(Ipv4Addr::new(100, 64, 3, 3));
        assert!(ip.is_ipv4());
        assert!(!ip.is_ipv6());
    }

    #[test]
    fn is_v6() {
        let ip = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x1234, 0x5678));
        assert!(!ip.is_ipv4());
        assert!(ip.is_ipv6());
    }
}