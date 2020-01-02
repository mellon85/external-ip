mod dns;
mod http;

#[cfg(feature = "igd")]
mod igd;

mod interfaces;

pub use self::dns::{get_dns_sources, DNSSource, QueryType};
pub use self::http::{get_http_sources, HTTPSource};
#[cfg(feature = "igd")]
pub use self::igd::IGD;
pub use interfaces::*;

/// Returns a collection of all possible sources
pub fn get_sources<T>() -> T
where
    T: std::iter::FromIterator<Box<dyn Source>>,
{
    let h: Vec<_> = get_http_sources();
    let d: Vec<_> = get_dns_sources();

    let sources = h.into_iter().chain(d.into_iter());

    #[cfg(feature = "igd")]
    let sources = sources.chain(std::iter::once(IGD::source()));

    sources.into_iter().collect()
}
