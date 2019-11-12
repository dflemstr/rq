# Tutorial

This assumes that `rq` is installed.  See
[installation](installation.md) for more details on how to do that if
you want to follow along.

## Input/Output

`rq` reads record data from stdin, and writes transformed data to
stdout.  By default, it uses JSON for the input and output format, and
returns each input record unmodified:

    $ rq <<< 'null true {"a": 2.5}'
    null
    true
    {"a":2.5}

## Highlighting

This Markdown document doesn't do the `rq` output justice.  The output
of `rq` is actually very colorful!

![highlighting](image/highlighting.png)

## Record formats

You can configure the input and output formats to use with flags (see
`rq --help` for details).  A lower-case single-letter flag sets the
input format, and an upper-case single-letter flag sets the output
format.  For example, to read JSON and output CBOR, pass `-jC` and to
read CBOR and output JSON, pass `-cJ`.  This can be used to build a
not-very-useful conversion pipeline that round-trips to CBOR (maybe
you could pipe it through `gzip` and `ssh` in-between and it might be
worth it):

    $ (rq -jC | rq -cJ) <<< 'null true {"a": 2.5}'
    null
    true
    {"a":2.5}

Some format flags take an argument to configure them, for example
Google Protocol Buffers:

    $ rq protobuf add example.proto
    $ rq -p .example.Person < person.pb
    {"name":"John","age":34}
