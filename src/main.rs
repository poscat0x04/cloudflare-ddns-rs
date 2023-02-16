use anyhow::{bail, Context, Result};
use tokio::fs::read_to_string;
use toml::from_str;

use crate::cli::Args;
use crate::config::Config;
use crate::utils::build_client_from_env;

mod config;
mod cli;
mod dns;
mod iface;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    // initialization
    let args: Args = argh::from_env();

    let cfg_file =
        read_to_string(&args.config).await
            .context("Failed to read config file")?;
    let cfg: Config = from_str(&cfg_file).context("Failed to parse config file")?;
    drop(cfg_file);

    if !cfg.v4 && !cfg.v6 { bail!(r#"The config options "v4" and "v6" cant both be false."#) }

    let client = build_client_from_env()?;
    // initialization complete

    Ok(())
}

/*
fn main_() -> i32 {
    match Opts::try_parse() {
        Err(e) => {
            e.print()
                .expect("Internal error: unable to print to stderr");
            return 17;
        }
        Ok(Opts { config, oneshot }) => match read_to_string(&config) {
            Err(e) => {
                let mut err_msg =
                    format!("Unable to read config file \"{}\", I/O error:", &config).yellow();
                println!("{}", err_msg);
                err_msg = format!("{}", e).red();
                println!("{}", err_msg);
                return 18;
            }
            Ok(cfg_str) => match from_str(&cfg_str) {
                Err(e) => {
                    let mut err_msg =
                        format!("Unable to read config file \"{}\", parse error:", &config)
                            .yellow();
                    println!("{}", err_msg);
                    err_msg = format!("{}", e).red();
                    println!("{}", err_msg);
                    return 19;
                }
                Ok(cfg) => app(cfg, oneshot),
            },
        },
    }
    return 0;
}
*/

/*
fn app(config: Config, oneshot: bool) -> i32 {
    // construct the interface address mapping
    let mut ipv4_map = HashMap::new();
    let mut ipv6_map = HashMap::new();
    match build_iface_map(&mut ipv4_map, &mut ipv6_map) {
        Ok(()) => (),
        Err(e) => {
            let mut err_msg = "Unable to read interface addresses, errno:".yellow();
            println!("{}", err_msg);
            err_msg = String::from(e.desc()).red();
            println!("{}", err_msg);
            return 1;
        }
    }

    // build the API client
    match HttpApiClient::new(
        config.credential.convert(),
        Default::default(),
        Environment::Production,
    ) {
        Err(e) => {
            let mut err_msg = "Unable to create cloudflare API client, error:".yellow();
            println!("{}", err_msg);
            err_msg = format!("{}", e).red();
            println!("{}", err_msg);
            return 2;
        }
        Ok(client) => {
            let DomainConfig {
                name,
                iface,
                zone_id,
                dualstack,
                proxied,
                ttl,
            } = config.config;
            loop {
                match get_all_record_ids(&client, &zone_id, &name) {
                    Err(err) => {
                        let mut err_msg = format!(
                        "Failed to fetch A and AAAA records associated with the domain name \"{}\":",
                        name
                    )
                    .yellow();
                        println!("{}", err_msg);
                        err_msg = err.to_string().red();
                        println!("{}", err_msg);
                        return 1;
                    }
                    Ok((ipv4_ids, ipv6_ids)) => {
                        let empty_vec4 = Vec::new();
                        let empty_vec6 = Vec::new();
                        let iface_ipv4_addrs = ipv4_map.get(&iface).unwrap_or(&empty_vec4);
                        let iface_ipv6_addrs = ipv6_map.get(&iface).unwrap_or(&empty_vec6);
                        if ipv4_ids.len() == 1 && iface_ipv4_addrs.len() == 1 {
                            let addr = iface_ipv4_addrs[0];
                            let r = client.request(&UpdateDnsRecord {
                                zone_identifier: &zone_id,
                                identifier: &ipv4_ids[0],
                                params: UpdateDnsRecordParams {
                                    ttl: Some(ttl),
                                    proxied: Some(proxied),
                                    name: &name,
                                    content: DnsContent::A { content: addr },
                                },
                            });
                            if let Err(err) = r {
                                let mut err_msg = format!(
                                    "Failed to update A record with content {} for domain \"{}\"",
                                    addr, name
                                )
                                .yellow();
                                println!("{}", err_msg);
                                err_msg = err.to_string().red();
                                println!("{}", err_msg);
                                return 1;
                            }
                        } else {
                            if let Err(err) = delete_all_records(&client, &zone_id, &ipv4_ids) {
                                let mut err_msg = format!(
                                    "Failed to delete all A records associated with the domain name \"{}\"",name
                                )
                                .yellow();
                                println!("{}", err_msg);
                                err_msg = err.to_string().red();
                                println!("{}", err_msg);
                                return 1;
                            }
                            for addr in iface_ipv4_addrs {
                                let r = client.request(&CreateDnsRecord {
                                    zone_identifier: &zone_id,
                                    params: CreateDnsRecordParams {
                                        ttl: Some(ttl),
                                        priority: None,
                                        proxied: Some(proxied),
                                        name: &name,
                                        content: DnsContent::A { content: *addr },
                                    },
                                });
                                if let Err(err) = r {
                                    let mut err_msg = format!(
                                    "Failed to create A record with content {} for domain \"{}\"",
                                    addr, name
                                )
                                    .yellow();
                                    println!("{}", err_msg);
                                    err_msg = err.to_string().red();
                                    println!("{}", err_msg);
                                    return 1;
                                }
                            }
                        }
                        if dualstack {
                            if ipv6_ids.len() == 1 && iface_ipv6_addrs.len() == 1 {
                                let addr = iface_ipv6_addrs[0];
                                let r = client.request(&UpdateDnsRecord {
                                    zone_identifier: &zone_id,
                                    identifier: &ipv6_ids[0],
                                    params: UpdateDnsRecordParams {
                                        ttl: Some(ttl),
                                        proxied: Some(proxied),
                                        name: &name,
                                        content: DnsContent::AAAA { content: addr },
                                    },
                                });
                                if let Err(err) = r {
                                    let mut err_msg = format!(
                                "Failed to update AAAA record with content {} for domain \"{}\"",
                                addr, name
                            )
                                    .yellow();
                                    println!("{}", err_msg);
                                    err_msg = err.to_string().red();
                                    println!("{}", err_msg);
                                    return 1;
                                }
                            } else {
                                if let Err(err) = delete_all_records(&client, &zone_id, &ipv6_ids) {
                                    let mut err_msg = format!(
                                    "Failed to delete all AAAA records associated with the domain name \"{}\"",name
                                )
                                .yellow();
                                    println!("{}", err_msg);
                                    err_msg = err.to_string().red();
                                    println!("{}", err_msg);
                                    return 1;
                                }
                                for addr in iface_ipv6_addrs {
                                    let r = client.request(&CreateDnsRecord {
                                        zone_identifier: &zone_id,
                                        params: CreateDnsRecordParams {
                                            ttl: Some(ttl),
                                            priority: None,
                                            proxied: Some(proxied),
                                            name: &name,
                                            content: DnsContent::AAAA { content: *addr },
                                        },
                                    });
                                    if let Err(err) = r {
                                        let mut err_msg = format!(
                                    "Failed to create AAAA record with content {} for domain \"{}\"",
                                    addr, name
                                )
                                .yellow();
                                        println!("{}", err_msg);
                                        err_msg = err.to_string().red();
                                        println!("{}", err_msg);
                                        return 1;
                                    }
                                }
                            }
                        }
                    }
                }
                let msg = "Successfully updated DNS records".green();
                println!("{}", msg);
                if oneshot {
                    return 0;
                }
                println!("Going into sleep for {} seconds", SLEEP_TIME);
                thread::sleep(Duration::from_secs(SLEEP_TIME));
            }
        }
    }
}

//

fn get_up_ifaddrs() -> nix::Result<impl Iterator<Item = InterfaceAddress>> {
    let addrs = getifaddrs()?;
    Ok(addrs.filter(|x| {
        x.flags.contains(InterfaceFlags::IFF_UP)
            && x.flags.contains(InterfaceFlags::IFF_RUNNING)
            && !x.flags.contains(InterfaceFlags::IFF_LOOPBACK)
    }))
}

fn build_iface_map(
    ipv4_map: &mut HashMap<String, Vec<Ipv4Addr>>,
    ipv6_map: &mut HashMap<String, Vec<Ipv6Addr>>,
) -> nix::Result<()> {
    let ifaddrs = get_up_ifaddrs()?;
    for addr in ifaddrs {
        let iface = addr.interface_name;
        match addr.address {
            Some(SockAddr::Inet(InetAddr::V4(sockaddr_in))) => {
                let ipv4_addr: Ipv4Addr = unsafe { transmute(sockaddr_in.sin_addr.s_addr) };
                if !ipv4_addr.is_link_local() {
                    match ipv4_map.get_mut(&iface) {
                        None => {
                            let mut v = Vec::new();
                            v.push(ipv4_addr);
                            ipv4_map.insert(iface, v);
                        }
                        Some(v) => {
                            v.push(ipv4_addr);
                        }
                    }
                }
            }
            Some(SockAddr::Inet(InetAddr::V6(sockaddr_in6))) => {
                let ipv6_addr = Ipv6Addr::from(sockaddr_in6.sin6_addr.s6_addr);
                if true {
                    match ipv6_map.get_mut(&iface) {
                        None => {
                            let mut v = Vec::new();
                            v.push(ipv6_addr);
                            ipv6_map.insert(iface, v);
                        }
                        Some(v) => {
                            v.push(ipv6_addr);
                        }
                    }
                }
            }
            _ => (),
        }
    }
    return Ok(());
}

fn delete_all_records<T>(
    client: &T,
    zone_identifier: &str,
    ids: &Vec<String>,
) -> Result<(), ApiFailure>
where
    T: ApiClient,
{
    for identifier in ids {
        client.request(&DeleteDnsRecord {
            zone_identifier,
            identifier,
        })?;
        ()
    }
    Ok(())
}

fn get_all_record_ids<T>(
    client: &T,
    zone_identifier: &str,
    name: &String,
) -> Result<(Vec<String>, Vec<String>), ApiFailure>
where
    T: ApiClient,
{
    const PER_PAGE: u32 = 100;
    let mut finished = false;
    let mut ipv4_ids = Vec::new();
    let mut ipv6_ids = Vec::new();
    let mut items_counter = 0;
    let mut page_counter = 1;
    let mut params: ListDnsRecordsParams = Default::default();
    params.name = Some(String::from(name));
    params.per_page = Some(PER_PAGE);
    let mut api_call = ListDnsRecords {
        zone_identifier,
        params,
    };
    while !finished {
        api_call.params.page = Some(page_counter);
        let records = client.request(&api_call)?.result;
        for record in records {
            match record.content {
                DnsContent::A { content: _ } => ipv4_ids.push(record.id),
                DnsContent::AAAA { content: _ } => ipv6_ids.push(record.id),
                _ => (),
            }
            items_counter += 1;
        }
        if items_counter < PER_PAGE {
            finished = true;
        } else {
            page_counter += 1;
            items_counter = 0;
        }
    }
    return Ok((ipv4_ids, ipv6_ids));
}
*/
