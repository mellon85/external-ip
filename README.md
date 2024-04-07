# external-ip

[![Build Status](https://travis-ci.com/mellon85/external-ip.svg?branch=master)](https://travis-ci.com/mellon85/external-ip) 

Finds the current external IP address contacting http and dns external
services.

If at least one of the sources replies the reply with the highest occurrences
will be reported as the IP address.

Three functions provides sets of known working sources.

* `get_http_sources`
  Returns all known http sources
* `get_dns_sources`
  Returns all known dns sources
* `get_sources`
  Returns all sources combined

Additionally a single igd source can be instantiated if the feature is enabled
(`discover_igd`), to retrieve the IP from an home router.
If the feature is enabled `get_sources` will return it as a source too.


# Runtime

It requires to run with Tokio runtime due to the dependency on hyper if you use the HTTP resolver.
The DNS resolver can work with other executors at the moment. (tested with futures)

# Extend

It's possible to extend how the sources dynamically via the API as long as the
Source interface is implemented and it's passed as a boxed trait object.

# Example

For ease of use a single async function is enough to obtain the IP trying with
all the default sources enabled

```rust
  let result = external_ip::get_ip();
  let value : Option<IpAddr> = block_on(result);
```

This is the same as doing

```rust
  let sources: external_ip::Sources = external_ip::get_sources();
  let consensus = external_ip::ConsensusBuilder::new()
      .add_sources(sources)
      .build();
  let result = consensus.get_consensus();
  let value : Option<IpAddr>  = block_on(result);
```

# Policies

The library supports 3 consensus policies. The default policy is Random

- All
  Query all sources in parallel and return the most common response
- First
  Query the sources one by one and return the first success
- Random
  Query the sources one by one in random order and return the first success

# Families

It's possible to select a specific address family to resolve to and all resolver will try to resolve to that or fail.
- All
- IPv4
- IPv6

# Changelog

## v1

- Initial release

## v2

- Based on trust dns instead of c-ares. Now requires less external dependencies (only ssl at the moment)

## v3

- The default policy is now random

## v4

- Consensus is now not consumed when used

## v5

- Added ip family selection option
- Support http sources with IPv6
