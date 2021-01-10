use std::fmt::Display;
use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;

#[derive(Debug)]
pub enum Error {
    Http(hyper::Error),
    HttpInvalidUri(http::uri::InvalidUri),
    DecodeError(std::str::Utf8Error),
    InvalidAddress(std::net::AddrParseError),
    Dns(trust_dns_resolver::error::ResolveError),
    DnsResolutionEmpty,

    #[cfg(feature = "igd")]
    IgdExternalIp(igd::GetExternalIpError),
    #[cfg(feature = "igd")]
    IgdSearch(igd::SearchError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error {:?}", self)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Http(e) => Some(e),
            Error::HttpInvalidUri(e) => Some(e),
            Error::DecodeError(e) => Some(e),
            Error::InvalidAddress(e) => Some(e),
            Error::Dns(e) => Some(e),
            Error::DnsResolutionEmpty => None,
            #[cfg(feature = "igd")]
            Error::IgdExternalIp(e) => Some(e),
            #[cfg(feature = "igd")]
            Error::IgdSearch(e) => Some(e),
        }
    }
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Error {
        Error::Http(err)
    }
}

impl From<http::uri::InvalidUri> for Error {
    fn from(err: http::uri::InvalidUri) -> Error {
        Error::HttpInvalidUri(err)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Error {
        Error::DecodeError(err)
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(err: std::net::AddrParseError) -> Error {
        Error::InvalidAddress(err)
    }
}

impl From<trust_dns_resolver::error::ResolveError> for Error {
    fn from(err: trust_dns_resolver::error::ResolveError) -> Error {
        Error::Dns(err)
    }
}

#[cfg(feature = "igd")]
impl From<igd::GetExternalIpError> for Error {
    fn from(err: igd::GetExternalIpError) -> Error {
        Error::IgdExternalIp(err)
    }
}

#[cfg(feature = "igd")]
impl From<igd::SearchError> for Error {
    fn from(err: igd::SearchError) -> Error {
        Error::IgdSearch(err)
    }
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
