<p align="center"><img src="doc/image/example-480.png" alt="example"></p>

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
  - [Process quick reference](https://dflemstr.github.io/rq/js/module-prelude.html)
    — Quickly find a process you need.
  - [Protobuf](doc/protobuf.md) — Configure Protobuf specifics.
  - [Development](CONTRIBUTING.md) — Contribute to `rq`.

## Platform support status

<table>
  <thead>
    <tr>
      <th rowspan="2">OS</th>
      <th colspan="2">Intel x86</th>
      <th colspan="3">ARM</th>
    </tr>
    <tr>
      <th>i686</th>
      <th>x86_64</th>
      <th>v6<a href="#user-content-foot1"><sup>1</sup></a></th>
      <th>v6 HF<a href="#user-content-foot2"><sup>2</sup></a></th>
      <th>v7<a href="#user-content-foot3"><sup>3</sup></a></th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <th>Linux <code>glibc</code><a href="#user-content-foot4"><sup>4</sup></a></th>
      <td>
        <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/i686-unknown-linux-gnu/rq">
          <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/i686-unknown-linux-gnu/badge-small.svg?v=3"
               alt="i686-unknown-linux-gnu">
        </a>
      </td>
      <td>
        <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/x86_64-unknown-linux-gnu/rq">
          <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/x86_64-unknown-linux-gnu/badge-small.svg?v=3"
               alt="x86_64-unknown-linux-gnu">
        </a>
      </td>
      <td>
        <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/arm-unknown-linux-gnueabi/rq">
          <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/arm-unknown-linux-gnueabi/badge-small.svg?v=3"
               alt="arm-unknown-linux-gnueabi">
        </a>
      </td>
      <td>
        <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/arm-unknown-linux-gnueabihf/rq">
          <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/arm-unknown-linux-gnueabihf/badge-small.svg?v=3"
               alt="arm-unknown-linux-gnueabihf">
        </a>
      </td>
      <td>
        <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/armv7-unknown-linux-gnueabihf/rq">
          <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/armv7-unknown-linux-gnueabihf/badge-small.svg?v=3"
               alt="armv7-unknown-linux-gnueabihf">
        </a>
      </td>
    </tr>
    <tr>
      <th>Linux <code>musl</code><a href="#user-content-foot5"><sup>5</sup></a></th>
      <td>
        <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/i686-unknown-linux-musl/rq">
          <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/i686-unknown-linux-musl/badge-small.svg?v=3"
               alt="i686-unknown-linux-musl">
        </a>
      </td>
      <td>
        <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/x86_64-unknown-linux-musl/rq">
          <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/x86_64-unknown-linux-musl/badge-small.svg?v=3"
               alt="x86_64-unknown-linux-musl">
        </a>
      </td>
      <td>
        <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/arm-unknown-linux-musleabi/rq">
          <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/arm-unknown-linux-musleabi/badge-small.svg?v=3"
               alt="arm-unknown-linux-musleabi">
        </a>
      </td>
      <td>
        <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/arm-unknown-linux-musleabihf/rq">
          <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/arm-unknown-linux-musleabihf/badge-small.svg?v=3"
               alt="arm-unknown-linux-musleabihf">
        </a>
      </td>
      <td>
        <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/armv7-unknown-linux-musleabihf/rq">
          <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/armv7-unknown-linux-musleabihf/badge-small.svg?v=3"
               alt="armv7-unknown-linux-musleabihf">
        </a>
      </td>
    </tr>
    <tr>
      <th>Mac OS X</th>
      <td>
        <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/i686-apple-darwin/rq">
          <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/i686-apple-darwin/badge-small.svg?v=3"
               alt="i686-apple-darwin">
        </a>
      </td>
      <td>
        <a href="https://s3-eu-west-1.amazonaws.com/record-query/record-query/x86_64-apple-darwin/rq">
          <img src="https://s3-eu-west-1.amazonaws.com/record-query/record-query/x86_64-apple-darwin/badge-small.svg?v=3"
               alt="x86_64-apple-darwin">
        </a>
      </td>
      <td>
        &nbsp;
      </td>
      <td>
        &nbsp;
      </td>
      <td>
        &nbsp;
      </td>
    </tr>
  </tbody>
</table>

<a name="foot1"><sup>1</sup></a> For example Raspberry Pi 1 (A and B) running Raspbian. <br/>
<a name="foot2"><sup>2</sup></a> For example Raspberry Pi 1 (A and B) running Arch Linux. <br/>
<a name="foot3"><sup>3</sup></a> For example Raspberry Pi 2+. <br/>
<a name="foot4"><sup>4</sup></a> Requires a recent version of `glibc`/`libstdc++`, so use musl if possible. <br/>
<a name="foot5"><sup>5</sup></a> Completely statically linked; only depends on a recent kernel version.

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
