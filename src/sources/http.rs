use crate::sources::interfaces::{Error, Family, IpFuture, IpResult, Source};
use log::trace;

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
    fn get_ip<'a>(&'a self, family: Family) -> IpFuture<'a> {
        async fn run(_self: &HTTPSource, family: Family) -> IpResult {
            trace!("Contacting {:?}", _self.url);
            if family == Family::IPv6 {
                return Err(Error::UnsupportedFamily)
            }

            let resp = reqwest::get(&_self.url).await?.text().await?;

            Ok(resp.trim().parse()?)
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
        "https://diagnostic.opendns.com/myip",
    ]
    .iter()
    .map(|x| HTTPSource::source(*x))
    .collect()
}
