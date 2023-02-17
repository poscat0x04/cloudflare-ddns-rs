/// Abstracts over cloudflare API for easily updating DNS records
use std::net::{Ipv4Addr, Ipv6Addr};
use std::process::ExitCode;

use anyhow::Context;
use cloudflare::endpoints::dns::{
    CreateDnsRecord, CreateDnsRecordParams, DeleteDnsRecord, DeleteDnsRecordResponse, DnsContent, DnsRecord,
    ListDnsRecords, ListDnsRecordsParams, UpdateDnsRecord, UpdateDnsRecordParams,
};
use cloudflare::framework::async_api::{ApiClient, Client};
use cloudflare::framework::response::{ApiFailure, ApiSuccess};
use itertools::{EitherOrBoth, Itertools};

use crate::config::Config;
use crate::exit_code::{Result, user_error_exitcode, WithExitCode};

struct RecordIDs {
    v4_ids: Vec<String>,
    v6_ids: Vec<String>,
}

/// Update DNS records with regard to a config
pub async fn update_dns_with(
    client: &Client,
    cfg: &Config,
    v4_addrs: impl Iterator<Item=Ipv4Addr>,
    v6_addrs: impl Iterator<Item=Ipv6Addr>,
) -> Result<()> {
    let RecordIDs {
        v4_ids,
        v6_ids,
    } = get_all_record_ids(client, &cfg.zone_id, &cfg.name).await?;

    // We use a macro because while the update logic is the same for both A and AAAA records, it can't be typed if
    // we try to implement it as a function.
    //
    // The logic itself is very simple, we simply use Itertools::zip_longest to zip together existing record IDs and
    // addrs, which handles the potential length difference, and then iterate over the zip product to
    // update/create/delete.
    macro_rules! update_all {
        ($cons:ident, $pairs:expr) => {
            for pair in $pairs {
                match pair {
                    EitherOrBoth::Both(addr, record_id) => {
                        let _: ApiSuccess<DnsRecord> =
                            client.request(&UpdateDnsRecord {
                                zone_identifier: &cfg.zone_id,
                                identifier: &record_id,
                                params: UpdateDnsRecordParams {
                                    ttl: Some(cfg.ttl),
                                    proxied: Some(cfg.proxied),
                                    name: &cfg.name,
                                    content: DnsContent::$cons {content: addr},
                                },
                            }).await
                            .with_context(|| format!("Failed to update DNS record with ID {} to {}", &record_id, addr))
                            .map_exit_code(api_error_to_exitcode)?;
                    }
                    EitherOrBoth::Left(addr) => {
                        let _: ApiSuccess<DnsRecord> =
                            client.request(&CreateDnsRecord {
                                zone_identifier: &cfg.zone_id,
                                params: CreateDnsRecordParams {
                                    ttl: Some(cfg.ttl),
                                    priority: None,
                                    proxied: Some(cfg.proxied),
                                    name: &cfg.name,
                                    content: DnsContent::$cons {content: addr},
                                },
                            }).await
                            .with_context(|| format!("Failed to create DNS record for {}", addr))
                            .map_exit_code(api_error_to_exitcode)?;
                    }
                    EitherOrBoth::Right(record_id) => {
                        let _: ApiSuccess<DeleteDnsRecordResponse> =
                            client.request(&DeleteDnsRecord {
                                zone_identifier: &cfg.zone_id,
                                identifier: &record_id,
                            }).await
                            .with_context(|| format!("Failed to delete DNS record with ID {}", &record_id))
                            .map_exit_code(api_error_to_exitcode)?;
                    }
                }
            }
        }
    }

    if cfg.v4 {
        let v4_pairs = v4_addrs.zip_longest(v4_ids);
        update_all!(A, v4_pairs)
    }

    if cfg.v6 {
        let v6_pairs = v6_addrs.zip_longest(v6_ids);
        update_all!(AAAA, v6_pairs)
    }

    Ok(())
}

/// Fetch all non-locked A and AAAA records of a specific domain name in a zone.
/// Handles pagination
async fn get_all_record_ids(
    client: &Client,
    zone_id: &str,
    name: &str,
) -> Result<RecordIDs> {
    const PAGE_SIZE: u16 = 5000;

    let mut v4_ids = Vec::new();
    let mut v6_ids = Vec::new();
    // pagination tracker
    let mut page = 1;

    // TODO: query only A and AAAA records when the cloudflare crates supports it
    loop {
        let r: ApiSuccess<Vec<DnsRecord>> =
            client.request(&ListDnsRecords {
                zone_identifier: zone_id,
                params: ListDnsRecordsParams {
                    record_type: None,
                    name: Some(name.to_string()),
                    page: Some(page),
                    per_page: Some(u32::from(PAGE_SIZE)),
                    order: None,
                    direction: None,
                    search_match: None,
                },
            }).await.context("Failed to list DNS records")
                .map_exit_code(api_error_to_exitcode)?;

        let finish = r.result.len() < usize::from(PAGE_SIZE);

        let non_locked =
            r.result.into_iter().filter(|r| !r.locked);

        for record in non_locked {
            match record.content {
                DnsContent::A { .. } => v4_ids.push(record.id),
                DnsContent::AAAA { .. } => v6_ids.push(record.id),
                _ => (),
            }
        }

        if finish { break; }
        page += 1
    }

    Ok(RecordIDs { v4_ids, v6_ids })
}

fn api_error_to_exitcode(err: &anyhow::Error) -> ExitCode {
    match err.downcast_ref::<ApiFailure>() {
        Some(ApiFailure::Error(_, _)) => user_error_exitcode(),
        _ => ExitCode::FAILURE,
    }
}

#[cfg(test)]
mod test {}