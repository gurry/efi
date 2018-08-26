//! The network-agnostic DNS parser library
//!
//! [Documentation](https://docs.rs/dns-parser) |
//! [Github](https://github.com/tailhook/dns-parser) |
//! [Crate](https://crates.io/crates/dns-parser)
//!
//! Use [`Builder`] to create a new outgoing packet.
//!
//! Use [`Packet::parse`] to parse a packet into a data structure.
//!
//! [`Builder`]: struct.Builder.html
//! [`Packet::parse`]: struct.Packet.html#method.parse
//!
#![warn(missing_docs)]

#[cfg(test)] #[macro_use] extern crate matches;
//#[macro_use(quick_error)] extern crate quick_error;
//#[cfg(feature = "with-serde")] #[macro_use] extern crate serde_derive;

mod enums;
mod structs;
mod name;
mod parser;
mod error;
mod header;
mod builder;

pub mod rdata;

pub use self::enums::{Type, QueryType, Class, QueryClass, ResponseCode, Opcode};
pub use self::structs::{Question, ResourceRecord, Packet};
pub use self::name::{Name};
pub use self::error::{Error};
pub use self::header::{Header};
pub use self::rdata::{RData};
pub use self::builder::{Builder};

use core;
use super::{UdpSocket, SocketAddr, IpAddr, Ipv4Addr};
use alloc::Vec;
use net::dhcp;

struct DnsServer {
    addr: SocketAddr
}

// TODO: Swallowing/transmorgifying all errors. Fix this large scale shit wherever present
impl DnsServer {
    fn query(&self, hostname: &str) -> ::Result<Vec<IpAddr>> {
        use net::dns::rdata::a::Record;
        let mut builder = Builder::new_query(1, true);
        builder.add_question(hostname, false, QueryType::A, QueryClass::IN);
        let packet = builder.build().map_err(|_| ::EfiErrorKind::DeviceError)?; 
        let mut socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.send_to(&packet, self.addr)?;
        let mut buf = [0u8; 4096];
        socket.recv(&mut buf)?; // TODO: Add timeout to this call. Or it may be stuck forever.
        let pkt = Packet::parse(&buf).unwrap();
        if pkt.header.response_code != ResponseCode::NoError {
            // return Err(pkt.header.response_code.into());
            return Err(::EfiErrorKind::DeviceError.into());
        }

        if pkt.answers.len() == 0 {
            return Err(::EfiErrorKind::DeviceError.into());
        }

        let addrs = pkt.answers.iter()
                            .filter_map(|a| { 
                                match a.data {
                                    RData::A(Record(addr)) => Some(IpAddr::V4(addr)),
                                    _ => None
                                }
                            }).collect::<Vec<_>>();
        Ok(addrs)
    }
}

pub (crate) fn lookup_host(hostname: &str) -> ::Result<Vec<IpAddr>> {
    let dns_servers = get_dns_servers()?;
    if dns_servers.is_empty() {
        return Err(::EfiErrorKind::DeviceError.into());
    }

    for dns_server in dns_servers {
        let addrs = match dns_server.query(hostname) {
            Ok(addrs) => addrs,
            Err(_) => continue
        };
        if !addrs.is_empty() {
            return Ok(addrs);
        }
    }

    Ok(Vec::new())
}

fn get_dns_servers() -> ::Result<Vec<DnsServer>> {
    // TODO: Assuming here that PXE has already happened. Should we kick it off here if it hasn't?
    let dhcp_config = dhcp::cached_dhcp_config()?
                .ok_or_else(|| ::EfiError::from(::EfiErrorKind::DeviceError))?;
    const DHCP_DNS_SERVERS_OPTION: u8 = 6;
    let dns_servers_option = dhcp_config.dhcp_ack_packet().dhcp_options().find(|o| o.code() == DHCP_DNS_SERVERS_OPTION)
                .ok_or_else(|| ::EfiError::from(::EfiErrorKind::DeviceError))?;
    let dns_servers_buf = dns_servers_option.value()
                .ok_or_else(|| ::EfiError::from(::EfiErrorKind::DeviceError))?;

    // Using explicit invocation syntax for 'exact_chunks' because of a compiler bug which leads to 
    // multiple candidates found for this method: https://github.com/rust-lang/rust/issues/51402.
    // We actually don't even want to use the SliceExt trait but the method on the inherent impl, 
    // but I couldn't find a way to do it.
    let ip_addresses = core::slice::SliceExt::exact_chunks(dns_servers_buf, 4).map(|c| Ipv4Addr::new(c[0], c[1], c[2], c[3])); 

    const DNS_PORT: u16 = 53;
    let dns_servers = ip_addresses.map(|ip| DnsServer { addr: (IpAddr::V4(ip), DNS_PORT).into() }).collect::<Vec<_>>();

    Ok(dns_servers)
}
