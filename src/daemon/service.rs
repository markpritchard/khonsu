use std::net::IpAddr;

use cloudflare::endpoints::dns;
use cloudflare::endpoints::dns::{DnsContent, DnsRecord};
use cloudflare::framework::{
    apiclient::ApiClient,
    auth::Credentials,
    Environment,
    HttpApiClient, HttpApiClientConfig,
};
use cloudflare::framework::response::ApiSuccess;
#[cfg(test)]
use mockito;

use crate::{config, daemon};
use crate::daemon::RefreshErrorKind;

pub(crate) fn cloudflare(config: &config::ServiceCloudflareConfig, detected_ip_addr: IpAddr) -> daemon::RefreshResult<()> {
    // Create the cloudflare domain object required to create or update the address
    let detected_dns_content = match detected_ip_addr {
        IpAddr::V4(ip) => DnsContent::A { content: ip },
        IpAddr::V6(ip) => DnsContent::AAAA { content: ip },
    };

    // Initialise the Cloudflare client
    let credentials = Credentials::UserAuthToken { token: config.api_token.clone() };
    let api_client = cf_init_client(credentials)?;

    // Fetch the current DNS record for this name
    let current_dns_record = cf_current_dns_record(&config, &api_client)?;

    // If the record exists
    if let Some(current_dns_record) = current_dns_record {
        // If its the same as the detected, there is nothing to do
        let current_ip_addr = cf_ip_from_record(&current_dns_record);
        if detected_ip_addr == current_ip_addr {
            log::info!("No change required. {} is already set to {}", config.name, current_ip_addr);
        } else {
            log::info!("Updating {} from {} to {}", config.name, current_ip_addr, detected_ip_addr);

            // Update the DNS record
            api_client.request(&dns::UpdateDnsRecord {
                zone_identifier: &config.zone_id,
                identifier: &current_dns_record.id,
                params: dns::UpdateDnsRecordParams { name: &config.name, content: detected_dns_content, proxied: Some(current_dns_record.proxied), ttl: Some(current_dns_record.ttl) },
            })
                .map_err(|e| RefreshErrorKind::ServiceError(Box::new(e)))?;
        }
    } else {
        log::info!("No record for {} exists. Creating as {}", config.name, detected_ip_addr);

        // Create the DNS record
        api_client.request(&dns::CreateDnsRecord {
            zone_identifier: &config.zone_id,
            params: dns::CreateDnsRecordParams { name: &config.name, content: detected_dns_content, proxied: Some(false), ttl: None, priority: None },
        })
            .map_err(|e| RefreshErrorKind::ServiceError(Box::new(e)))?;
    }

    Ok(())
}

// Retrieves the current Cloudflare DNS record for the name of interest
fn cf_current_dns_record(config: &config::ServiceCloudflareConfig, api_client: &HttpApiClient) -> daemon::RefreshResult<Option<DnsRecord>> {
    let current = api_client.request(&dns::ListDnsRecords {
        zone_identifier: &config.zone_id,
        params: dns::ListDnsRecordsParams { name: Some(config.name.to_string()), ..Default::default() },
    })
        .map_err(|e| RefreshErrorKind::ServiceError(Box::new(e)))
        .and_then(|response: ApiSuccess<Vec<DnsRecord>>| {
            Ok(response.result.into_iter()
                .find(|record| {
                    match record.content {
                        DnsContent::A { .. } | DnsContent::AAAA { .. } => true,
                        _ => false
                    }
                }))
        })?;

    Ok(current)
}

// Initialises a Cloudflare API client
#[cfg(not(test))]
fn cf_init_client(credentials: Credentials) -> daemon::RefreshResult<HttpApiClient> {
    let api_client = HttpApiClient::new(credentials, HttpApiClientConfig::default(), Environment::Production)
        .map_err(|e| RefreshErrorKind::RuntimeError(format!("Unable to initialise client - {}", e)))?;
    Ok(api_client)
}

#[cfg(test)]
fn cf_init_client(credentials: Credentials) -> daemon::RefreshResult<HttpApiClient> {
    use reqwest::Url;

    let url = Url::parse(&mockito::server_url()).unwrap();
    let api_client = HttpApiClient::new(credentials, HttpApiClientConfig::default(), Environment::Custom(url))
        .map_err(|e| RefreshErrorKind::RuntimeError(format!("Unable to initialise client - {}", e)))?;
    Ok(api_client)
}

// Extracts the IP address from a DNS record
fn cf_ip_from_record(record: &DnsRecord) -> IpAddr {
    match record.content {
        DnsContent::A { content } => IpAddr::V4(content),
        DnsContent::AAAA { content } => IpAddr::V6(content),
        _ => panic!("Unsupported record type: {:?}", record)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use mockito::Matcher;

    use crate::config::ServiceCloudflareConfig;
    use crate::util::test;

    use super::*;

    // Verify we create a cloudflare DNS record when the address doesn't exist
    #[test]
    fn test_cloudflare_create() {
        let _ = env_logger::try_init();

        let config = ServiceCloudflareConfig {
            api_token: "test_api_token".to_string(),
            zone_id: "test_zone_id".to_string(),
            record_type: "A".to_string(),
            name: "test_name".to_string(),
        };

        // First we list entries in the zone
        let _mock_list = mockito::mock("GET", "/zones/test_zone_id/dns_records")
            .match_query(Matcher::UrlEncoded("name".into(), "test_name".into()))
            .with_status(200)
            .with_body(test::fixture_content("cf_api_list_response_empty.json"))
            .create();

        // Since the address didn't exist, we create it
        let _mock_create = mockito::mock("POST", "/zones/test_zone_id/dns_records")
            .match_body(Matcher::Exact(test::fixture_content("cf_api_create_post_1.2.3.4.json")))
            .with_status(200)
            .with_body(test::fixture_content("cf_api_create_response_1.2.3.4.json"))
            .create();

        // Call with an IP that is created
        cloudflare(&config, IpAddr::from_str("1.2.3.4").unwrap()).unwrap();
    }

    // Verify we update cloudflare DNS records
    #[test]
    fn test_cloudflare_update() {
        let _ = env_logger::try_init();

        let config = ServiceCloudflareConfig {
            api_token: "test_api_token".to_string(),
            zone_id: "test_zone_id".to_string(),
            record_type: "A".to_string(),
            name: "test_name".to_string(),
        };

        // First we list entries in the zone
        let _mock_list = mockito::mock("GET", "/zones/test_zone_id/dns_records")
            .match_query(Matcher::UrlEncoded("name".into(), "test_name".into()))
            .with_status(200)
            .with_body(test::fixture_content("cf_api_list_response_1.1.1.1.json"))
            .create();

        // Since the IP has changed, we update the record
        let _mock_update = mockito::mock("PUT", "/zones/test_zone_id/dns_records/test_record_id")
            .match_body(Matcher::Exact(test::fixture_content("cf_api_update_put_1.2.3.4.json")))
            .with_status(200)
            .with_body(test::fixture_content("cf_api_update_response_1.2.3.4.json"))
            .create();

        // Call with a different detected IP that what is returned in the list
        cloudflare(&config, IpAddr::from_str("1.2.3.4").unwrap()).unwrap();
    }
}
