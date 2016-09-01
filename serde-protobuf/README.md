# serde_protobuf [![Crates.io](https://img.shields.io/crates/v/serde_protobuf.svg?maxAge=2592000)]()

Support for [Google protocol buffers][1] in combination with `serde`.

The crate is split up into several logical parts.

  * The [`descriptor`](https://dflemstr.github.io/rq/serde_protobuf/descriptor/index.html) module provides an API for managing dynamically
    loaded protocol buffer schemata.
  * The [`value`](https://dflemstr.github.io/rq/serde_protobuf/value/index.html) module provides structs that can hold any raw protocol
    buffer decoded data (but the representation is heavily coupled with a schema).
  * The [`de`](https://dflemstr.github.io/rq/serde_protobuf/de/index.html) module can be used to deserialize binary encoded protocol buffer
    messages given some schema descriptors.

Serialization is not yet implemented in this version.

[1]: https://developers.google.com/protocol-buffers/
