//! Crate to figure out the system external IP
mod consensus;
mod sources;

pub use consensus::*;
pub use sources::*;

use std::net::IpAddr;

/// For ease of use a single async function is enough to obtain the IP trying with all the default
/// sources enabled.
pub async fn get_ip() -> Option<IpAddr> {
    let sources: Sources = get_sources();
    let consensus = ConsensusBuilder::new()
        .add_sources(sources)
        .build();
    consensus.get_consensus().await
}
