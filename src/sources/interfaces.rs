use std::fmt::Display;
use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;

#[derive(Debug)]
pub enum Error {
    Http(hyper::error::Error),
    HttpInvalidUri(http::uri::InvalidUri),
    DecodeError(std::str::Utf8Error),
    InvalidAddress(std::net::AddrParseError),
    Dns(c_ares_resolver::Error),
    DnsResolutionEmpty,
}

impl From<hyper::error::Error> for Error {
    fn from(err: hyper::error::Error) -> Error {
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

impl From<c_ares_resolver::Error> for Error {
    fn from(err: c_ares_resolver::Error) -> Error {
        Error::Dns(err)
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
