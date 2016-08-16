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
UNIX shell.  A very basic process is the `select` process.  It takes a
path argument that can be a [JSONPath][jsonpath] for example.  In the
JSON object, we want to select the `tracks` key, then for each element
in the `items` array select the `name` key:

    $ rq 'select "$.tracks.items[*].name"' < data.json
    "Consideration"
    "Umbrella"
    ...

## Filters and maps

These are way too tame for us.  We want only the explicit tracks!  We
can use the `filter` process for that.  It takes a predicate argument
that can be a [lodash iteratee][lodash-iteratee] for example.

We can also use the `map` process, which also takes a lodash iteratee
as its argument; a more light-weight version of `select`.

By the way, process string arguments that don't contain special
characters don't need to be quoted.

Let's use three processes: first selecting the tracks, then filtering
only the explicit ones, and then mapping them to their names:

    $ rq 'select "$.tracks.items[*]"|filter explicit|map name' < data.json
    "Needed Me"
    "Too Good"
    "Work"
    ...

Looks better!

## Higher-order functions

When we're done with all of these explicit tracks, we want to take a
step back and know the artists that have worked on all of the tracks.

    $ rq 'select "$.tracks.items[*]"|map artists' < data.json
    [...]
    [...]
    ...

It turns out that a track can have multiple artists, so that we get a
bunch of arrays!  We can solve this by using `map ...|flatten` or the
more idiomatic `flatMap ...`.

    $ rq 'select "$.tracks.items[*]"|flatMap artists|map name' < data.json
    "Rockabye Baby!"
    "J:Kenzo"
    "Rhianna Kenny"
    ...

## Aggregations

How many times has each artist starred on these tracks?  Let's count
(by name for simplicity):

    $ rq 'select "$.tracks.items[*]"|flatMap artists|countBy name' < data.json
    {"Calvin Harris":2.0,"Drake":7.0,"Eminem":3.0,...}

## Lambdas, sorting and slicing

The results are alphabetical, which is not very useful... Let's first
map them to pairs by using a lambda expression and the lodash
`toPairs` function:

    $ rq 'select "$.tracks.items[*]"|flatMap artists|countBy name|flatMap (o)=>{_.toPairs(o)}' < data.json
    ["Calvin Harris",2.0]
    ["Drake",7.0]
    ["Eminem",3.0]
    ...

Now we can sort the results descending by the second column:

    $ rq 'select "$.tracks.items[*]"|flatMap artists|countBy name|flatMap (o)=>{_.toPairs(o)}|orderBy 1 desc' < data.json
    ["Rihanna",50.0]
    ["Drake",7.0]
    ["Eminem",3.0]
    ...

If we want to save the result in a single JSON array, we can use
`collect`:

    $ rq 'select "$.tracks.items[*]"|flatMap artists|countBy name|flatMap (o)=>{_.toPairs(o)}|orderBy 1 desc|collect' < data.json
    [["Rihanna",50.0],["Drake",7.0],["Eminem",3.0],...]

Looks good!  We are ready to put this data in our web-scale static
responsive blogosphere generator to be used for whatever!

[jsonpath]: http://goessner.net/articles/JsonPath/
[lodash-iteratee]: https://lodash.com/docs#iteratee
