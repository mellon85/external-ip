use tokio_test::block_on;

#[test]
fn http_get_ip() {
    let sources: external_ip::Sources = external_ip::get_http_sources();
    let consensus = external_ip::ConsensusBuilder::new()
        .add_sources(sources)
        .build();
    let result = consensus.get_consensus();
    let value = block_on(result);
    assert_ne!(value, None);
}

#[test]
#[should_panic]
fn requires_tokio() {
    let sources: external_ip::Sources = external_ip::get_http_sources();
    let consensus = external_ip::ConsensusBuilder::new()
        .add_sources(sources)
        .build();
    let result = consensus.get_consensus();

    let value = futures::executor::block_on(result);
    assert_ne!(value, None);
}
