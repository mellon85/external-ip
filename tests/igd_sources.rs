#[test]
#[cfg(feature = "igd")]
#[ignore]
fn igd_get_ip() {
    use external_ip;
    use tokio_test::block_on;

    let sources: external_ip::Sources = vec![external_ip::IGD::source()];
    let consensus = external_ip::ConsensusBuilder::new()
        .add_sources(sources)
        .build();
    let result = consensus.get_consensus();
    let value = block_on(result);
    assert_ne!(value, None);
}
