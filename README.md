# `rq` [![Build Status](https://travis-ci.org/dflemstr/rq.svg?branch=master)](https://travis-ci.org/dflemstr/rq) [![codecov](https://codecov.io/gh/dflemstr/rq/branch/master/graph/badge.svg)](https://codecov.io/gh/dflemstr/rq) [![Language (Rust)](https://img.shields.io/badge/powered_by-Rust-blue.svg)](http://www.rust-lang.org/)

This is the home of the tool called `rq` (record query).  It's a tool
that's used for performing queries on streams of records in various
formats.

The goal is to make ad-hoc exploration of data sets easy without
having to use more heavy-weight tools like SQL/MapReduce/custom
programs.  `rq` fills a similar niche as tools like `awk` or `sed`,
but works with structured (record) data instead of text.

It was created with love out of the best parts of Rust, C and
Javascript, and is distributed as a dependency-free binary on many
operating systems and architectures.

## Quick links

  - [Installation](doc/installation.md) — How to install `rq`.
  - [Tutorial](doc/tutorial.md) — Learn `rq` from scratch.
  - [Demo](doc/demo.md) — Showing off misc. `rq` features.
  - [Process quick reference](https://dflemstr.github.io/rq/js/global.html)
    — Quickly find a process you need.
  - [Protobuf](doc/protobuf.md) — Configure Protobuf specifics.
  - [Development](CONTRIBUTING.md) — Contribute to `rq`.

## Format support status

| Format                  | Read | Write |
|-------------------------|------|-------|
| Apache Avro             | ✔️    | ✖️     |
| CBOR                    | ✔️    | ✔️     |
| HJSON                   | ✔️    | ✔️     |
| JSON                    | ✔️    | ✔️     |
| MessagePack             | ✔️    | ✔️     |
| Google Protocol Buffers | ✔️    | ✖️     |
| YAML                    | ✔️    | ✔️     |
| TOML                    | ✔️    | ✔️     |
