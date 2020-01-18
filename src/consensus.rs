use crate::sources;
use futures;
use log::{debug, error};
use std::collections::HashMap;
use std::net::IpAddr;
use std::option::Option;
use std::vec::Vec;

/// Type alias for easier usage of the library
pub type Sources = Vec<Box<dyn sources::Source>>;

/// Consensus system that aggregates the various sources of information and returns the most common reply
pub struct Consensus {
    voters: Sources,
}

/// Consensus builder
pub struct ConsensusBuilder {
    voters: Sources,
}

impl ConsensusBuilder {
    pub fn new() -> ConsensusBuilder {
        ConsensusBuilder { voters: vec![] }
    }

    /// Adds sources to the builder
    ///
    /// # Arguments
    ///
    /// * `source` - Iterable of sources to add
    pub fn add_sources<T>(&mut self, source: T) -> &mut ConsensusBuilder
    where
        T: IntoIterator<Item = Box<dyn sources::Source>>,
    {
        self.voters.extend(source);
        self
    }

    /// Returns the configured consensus struct from the builder
    pub fn build(&self) -> Consensus {
        Consensus {
            voters: self.voters.clone(),
        }
    }
}

impl Consensus {
    /// Returns the IP address it found or None if no source worked.
    pub async fn get_consensus(self) -> Option<IpAddr> {
        let results =
            futures::future::join_all(self.voters.iter().map(|voter| voter.get_ip())).await;

        debug!("Results {:?}", results);
        let mut accumulate = HashMap::new();
        for (pos, result) in results.into_iter().enumerate() {
            match result {
                Ok(result) => {
                    accumulate
                        .entry(result)
                        .and_modify(|c| *c += 1)
                        .or_insert(1);
                }
                Err(err) => error!("Source {} failed {:?}", self.voters[pos], err),
            };
        }

        let mut ordered_output: Vec<_> = accumulate.iter().collect();
        ordered_output.sort_unstable_by(|(_, a), (_, b)| a.cmp(b));
        debug!("Sorted results {:?}", ordered_output);

        ordered_output.pop().map(|x| *x.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::sources::MockSource;
    use std::net::Ipv4Addr;
    use tokio_test::block_on;

    const IP0: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

    fn make_success(ip: IpAddr) -> Box<dyn sources::Source> {
        let mut mock = MockSource::new();
        mock.expect_box_clone().times(1).returning(move || {
            let mut clone_mock = MockSource::new();
            clone_mock
                .expect_get_ip()
                .times(1)
                .returning(move || Box::pin(futures::future::ready(Ok(ip))));
            Box::new(clone_mock)
        });
        Box::new(mock)
    }

    fn make_fail() -> Box<dyn sources::Source> {
        let mut mock = MockSource::new();
        mock.expect_box_clone().times(1).returning(move || {
            let mut clone_mock = MockSource::new();
            clone_mock.expect_get_ip().times(1).returning(move || {
                let invalid_ip: Result<IpAddr, std::net::AddrParseError> = "x.0.0.0".parse();
                Box::pin(futures::future::ready(Err(sources::Error::InvalidAddress(
                    invalid_ip.err().unwrap(),
                ))))
            });
            Box::new(clone_mock)
        });
        Box::new(mock)
    }

    #[test]
    fn test_success() {
        let sources: Sources = vec![make_success(IP0)];
        let consensus = ConsensusBuilder::new().add_sources(sources).build();
        let result = consensus.get_consensus();
        let value = block_on(result);
        assert_eq!(Some(IP0), value);
    }

    #[test]
    fn test_success_multiple_same() {
        let consensus = ConsensusBuilder::new()
            .add_sources(vec![make_success(IP0), make_success(IP0)])
            .build();

        let result = consensus.get_consensus();
        let value = block_on(result);
        assert_eq!(Some(IP0), value);
    }

    #[test]
    fn test_success_multiple_same_diff() {
        let ip2 = "0.0.0.1".parse().expect("valid ip");
        let consensus = ConsensusBuilder::new()
            .add_sources(vec![
                make_success(IP0),
                make_success(IP0),
                make_success(ip2),
            ])
            .build();

        let result = consensus.get_consensus();
        let value = block_on(result);
        assert_eq!(Some(IP0), value);
    }

    #[test]
    fn test_success_multiple_with_fails() {
        let result = ConsensusBuilder::new()
            .add_sources(vec![make_success(IP0), make_fail()])
            .build()
            .get_consensus();
        let value = block_on(result);
        assert_eq!(Some(IP0), value);
    }

    #[test]
    fn test_all_fails() {
        let result = ConsensusBuilder::new()
            .add_sources(vec![make_fail()])
            .build()
            .get_consensus();
        let value = block_on(result);
        assert_eq!(None, value);
    }

    #[test]
    fn test_add_sources_multiple_times() {
        let result = ConsensusBuilder::new()
            .add_sources(vec![make_fail()])
            .add_sources(vec![make_success(IP0)])
            .build()
            .get_consensus();
        let value = block_on(result);
        assert_eq!(Some(IP0), value);
    }
}
