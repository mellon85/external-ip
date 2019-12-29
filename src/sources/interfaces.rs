use std::fmt::Display;
use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;

#[derive(Debug)]
pub enum Error {
    Http(reqwest::Error),
    InvalidAddress(std::net::AddrParseError),
    Dns(c_ares_resolver::Error),
    DnsResolutionEmpty,
}

pub type IpResult = Result<IpAddr, Error>;
pub type IpFuture<'a> = Pin<Box<dyn Future<Output = IpResult> + Send + 'a>>;

/// Interface for any kind of external ip source
#[cfg_attr(test, mockall::automock)]
pub trait Source: Display {
    /// Returns a future that will represent the IP the source obtained
    fn get_ip<'a>(&'a self) -> IpFuture<'a>;

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
