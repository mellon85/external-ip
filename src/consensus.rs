use crate::sources;
use futures;
use log::{debug, error};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::net::IpAddr;
use std::option::Option;
use std::vec::Vec;

use crate::sources::Family;

/// Type alias for easier usage of the library
pub type Sources = Vec<Box<dyn sources::Source>>;

use std::default::Default;

/// Policies for Consensus resolution
#[derive(Debug, Copy, Clone)]
pub enum Policy {
    /// Requires all sources to be queried, it will ignore the sources returning errors but and it
    /// will return the IP with the most replies as the result.
    All,
    /// Will test the sources one by one in order until there's one success and will return it as
    /// the result.
    First,
    /// Will test the sources one by one in random order until there's one success and will return
    /// it as the result.
    Random,
}

impl Default for Policy {
    fn default() -> Self {
        Policy::Random
    }
}

/// Consensus system that aggregates the various sources of information and returns the most common
/// reply
pub struct Consensus {
    voters: Sources,
    policy: Policy,
    family: Family,
}

/// Consensus builder
pub struct ConsensusBuilder {
    voters: Sources,
    policy: Policy,
    family: Family,
}

impl ConsensusBuilder {
    pub fn new() -> ConsensusBuilder {
        ConsensusBuilder {
            voters: vec![],
            policy: Policy::default(),
            family: Family::default(),
        }
    }

    /// Adds sources to the builder
    ///
    /// # Arguments
    ///
    /// * `source` - Iterable of sources to add
    pub fn add_sources<T>(mut self, source: T) -> ConsensusBuilder
    where
        T: IntoIterator<Item = Box<dyn sources::Source>>,
    {
        self.voters.extend(source);
        self
    }

    pub fn policy(mut self, policy: Policy) -> ConsensusBuilder {
        self.policy = policy;
        self
    }

    pub fn family(mut self, family: Family) -> ConsensusBuilder {
        self.family = family;
        self
    }

    /// Returns the configured consensus struct from the builder
    pub fn build(self) -> Consensus {
        Consensus {
            voters: self.voters,
            policy: self.policy,
            family: self.family,
        }
    }
}

impl Consensus {
    /// Returns the IP address it found or None if no source worked.
    pub async fn get_consensus(&self) -> Option<IpAddr> {
        match self.policy {
            Policy::All => self.all().await,
            Policy::First => self.first().await,
            Policy::Random => self.random().await,
        }
    }

    async fn all(&self) -> Option<IpAddr> {
        let results =
            futures::future::join_all(self.voters.iter().map(|voter| voter.get_ip(self.family)))
                .await;

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

    async fn first(&self) -> Option<IpAddr> {
        for voter in &self.voters {
            let result = voter.get_ip(self.family).await;
            debug!("Results {:?}", result);
            if result.is_ok() {
                return Some(result.unwrap());
            }
        }
        debug!("Tried all sources");
        None
    }

    async fn random(&self) -> Option<IpAddr> {
        let mut rng = rand::thread_rng();
        for voter in self.voters.choose_multiple(&mut rng, self.voters.len()) {
            let result = voter.get_ip(self.family).await;
            debug!("Results {:?}", result);
            if result.is_ok() {
                return Some(result.unwrap());
            }
        }
        debug!("Tried all sources");
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::sources::MockSource;
    use mockall::predicate::eq;
    use std::net::Ipv4Addr;
    use tokio_test::block_on;

    const IP0: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

    fn make_success(ip: IpAddr) -> Box<dyn sources::Source> {
        let mut mock = MockSource::new();
        mock.expect_get_ip()
            .with(eq(Family::Any))
            .times(1)
            .returning(move |_| Box::pin(futures::future::ready(Ok(ip))));
        Box::new(mock)
    }

    fn make_fail() -> Box<dyn sources::Source> {
        let mut mock = MockSource::new();
        mock.expect_get_ip()
        .with(eq(Family::Any))
        .times(1)
        .returning(move |_| {
            let invalid_ip: Result<IpAddr, std::net::AddrParseError> = "x.0.0.0".parse();
            Box::pin(futures::future::ready(Err(sources::Error::InvalidAddress(
                invalid_ip.err().unwrap(),
            ))))
        });
        Box::new(mock)
    }

    fn make_untouched() -> Box<dyn sources::Source> {
        let mut mock = MockSource::new();
        mock.expect_get_ip().with(eq(Family::Any)).times(0);
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
    fn test_all_success_multiple_same() {
        let consensus = ConsensusBuilder::new()
            .add_sources(vec![make_success(IP0), make_success(IP0)])
            .policy(Policy::All)
            .build();

        let result = consensus.get_consensus();
        let value = block_on(result);
        assert_eq!(Some(IP0), value);
    }

    #[test]
    fn test_all_success_multiple_same_diff() {
        let ip2 = "0.0.0.1".parse().expect("valid ip");
        let consensus = ConsensusBuilder::new()
            .policy(Policy::All)
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
    fn test_all_success_multiple_with_fails() {
        let consensus = ConsensusBuilder::new()
            .add_sources(vec![make_success(IP0), make_fail()])
            .policy(Policy::All)
            .build();
        let result = consensus.get_consensus();
        let value = block_on(result);
        assert_eq!(Some(IP0), value);
    }

    #[test]
    fn test_only_failures() {
        for policy in [Policy::All, Policy::Random, Policy::First].iter() {
            let consensus = ConsensusBuilder::new()
                .add_sources(vec![make_fail()])
                .policy(*policy)
                .build();
            let result = consensus.get_consensus();
            let value = block_on(result);
            assert_eq!(None, value);
        }
    }

    #[test]
    fn test_add_sources_multiple_times() {
        let consensus = ConsensusBuilder::new()
            .add_sources(vec![make_fail()])
            .add_sources(vec![make_success(IP0)])
            .policy(Policy::All)
            .build();
        let result = consensus.get_consensus();
        let value = block_on(result);
        assert_eq!(Some(IP0), value);
    }

    #[test]
    fn test_first_success_multiple_with_fails() {
        let consensus = ConsensusBuilder::new()
            .add_sources(vec![make_fail(), make_success(IP0)])
            .policy(Policy::First)
            .build();
        let result = consensus.get_consensus();
        let value = block_on(result);
        assert_eq!(Some(IP0), value);
    }

    #[test]
    fn test_first_success_with_first_success() {
        let consensus = ConsensusBuilder::new()
            .add_sources(vec![make_success(IP0), make_untouched()])
            .policy(Policy::First)
            .build();
        let result = consensus.get_consensus();
        let value = block_on(result);
        assert_eq!(Some(IP0), value);
    }
}
