use crate::sources::interfaces::{Error, Family, IpFuture, IpResult, Source};
use log::trace;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;

pub struct HTTPSourceBuilder {
    url: String,
    timeout: Duration,
    family: Family,
}
impl HTTPSourceBuilder {
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self {
            url: url.into(),
            timeout: Duration::from_secs(30),
            family: Family::Any,
        }
    }
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    pub fn with_supported_family(mut self, family: Family) -> Self {
        self.family = family;
        self
    }
    pub fn build(self) -> HTTPSource {
        let Self {
            url,
            timeout,
            family,
        } = self;
        HTTPSource {
            url,
            timeout,
            family,
        }
    }
}

/// HTTP(s) Source of the external ip
///
/// It expects a URL to contact to retrive in the content of the message the IP
/// without any additional processing (if not trimming the string).
#[derive(Debug, Clone)]
pub struct HTTPSource {
    url: String,
    timeout: Duration,
    family: Family,
}

impl Source for HTTPSource {
    fn get_ip(&self, family: Family) -> IpFuture<'_> {
        async fn run(_self: &HTTPSource, family: Family) -> IpResult {
            if !((_self.family == Family::Any)
                || (family == Family::Any)
                || (_self.family == family))
            {
                return Err(Error::UnsupportedFamily);
            }

            trace!("Contacting {:?}", _self.url);
            let client = reqwest::Client::builder().timeout(_self.timeout);
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
            match (family, parsed_ip) {
                (Family::Any, _)
                | (Family::IPv4, IpAddr::V4(_))
                | (Family::IPv6, IpAddr::V6(_)) => Ok(parsed_ip),
                _ => Err(Error::UnsupportedFamily),
            }
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
        ("https://icanhazip.com/", Family::Any),
        ("https://myexternalip.com/raw", Family::Any),
        ("https://ifconfig.io/ip", Family::Any),
        ("https://ipecho.net/plain", Family::Any),
        ("https://checkip.amazonaws.com/", Family::IPv4),
        ("https://ident.me/", Family::Any),
        ("http://whatismyip.akamai.com/", Family::IPv4),
        ("https://myip.dnsomatic.com/", Family::IPv4),
        ("https://api.ipify.org", Family::IPv4),
        ("https://ifconfig.me/ip", Family::Any),
        ("https://ipinfo.io/ip", Family::IPv4),
        ("https://ip2location.io/ip", Family::Any),
    ]
    .iter()
    .cloned()
    .map(|(url, family)| {
        HTTPSourceBuilder::new(url)
            .with_supported_family(family)
            .build()
    })
    .map(|x| -> Box<dyn Source> { Box::new(x) })
    .collect()
}
