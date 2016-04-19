# rq [![Build Status](https://travis-ci.org/dflemstr/rq.svg?branch=master)](https://travis-ci.org/dflemstr/rq)

This is the home of the record query tool called `rq`.  I created it
out of frustration while manipulating data records on hundreds of
different machines and never having the right tools available.

This is one of my hack projects for personal learning so the code
quality is intentionally pretty low.  Everything is currently a work
in progress so don't expect to be able to use `rq` for productivity
right now.

`rq` is similar to `awk` or `jq` but supports more record formats and
operations.

Currently, the following input and output formats are supported:

  - Google Protocol Buffers
  - JSON

## Installation

The tool is distributed as a native statically linked binary.  The
quickest way to install it is:

    curl -sSLf sh.dflemstr.name/rq | sh
