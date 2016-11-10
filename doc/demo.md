# Demo

This document showcases some of the things you can do with `rq`.  For
a more in-depth walk-through, see the [tutorial](tutorial.md).

This assumes that `rq` is installed.  See
[installation](installation.md) for more details on how to do that if
you want to follow along.

## Input/Output

`rq` supports many record formats; the default is JSON.  To show how
it works, simply invoke `rq` with no arguments and some JSON as input:

    $ rq <<< 'null true {"a": 2.5}'
    null
    true
    {"a":2.5}

By default, `rq` passes through each input value it sees unmodified.
This is not very useful, however.  `rq` is mostly useful for
performing queries on larger bodies of data.

## Simple selects

Let's first get some example data, by searching on Spotify for tracks
with the query `"rihanna"`.  If you have another big data set you
prefer to use, feel free :)

    curl 'https://api.spotify.com/v1/search?q=rihanna&type=track&limit=50' > data.json

Let's see what we can do with this data.  `rq` works with the concept
of "processes" that send data to each other via "pipes", similar to a
UNIX shell.  A very basic process is the `at` process.  It takes a
path argument that selects a certain field within an object.  In the
JSON object, we want to select the `tracks` key:

    $ rq 'at "tracks.items"' < data.json
    [...]

## Collecting and spreading values

`rq` prefers to work with streams of data instead of arrays as much as
possible, so that you don't have to keep all of the records in memory.

To convert from a stream to an array, you use `collect`, and to convert
from an array to a stream, you use `spread`.

In this case, we want to stream each individual track, so we append `spread`
to the pipeline:

    $ rq 'at "tracks.items"|spread' < data.json
    {...}
    {...}
    ...

## Maps

You can transform each element in a stream with the `map` process. It
takes a function as an argument that is applied to each element.  As a
convenience, if you specify a string instead of a function, it is equivalent
to mapping using the path described by the string (with the same syntax as
for `at`).  As another type of syntactic sugar, you don't have to quote
single word strings.

Let's get the name of all tracks:

    $ rq 'at "tracks.items"|spread|map name' < data.json
    "Consideration"
    "Umbrella"
    ...

## Filters

These are way too tame for us.  We want only the explicit tracks!  We
can use the `filter` process for that.  It takes a predicate argument
that can be a [lodash iteratee][lodash-iteratee] for example.

Let's insert the process before the `map` process:

    $ rq 'at "tracks.items"|spread|filter explicit|map name' < data.json
    "Needed Me"
    "Too Good"
    "Work"
    ...

Looks better!

## Other functions

When we're done with all of these explicit tracks, we want to take a
step back and know the artists that have worked on all of the tracks.

    $ rq 'at "tracks.items"|spread|map artists' < data.json
    [...]
    [...]
    ...

It turns out that a track can have multiple artists, so that we get a
bunch of arrays!  We can solve this by using `map ...|spread` or the
more idiomatic `flatMap ...`.

    $ rq 'at "tracks.items"|spread|flatMap artists|map name' < data.json
    "Rockabye Baby!"
    "J:Kenzo"
    "Rhianna Kenny"
    ...

## Aggregations

How many times has each artist starred on these tracks?  Let's count
(by name for simplicity):

    $ rq 'at "tracks.items"|spread|flatMap artists|countBy name' < data.json
    {"Calvin Harris":2.0,"Drake":7.0,"Eminem":3.0,...}

## Lambdas, sorting and slicing

The results are alphabetical, which is not very useful... Let's first
map them to pairs by using a lambda expression and the lodash
`toPairs` function:

    $ rq 'at "tracks.items"|spread|flatMap artists|countBy name|flatMap (o)=>{_.toPairs(o)}' < data.json
    ["Calvin Harris",2.0]
    ["Drake",7.0]
    ["Eminem",3.0]
    ...

Now we can sort the results descending by the second column:

    $ rq 'at "tracks.items"|spread|flatMap artists|countBy name|flatMap (o)=>{_.toPairs(o)}|orderBy 1 desc' < data.json
    ["Rihanna",50.0]
    ["Drake",7.0]
    ["Eminem",3.0]
    ...

If we want to save the result in a single JSON array, we can use
`collect`:

    $ rq 'at "tracks.items"|spread|flatMap artists|countBy name|flatMap (o)=>{_.toPairs(o)}|orderBy 1 desc|collect' < data.json
    [["Rihanna",50.0],["Drake",7.0],["Eminem",3.0],...]

Looks good!  We are ready to put this data in our web-scale static
responsive blogosphere generator to be used for whatever!

[jsonpath]: http://goessner.net/articles/JsonPath/
[lodash-iteratee]: https://lodash.com/docs#iteratee
