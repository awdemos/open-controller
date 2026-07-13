use regex::Regex;
use reqwest;
use rmcp::schemars;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::net::IpAddr;
use url::Url;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScrapeArgs {
    pub url: String,
    #[serde(default)]
    pub query: Option<String>,
}

/// Reserved / private IP ranges that the scrape tool must never contact by default.
fn is_private_or_reserved_addr(addr: IpAddr) -> bool {
    match addr {
        IpAddr::V4(ip) => {
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_multicast()
                || ip.is_broadcast()
                || ip.is_unspecified()
                || ip.octets()[0] == 0
                // 169.254.169.254 cloud metadata
                || ip.octets() == [169, 254, 169, 254]
        }
        IpAddr::V6(ip) => {
            ip.is_loopback()
                || ip.is_multicast()
                || ip.is_unspecified()
                || ip.to_canonical().to_string().starts_with("fc")
                || ip.to_canonical().to_string().starts_with("fd")
        }
    }
}

fn is_allowed_url(url: &Url, allowlist: &[Regex]) -> bool {
    if allowlist.is_empty() {
        return true;
    }
    let url_str = url.as_str();
    allowlist.iter().any(|r| r.is_match(url_str))
}

fn has_ssrf_risk(url: &Url) -> bool {
    match url.host() {
        Some(url::Host::Domain(domain)) => {
            let lower = domain.to_lowercase();
            lower == "localhost"
                || lower.ends_with(".local")
                || lower == "metadata.google.internal"
                || lower.ends_with(".metadata.google.internal")
        }
        Some(url::Host::Ipv4(ip)) => is_private_or_reserved_addr(IpAddr::V4(ip)),
        Some(url::Host::Ipv6(ip)) => is_private_or_reserved_addr(IpAddr::V6(ip)),
        None => true,
    }
}

pub fn validate_scrape_url(url_str: &str, allowlist: &[Regex]) -> anyhow::Result<Url> {
    let url = Url::parse(url_str).map_err(|e| anyhow::anyhow!("invalid URL: {}", e))?;

    if url.scheme() != "http" && url.scheme() != "https" {
        anyhow::bail!("only http and https URLs are allowed");
    }

    // Reject credentials embedded in the URL.
    if url.username() != "" || url.password().is_some() {
        anyhow::bail!("URLs containing credentials are not allowed");
    }

    if !is_allowed_url(&url, allowlist) {
        anyhow::bail!("URL is not allowed by the configured scrape allowlist");
    }

    // When no explicit allowlist is configured, block private / localhost / metadata targets.
    if allowlist.is_empty() && has_ssrf_risk(&url) {
        anyhow::bail!("private / localhost / metadata URLs are not allowed by default");
    }

    Ok(url)
}

pub async fn run_scrape(args: &ScrapeArgs, allowlist: &[Regex]) -> anyhow::Result<String> {
    let url = validate_scrape_url(&args.url, allowlist)?;
    let body = reqwest::get(url).await?.text().await?;
    if let Some(query) = args.query.as_deref() {
        let doc = Html::parse_document(&body);
        let selector =
            Selector::parse(query).map_err(|e| anyhow::anyhow!("invalid CSS selector: {:?}", e))?;
        let texts: Vec<String> = doc
            .select(&selector)
            .map(|e| e.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Ok(texts.join("\n"))
    } else {
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_private_and_metadata_urls_by_default() {
        let empty: Vec<Regex> = vec![];
        assert!(validate_scrape_url("http://127.0.0.1/", &empty).is_err());
        assert!(validate_scrape_url("http://192.168.1.1/", &empty).is_err());
        assert!(validate_scrape_url("http://10.0.0.1/", &empty).is_err());
        assert!(validate_scrape_url("http://172.16.0.1/", &empty).is_err());
        assert!(validate_scrape_url("http://169.254.169.254/latest/meta-data/", &empty).is_err());
        assert!(validate_scrape_url("http://localhost/foo", &empty).is_err());
        assert!(validate_scrape_url("ftp://example.com/file", &empty).is_err());
    }

    #[test]
    fn allows_public_http_urls() {
        let empty: Vec<Regex> = vec![];
        assert!(validate_scrape_url("http://example.com/", &empty).is_ok());
        assert!(validate_scrape_url("https://example.com/path?q=1", &empty).is_ok());
    }

    #[test]
    fn allowlist_can_override_private_block() {
        let allowlist = vec![Regex::new(r"^http://127\.0\.0\.1:").unwrap()];
        assert!(validate_scrape_url("http://127.0.0.1:8080/", &allowlist).is_ok());
    }

    #[test]
    fn enforces_url_allowlist() {
        let allowlist = vec![Regex::new(r"^https://example\.com/").unwrap()];
        assert!(validate_scrape_url("https://example.com/page", &allowlist).is_ok());
        assert!(validate_scrape_url("https://other.com/page", &allowlist).is_err());
    }
}
