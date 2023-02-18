/// Reading interface addresses
use std::net::{Ipv4Addr, Ipv6Addr};

use nix::ifaddrs::getifaddrs;
use nix::Result;
use nix::sys::socket::AddressFamily::{Inet, Inet6};
use nix::sys::socket::SockaddrLike;

pub struct Addrs {
    pub v4_addrs: Box<dyn Iterator<Item=Ipv4Addr>>,
    pub v6_addrs: Box<dyn Iterator<Item=Ipv6Addr>>,
}

/// Getting all IPv{4,6} addresses on a specific interface
fn get_addrs(if_name: &str) -> Result<(Vec<Ipv4Addr>, Vec<Ipv6Addr>)> {
    let mut v4 = Vec::new();
    let mut v6 = Vec::new();

    // Find all the addresses on that interface
    let addrs =
        getifaddrs()?
            .filter(|a| a.interface_name == if_name)
            // getifaddrs() will return return at least one entry for each interface. If that interface
            // have neither MAC nor L3 addresses, which is possible for e.g. tun and ppp interfaces,
            // then the address field of the only entry will be set to NULL/None. We filter those out.
            .filter_map(|a| a.address);

    // Loop over the addresses, add them to the vec if they are a IPv4 address or a IPv6 address
    for addr in addrs {
        // family() will return None if the `nix` crate does not recognize the address family. We
        // will only check the address families that we're interested in, i.e., `AF_INET` (IPv4) and
        // `AF_INET6` (IPv6)
        if addr.family() == Some(Inet) {
            let ip =
                addr.as_sockaddr_in().expect("Impossible, failed to cast to sockaddr_in")
                    .ip();
            v4.push(Ipv4Addr::from(ip))
        } else if addr.family() == Some(Inet6) {
            let ip =
                addr.as_sockaddr_in6().expect("Impossible, failed to cast to sockaddr_in6")
                    .ip();
            v6.push(ip)
        }
        // And ignore other address families.
    }

    Ok((v4, v6))
}

/// Get all the addresses that we are interested in, which is all the addresses
/// except link local addresses and loopback addresses.
pub fn get_interested_addrs(if_name: &str) -> Result<Addrs>
{
    let (all_v4, all_v6) = get_addrs(if_name)?;
    let filtered_v4 =
        all_v4.into_iter().filter(
            |addr| !addr.is_link_local() && !addr.is_loopback()
        );
    let filtered_v6 =
        all_v6.into_iter().filter(|addr| {
            let seg = addr.segments();
            !addr.is_loopback() && seg[0] & 0xffc0 != 0xfe80
        });
    Ok(Addrs {
        v4_addrs: Box::new(filtered_v4),
        v6_addrs: Box::new(filtered_v6),
    })
}

#[cfg(test)]
mod test {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use nix::Result;

    use crate::interface::{Addrs, get_addrs, get_interested_addrs};

    #[test]
    #[cfg(linux)]
    fn test_get_addrs() -> Result<()> {
        let (v4addrs, v6addrs) = get_addrs("lo")?;

        assert_eq!(v4addrs[0], Ipv4Addr::new(127, 0, 0, 1));
        assert_eq!(v6addrs[0], Ipv6Addr::from(1));

        Ok(())
    }

    #[test]
    #[cfg(not(linux))]
    fn test_get_addrs() -> Result<()> {
        // loopback interface is named "lo0" on BSDs (and MacOS)
        let (v4addrs, v6addrs) = get_addrs("lo0")?;

        assert_eq!(v4addrs[0], Ipv4Addr::new(127, 0, 0, 1));
        assert_eq!(v6addrs[0], Ipv6Addr::from(1));

        Ok(())
    }

    #[test]
    fn test_get_interested_addrs() -> Result<()> {
        let Addrs {
            v4_addrs, v6_addrs
        } = get_interested_addrs("ens34")?;

        println!("{:?}", v4_addrs.collect::<Vec<_>>());
        println!("{:?}", v6_addrs.collect::<Vec<_>>());
        Ok(())
    }
}
