//! Crate to figure out the system external IP
mod consensus;
mod sources;

pub use consensus::{Consensus, ConsensusBuilder, Sources};
pub use sources::{get_http_sources, HTTPSource, Source};
