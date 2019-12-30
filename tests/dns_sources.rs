use external_ip;

use tokio_test::block_on;

#[test]
fn dns_get_ip() {
    let sources: external_ip::Sources = external_ip::get_dns_sources();
    let consensus = external_ip::ConsensusBuilder::new()
        .add_sources(sources)
        .build();
    let result = consensus.get_consensus();
    let value = block_on(result);
    assert_ne!(value, None);
}

#[test]
fn requires_tokio() {
    let sources: external_ip::Sources = external_ip::get_dns_sources();
    let consensus = external_ip::ConsensusBuilder::new()
        .add_sources(sources)
        .build();
    let result = consensus.get_consensus();

    let value = futures::executor::block_on(result);
    assert_ne!(value, None);
}
