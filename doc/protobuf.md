# Protobuf

The Google Protocol Buffer support in `rq` is special because Protobuf
requires an external schema to parse messages.

`rq` maintains its own database of Protobuf schemata that is used to
parse messages.  You can add a schema to the database and all
definitions in that schema will be made available.  The schemata all
share the same namespace so you can't provide conflicting definitions
for messages.

## Adding new schemata

Adding new schemata to the database is simple:

    rq protobuf add myschema.proto

This stashes away the schema to be used the next time you run `rq`
with the `-p` flag.

Some schemata need to be in specific directories because of references
by other schema files.  `rq` will by default use the relative file
name specified in the invocation to save the file internally.  That
means that if you call `rq` like so:

    rq protobuf add foo/bar/schema.proto

...then the schema will be stored internally with the given relative
path of `foo/bar/schema.proto`.  You can control this behavior with
the `--base` flag, so this:

    rq protobuf add foo/bar/schema.proto --base foo

...will store the schema as `bar/schema.proto`.

## Deserializing messages

You specify the fully qualified message name when deserializing
Protobuf:

    rq -p .foo.bar.Person

The leading `.` is needed to disambiguate namespace/package aliases,
which are yet to be implemented.
