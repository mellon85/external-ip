//! Crate to figure out the system external IP
mod consensus;
mod sources;

pub use consensus::{Consensus, ConsensusBuilder, Sources};
pub use sources::{get_dns_sources, get_http_sources, get_sources, DNSSource, HTTPSource, Source};

