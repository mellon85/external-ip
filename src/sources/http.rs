use crate::sources::interfaces::{Error, Family, IpFuture, IpResult, Source};
use log::trace;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// HTTP(s) Source of the external ip
///
/// It expects a URL to contact to retrive in the content of the message the IP
/// without any additional processing (if not trimming the string).
#[derive(Debug, Clone)]
pub struct HTTPSource {
    url: String,
}

impl HTTPSource {
    fn source<S: Into<String>>(url: S) -> Box<dyn Source> {
        Box::new(HTTPSource { url: url.into() })
    }
}

impl Source for HTTPSource {
    fn get_ip(&self, family: Family) -> IpFuture<'_> {
        async fn run(_self: &HTTPSource, family: Family) -> IpResult {
            trace!("Contacting {:?}", _self.url);
            let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(30));
            let client = match family {
                Family::IPv4 => client.local_address(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
                Family::IPv6 => {
                    client.local_address(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)))
                }
                Family::Any => client,
            }
            .build()?;
            let resp = client.get(&_self.url).send().await?.text().await?;
            let parsed_ip: IpAddr = resp.trim().parse()?;
            if matches!(parsed_ip, IpAddr::V4(_)) && matches!(family, Family::IPv4 | Family::Any) {
                return Ok(parsed_ip);
            }
            if matches!(parsed_ip, IpAddr::V6(_)) && matches!(family, Family::IPv6 | Family::Any) {
                return Ok(parsed_ip);
            }
            Err(Error::UnsupportedFamily)
        }

        Box::pin(run(self, family))
    }

    fn box_clone(&self) -> Box<dyn Source> {
        Box::new(self.clone())
    }
}

impl std::fmt::Display for HTTPSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HttpSource: {}", self.url)
    }
}

/// Returns a collection of HTTP(s) sources to use to retrieve the external ip
pub fn get_http_sources<T>() -> T
where
    T: std::iter::FromIterator<Box<dyn Source>>,
{
    [
        "https://icanhazip.com/",
        "https://myexternalip.com/raw",
        "https://ifconfig.io/ip",
        "https://ipecho.net/plain",
        "https://checkip.amazonaws.com/",
        "https://ident.me/",
        "http://whatismyip.akamai.com/",
        "https://myip.dnsomatic.com/",
        "https://api.ipify.org",
        "https://ifconfig.me/ip",
        "https://ipinfo.io/ip",
    ]
    .iter()
    .map(|x| HTTPSource::source(*x))
    .collect()
}
