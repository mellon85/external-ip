use std::fmt::Display;
use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;

/// IP Address family to try to resolve for
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum Family {
    /// Doesn't provide a specific IP family, so it will try all of them
    #[default]
    Any,
    /// Lookup only IPv4 addresses
    IPv4,
    /// Lookup only IPv6 addresses
    IPv6,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP request: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Decode as UTF-8 failed: {0}")]
    DecodeError(#[from] std::str::Utf8Error),
    #[error("Address parsing: {0}")]
    InvalidAddress(#[from] std::net::AddrParseError),
    #[error("DNS resolution failed: {0}")]
    Dns(#[from] hickory_resolver::error::ResolveError),
    #[error("DNS resolution empty")]
    DnsResolutionEmpty,
    #[error("Unsupported family")]
    UnsupportedFamily,
    #[cfg(feature = "igd")]
    #[error("IGD external IP: {0}")]
    IgdExternalIp(#[from] igd::GetExternalIpError),
    #[cfg(feature = "igd")]
    #[error("IGD search {0}")]
    IgdSearch(#[from] igd::SearchError),
}

pub type IpResult = Result<IpAddr, Error>;
pub type IpFuture<'a> = Pin<Box<dyn Future<Output = IpResult> + Send + 'a>>;

/// Interface for any kind of external ip source
#[cfg_attr(test, mockall::automock)]
pub trait Source: Display {
    /// Returns a future that will represent the IP the source obtained
    fn get_ip(&self, family: Family) -> IpFuture<'_>;

    /// Clones the Source into a new Boxed trait object.
    fn box_clone(&self) -> Box<dyn Source>;
}

#[cfg(test)]
impl Display for MockSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockedSource")
    }
}

impl Clone for Box<dyn Source> {
    fn clone(&self) -> Box<dyn Source> {
        self.box_clone()
    }
}
