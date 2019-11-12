# `rq` [![Build Status](https://travis-ci.org/dflemstr/rq.svg?branch=master)](https://travis-ci.org/dflemstr/rq) [![Crates.io](https://img.shields.io/crates/v/record-query.svg)](https://crates.io/crates/record-query) [![Language (Rust)](https://img.shields.io/badge/powered_by-Rust-blue.svg)](http://www.rust-lang.org/)

**WARNING**: `rq` is in very low maintenance mode while I (@dflemstr) focus
on other things (such as my day job).  Pull requests are welcome but I won't
guarantee that I will fix reported issues.

This is the home of the tool called `rq` (record query).  It's a tool
that's used for performing queries on streams of records in various
formats.

The goal is to make ad-hoc exploration of data sets easy without
having to use more heavy-weight tools like SQL/MapReduce/custom
programs.  `rq` fills a similar niche as tools like `awk` or `sed`,
but works with structured (record) data instead of text.

It was created with love out of the best parts of Rust, C and
JavaScript, and is distributed as a dependency-free binary on many
operating systems and architectures.

## Quick links

  - [Installation](doc/installation.md) — How to install `rq`.
  - [Tutorial](doc/tutorial.md) — Learn `rq` from scratch.
  - [Protobuf](doc/protobuf.md) — Configure Protobuf specifics.
  - [Development](CONTRIBUTING.md) — Contribute to `rq`.

## Format support status

| Format                  | Read | Write |
|-------------------------|------|-------|
| Apache Avro             | ✔️    | ✔️     |
| CBOR                    | ✔️    | ✔️     |
| HJSON                   | ✔️    | ✔️     |
| JSON                    | ✔️    | ✔️     |
| MessagePack             | ✔️    | ✔️     |
| Google Protocol Buffers | ✔️    | ✖️     |
| YAML                    | ✔️    | ✔️     |
| TOML                    | ✔️    | ✔️     |
| Raw (plain text)        | ✔️    | ✔️     |
| CSV                     | ✔️    | ✔️     |
