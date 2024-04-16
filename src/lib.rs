//! Crate to figure out the system external IP
mod consensus;
mod sources;

pub use consensus::*;
pub use sources::*;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// For ease of use a single async function is enough to obtain the IP trying with all the default
/// sources enabled.
#[deprecated]
pub async fn get_ip() -> Option<IpAddr> {
    let sources: Sources = get_sources();
    let consensus = ConsensusBuilder::new().add_sources(sources).build();
    consensus.get_consensus().await
}

/// For ease of use a single async function is enough to obtain the IPv4 trying with all the default
/// sources enabled.
pub async fn get_ipv4() -> Option<Ipv4Addr> {
    let sources: Sources = get_sources();
    let consensus = ConsensusBuilder::new()
        .family(Family::IPv4)
        .add_sources(sources)
        .build();
    consensus.get_consensus().await.map(|addr| {
        let IpAddr::V4(ipv4) = addr else {
            panic!("Consensus returned a non-IPv4 address")
        };
        ipv4
    })
}

/// For ease of use a single async function is enough to obtain the IPv6 trying with all the default
/// sources enabled.
pub async fn get_ipv6() -> Option<Ipv6Addr> {
    let sources: Sources = get_sources();
    let consensus = ConsensusBuilder::new()
        .family(Family::IPv6)
        .add_sources(sources)
        .build();
    consensus.get_consensus().await.map(|addr| {
        let IpAddr::V6(ipv6) = addr else {
            panic!("Consensus returned a non-IPv6 address")
        };
        ipv6
    })
}
