# external-ip

Finds the current external IP address contacting http and dns external
services.

If at least one of the sources replies the reply with the highest occourrences
will be reported as the IP address.

Three functions provides sets of known working sources.

* get_http_sources
  Returns all known http sources
* get_dns_sources
  Returns all known dns sources
* get_sources
  Returns all sources combined

# Runtime

It requires to run with Tokio runtime due to the depenency on reqwest if you use the HTTP resolver.
The DNS resolver can work with other executors at the moment. (tested with futures)

# Extend

It's possible to extend how the sources dynamically via the API as long as the
Source interface is implemented and it's passed as a boxed trait object.

# Example

```rust
  let sources: external_ip::Sources = external_ip::get_sources();
  let consensus = external_ip::ConsensusBuilder::new()
      .add_sources(sources)
      .build();
  let result = consensus.get_consensus();
  let value = block_on(result);
  assert_ne!(value, None);
```
