/*
 * This is the rq standard library as implemented in Javascript.
 *
 * Note that the examples in this file are doctests.  Any line with the format:
 *
 *     <input> → <process> <args>* → <output>
 *
 * ...will be verified as part of the build.
 */

// Regex for converting (most) lodash Array JSDoc:
// Search: "_\.(\w+)\(\[([^])]+?)\](?:, ([^)]+))?\);\n \* // => \[(.*)\]$"
// Replace: "$2 => $1($3) => $4"

var _ = require('lodash');

/**
 * Passes through all of the values it sees untouched.
 *
 * @static
 * @example
 * {"a": 2, "b": 3} → id → {"a": 2, "b": 3}
 * true             → id → true
 */
function* id() {
  while (yield* this.pull()) {
    yield* this.push(this.value);
  }
}

/**
 * Selects the field(s) at the specified path for each value in the stream.
 *
 * @static
 * @example
 * {"a": {"b": {"c": 3}}} → select "/a/b" → {"c": 3}
 * {"a": {"b": {"c": 3}}} → select "/a/x" → (empty)
 *
 * @param {string} path the field path to follow
 */
function* select(path) {
  var self = this;
  while (yield* this.pull()) {
    var lenses = rq.util.path(this.value, path);
    if (lenses.length > 0) {
      for (var i = 0; i < lenses.length; i++) {
        var lens = lenses[i];
        var value = lens.get();
        yield* self.push(value);
      }
    } else {
      this.log.warn('path', JSON.stringify(path), 'did not match a value in',
                    JSON.stringify(this.value));
    }
  }
}

/**
 * Modifies the field at the specified path for each value in the stream, using the specified
 * function.
 *
 * @static
 * @example
 * {"a": {"b": 2, "c": true}} → modify "/a/b" (n)=>{n + 2} → {"a": {"b": 4, "c": true}}
 * {"a": {"b": 2, "c": true}} → modify "/a/x" (n)=>{n + 2} → {"a": {"b": 2, "c": true}}
 *
 * @param {string} path the field path to follow
 * @param {function(*): *} f the function to apply
 */
function* modify(path, f) {
  while (yield* this.pull()) {
    var lenses = rq.util.path(this.value, path);
    for (var i = 0; i < lenses.length; i++) {
      var lens = lenses[i];
      lens.set(f(lens.get()));
    }
    yield* this.push(this.value);
  }
}

/**
 * Logs each value that passes through to the info log.
 *
 * @static
 */
function* tee() {
  while (yield* this.pull()) {
    this.log.info(JSON.stringify(this.value));
    yield* this.push(this.value);
  }
}

/**
 * Collects all of the values from the input stream into an array.
 *
 * @static
 * @example
 * true [] 1 → collect → [true, [], 1]
 */
function* collect() {
  yield* this.push((yield* this.collect()));
}

/**
 * Spreads each array in the input stream into separate output values.
 *
 * @static
 * @example
 * [1, 2] [3, 4] 5 → spread → 1 2 3 4 5
 */
function* spread() {
  while (yield* this.pull()) {
    if (Array.isArray(this.value)) {
      yield* this.spread(this.value);
    } else {
      yield* this.push(this.value);
    }
  }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// lodash wrappers - Array                                                                      ///
///                                                                                              ///
/// NOTE: These are not streaming!                                                               ///
////////////////////////////////////////////////////////////////////////////////////////////////////

/**
 * Creates a stream of elements split into groups the length of `size`.
 * If the input stream can't be split evenly, the final chunk will be the remaining
 * elements.
 *
 * @static
 * @param {number} [size=1] The length of each chunk
 * @example
 * "a" "b" "c" "d" → chunk    → ["a"] ["b"] ["c"] ["d"]
 * "a" "b" "c" "d" → chunk  2 → ["a", "b"] ["c", "d"]
 * "a" "b" "c" "d" → chunk  3 → ["a", "b", "c"] ["d"]
 *
 * // Edge cases
 * "a" "b" "c" "d" → chunk -1 → (empty)
 * "a" "b" "c" "d" → chunk  0 → (empty)
 */
function* chunk(size) {
  if (size === undefined) {
    size = 1;
  } else {
    size = Math.max(0, _.toInteger(size));
  }

  if (size > 0) {
    var buffer = [];

    while (yield* this.pull()) {
      buffer.push(this.value);
      if (buffer.length >= size) {
        yield* this.push(buffer);
        buffer = [];
      }
    }

    if (buffer.length > 0) {
      yield* this.push(buffer);
    }
  }
}

/**
 * Creates a stream with all falsey values removed. The values `false`, `null`,
 * `0`, `""`, `undefined`, and `NaN` are falsey.
 *
 * @static
 * @example
 * 0 1 false 2 "" 3 → compact → 1 2 3
 */
function* compact() {
  yield* this.spread(require('lodash').compact((yield* this.collect())));
}

/**
 * Creates a new stream concatenating all input arrays.
 *
 * @static
 * @example
 * [1] 2 [3] [[4]] → concat → [1, 2, 3, [4]]
 */
function* concat() {
  yield* this.push(require('lodash').concat.apply(null, (yield* this.collect())));
}

/**
 * Creates a stream of values not included in the given array
 * using [`SameValueZero`](http://ecma-international.org/ecma-262/6.0/#sec-samevaluezero)
 * for equality comparisons. The order of result values is determined by the order they occur in
 * the input.
 *
 * @static
 * @param {Array} [values] The values to exclude.
 * @see without, xor
 * @example
 * 2 1 → difference [2, 3] → 1
 */
function* difference(values) {
  yield* this.spread(require('lodash').difference((yield* this.collect()), values));
}

/**
 * This method is like `difference` except that it accepts `iteratee` which
 * is invoked for each element of the input and `values` to generate the criterion
 * by which they're compared. Result values are chosen from the input stream.
 * The iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {Array} [values] The values to exclude.
 * @param {Function} [iteratee=_.identity] The iteratee invoked per element.
 * @example
 * 2.1 1.2 → differenceBy [2.3, 3.4] (x)=>{Math.floor(x)} → 1.2
 * // The `property` iteratee shorthand.
 * {"x": 2} {"x": 1} → differenceBy [{"x": 1}] "x" → {"x": 2}
 */
function* differenceBy(values, iteratee) {
  yield* this.spread(require('lodash').differenceBy((yield* this.collect()), values, iteratee));
}

/**
 * This method is like `difference` except that it accepts `comparator`
 * which is invoked to compare elements of the input to `values`. The comparator is invoked with
 * two arguments: (inputVal, othVal).
 *
 * @static
 * @param {Array} [values] The values to exclude.
 * @param {Function} [comparator] The comparator invoked per element.
 * @example
 * {"x": 1, "y": 2} {"x": 2, "y": 1} → differenceWith [{"x": 1, "y": 2}] (a, b)=>{_.isEqual(a, b)} → {"x": 2, "y": 1}
 */
function* differenceWith(values, comparator) {
  yield* this.spread(require('lodash').differenceWith((yield* this.collect()), values, comparator));
}

/**
 * Creates a slice of the input stream with `n` elements dropped from the beginning.
 *
 * @static
 * @param {number} [n=1] The number of elements to drop.
 * @example
 * 1 2 3 → drop   → 2 3
 * 1 2 3 → drop 2 → 3
 * 1 2 3 → drop 5 → (empty)
 * 1 2 3 → drop 0 → 1 2 3
 */
function* drop(n) {
  yield* this.spread(require('lodash').drop((yield* this.collect()), n));
}

/**
 * Creates a slice of the input stream with `n` elements dropped from the end.
 *
 * @static
 * @param {number} [n=1] The number of elements to drop.
 * @example
 * 1 2 3 → dropRight   → 1 2
 * 1 2 3 → dropRight 2 → 1
 * 1 2 3 → dropRight 5 → (empty)
 * 1 2 3 → dropRight 0 → 1 2 3
 */
function* dropRight(n) {
  yield* this.spread(require('lodash').dropRight((yield* this.collect()), n));
}

/**
 * Creates a slice of the input stream excluding elements dropped from the end.
 * Elements are dropped until `predicate` returns falsey. The predicate is
 * invoked with three arguments: (value, index, array).
 *
 * @static
 * @param {Function} [predicate=_.identity] The function invoked per iteration.
 * @example
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → dropRightWhile (o)=>{!o.a} → {"u": "b", "a": true}
 * // The `matches` iteratee shorthand.
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → dropRightWhile {"u": "p", "a": false} → {"u": "b", "a": true} {"u": "f", "a": false}
 * // The `matchesProperty` iteratee shorthand.
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → dropRightWhile ["a", false] → {"u": "b", "a": true}
 * // The `property` iteratee shorthand.
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → dropRightWhile "a" → {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false}
 */
function* dropRightWhile(predicate) {
  yield* this.spread(require('lodash').dropRightWhile((yield* this.collect()), predicate));
}

/**
 * Creates a slice of the input stream excluding elements dropped from the beginning.
 * Elements are dropped until `predicate` returns falsey. The predicate is
 * invoked with three arguments: (value, index, array).
 *
 * @static
 * @param {Function} [predicate=_.identity]
 *  The function invoked per iteration.
 * @example
 * {"u": "b", "a": false} {"u": "f", "a": false} {"u": "p", "a": true} → dropWhile (o)=>{!o.a} → {"u": "p", "a": true}
 * // The `matches` iteratee shorthand.
 * {"u": "b", "a": false} {"u": "f", "a": false} {"u": "p", "a": true} → dropWhile {"u": "b", "a": false} → {"u": "f", "a": false} {"u": "p", "a": true}
 * // The `matchesProperty` iteratee shorthand.
 * {"u": "b", "a": false} {"u": "f", "a": false} {"u": "p", "a": true} → dropWhile ["a", false] → {"u": "p", "a": true}
 * // The `property` iteratee shorthand.
 * {"u": "b", "a": false} {"u": "f", "a": false} {"u": "p", "a": true} → dropWhile "a" → {"u": "b", "a": false} {"u": "f", "a": false} {"u": "p", "a": true}
 */
function* dropWhile(predicate) {
  yield* this.spread(require('lodash').dropWhile((yield* this.collect()), predicate));
}

/**
 * Fills elements of the input stream with `value` from `start` up to, but not
 * including, `end`.
 *
 * @static
 * @param {*} value The value to fill the input stream with.
 * @param {number} [start=0] The start position.
 * @param {number} [end=array.length] The end position.
 * @example
 * 4 6 8 10 → fill "*" 1 3 → 4 "*" "*" 10
 */
function* fill(value, start, end) {
  yield* this.spread(require('lodash').fill((yield* this.collect()), value, start, end));
}

/**
 * This method is like `find` except that it returns the index of the first
 * element `predicate` returns truthy for instead of the element itself.
 *
 * @static
 * @param {Function} [predicate=_.identity]
 *  The function invoked per iteration.
 * @param {number} [fromIndex=0] The index to search from.
 * @example
 * {"u": "b", "a": false} {"u": "f", "a": false} {"u": "p", "a": true} → findIndex (o)=>{o.u == 'b'} → 0
 * // The `matches` iteratee shorthand.
 * {"u": "b", "a": false} {"u": "f", "a": false} {"u": "p", "a": true} → findIndex {"u": "f", "a": false} → 1
 * // The `matchesProperty` iteratee shorthand.
 * {"u": "b", "a": false} {"u": "f", "a": false} {"u": "p", "a": true} → findIndex ["a", false] → 0
 * // The `property` iteratee shorthand.
 * {"u": "b", "a": false} {"u": "f", "a": false} {"u": "p", "a": true} → findIndex "a" → 2
 */
function* findIndex(predicate, fromIndex) {
  yield* this.push(require('lodash').findIndex((yield* this.collect()), predicate, fromIndex));
}

/**
 * This method is like `findIndex` except that it iterates over elements
 * of `collection` from right to left.
 *
 * @static
 * @param {Function} [predicate=_.identity]
 *  The function invoked per iteration.
 * @param {number} [fromIndex=array.length-1] The index to search from.
 * @example
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → findLastIndex (o)=>{o.u == 'p'} → 2
 * // The `matches` iteratee shorthand.
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → findLastIndex {"u": "b", "a": true} → 0
 * // The `matchesProperty` iteratee shorthand.
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → findLastIndex ["a", false] → 2
 * // The `property` iteratee shorthand.
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → findLastIndex "a" → 0
 */
function* findLastIndex(predicate, fromIndex) {
  yield* this.push(require('lodash').findLastIndex((yield* this.collect()), predicate, fromIndex));
}

/**
 * Flattens the input stream a single level deep.
 *
 * @static
 * @example
 * 1  [2, [3, [4]], 5] → flatten → 1 2 [3, [4]] 5
 */
function* flatten() {
  yield* this.spread(require('lodash').flatten((yield* this.collect())));
}

/**
 * Recursively flattens the input stream.
 *
 * @static
 * @example
 * 1 [2, [3, [4]], 5] → flattenDeep → 1 2 3 4 5
 */
function* flattenDeep() {
  yield* this.spread(require('lodash').flattenDeep((yield* this.collect())));
}

/**
 * Recursively flatten the input stream up to `depth` times.
 *
 * @static
 * @param {number} [depth=1] The maximum recursion depth.
 * @example
 * 1 [2, [3, [4]], 5] → flattenDepth 1 → 1 2 [3, [4]] 5
 * 1 [2, [3, [4]], 5] → flattenDepth 2 → 1 2 3 [4] 5
 */
function* flattenDepth(depth) {
  yield* this.spread(require('lodash').flattenDepth((yield* this.collect()), depth));
}

/**
 * The inverse of `toPairs`; this method returns an object composed
 * from key-value `pairs`.
 *
 * @static
 * @example
 * ["a", 1] ["b", 2] → fromPairs → {"a": 1, "b": 2}
 */
function* fromPairs() {
  yield* this.push(require('lodash').fromPairs((yield* this.collect())));
}

/**
 * Gets the first element of the input stream.
 *
 * @static
 * @alias first
 * @example
 * 1 2 3   → head → 1
 * (empty) → head → null
 */
function* head() {
  yield* this.push(require('lodash').head((yield* this.collect())));
}

/**
 * Gets the index at which the first occurrence of `value` is found in the input stream
 * using [`SameValueZero`](http://ecma-international.org/ecma-262/6.0/#sec-samevaluezero)
 * for equality comparisons. If `fromIndex` is negative, it's used as the
 * offset from the end of the input stream.
 *
 * @static
 * @param {*} value The value to search for.
 * @param {number} [fromIndex=0] The index to search from.
 * @example
 * 1 2 1 2 → indexOf 2   → 1
 * // Search from the `fromIndex`.
 * 1 2 1 2 → indexOf 2 2 → 3
 */
function* indexOf(value, fromIndex) {
  yield* this.push(require('lodash').indexOf((yield* this.collect()), value, fromIndex));
}

/**
 * Gets all but the last element of the input stream.
 *
 * @static
 * @example
 * 1 2 3 → initial → 1 2
 */
function* initial() {
  yield* this.spread(require('lodash').initial((yield* this.collect())));
}

/**
 * Creates a stream of unique values that are included in the given array
 * using [`SameValueZero`](http://ecma-international.org/ecma-262/6.0/#sec-samevaluezero)
 * for equality comparisons. The order of result values is determined by the
 * order they occur in the input stream.
 *
 * @static
 * @param {Array} [values] The values to inspect.
 * @example
 * 2 1 → intersection [2, 3] → 2
 */
function* intersection(values) {
  yield* this.spread(require('lodash').intersection((yield* this.collect()), values));
}

/**
 * This method is like `intersection` except that it accepts `iteratee`
 * which is invoked for each element in `values` to generate the criterion
 * by which they're compared. Result values are chosen from the input stream.
 * The iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {Array} [values] The values to inspect.
 * @param {Function} [iteratee=_.identity] The iteratee invoked per element.
 * @example
 * 2.1 1.2 → intersectionBy [2.3, 3.4] (x)=>{Math.floor(x)} → 2.1
 * // The `property` iteratee shorthand.
 * {"x": 1} → intersectionBy [{"x": 2}, {"x": 1}] "x" → {"x": 1}
 */
function* intersectionBy(values, iteratee) {
  yield* this.spread(require('lodash').intersectionBy((yield* this.collect()), values, iteratee));
}

/**
 * This method is like `intersection` except that it accepts `comparator`
 * which is invoked to compare elements of `values`. Result values are chosen
 * from the input stream. The comparator is invoked with two arguments:
 * (arrVal, othVal).
 *
 * @static
 * @param {Array} [values] The values to inspect.
 * @param {Function} [comparator] The comparator invoked per element.
 * @example
 * {"x": 1, "y": 2} {"x": 2, "y": 1} → intersectionWith [{"x": 1, "y": 1}, {"x": 1, "y": 2}] (a, b)=>{_.isEqual(a, b)} → {"x": 1, "y": 2}
 */
function* intersectionWith(values, comparator) {
  yield* this.spread(require('lodash').intersectionWith((yield* this.collect()), values, comparator));
}

/**
 * Converts all elements in the input stream into a string separated by `separator`.
 *
 * @static
 * @param {string} [separator=','] The element separator.
 * @example
 * "a" "b" "c" → join     → "a,b,c"
 * "a" "b" "c" → join "~" → "a~b~c"
 */
function* join(separator) {
  yield* this.push(require('lodash').join((yield* this.collect()), separator));
}

/**
 * Gets the last element of the input stream.
 *
 * @static
 * @example
 * 1 2 3 → last → 3
 */
function* last() {
  yield* this.push(require('lodash').last((yield* this.collect())));
}

/**
 * This method is like `indexOf` except that it iterates over elements of
 * the input stream from right to left.
 *
 * @static
 * @param {*} value The value to search for.
 * @param {number} [fromIndex=array.length-1] The index to search from.
 * @example
 * 1 2 1 2 → lastIndexOf 2   → 3
 * 1 2 1 2 → lastIndexOf 2 2 → 1
 */
function* lastIndexOf(value, fromIndex) {
  yield* this.push(require('lodash').lastIndexOf((yield* this.collect()), value, fromIndex));
}

/**
 * Gets the element at index `n` of the input stream. If `n` is negative, the nth
 * element from the end is returned.
 *
 * @static
 * @param {number} [n=0] The index of the element to return.
 * @example
 * "a" "b" "c" "d" → nth  1 → "b"
 * "a" "b" "c" "d" → nth -2 → "c"
 */
function* nth(n) {
  yield* this.push(require('lodash').nth((yield* this.collect()), n));
}

// pull, pullAll, pullAllBy, pullAllWith, pullAt, remove don't make sense

/**
 * Reverses the input stream so that the first element becomes the last, the second
 * element becomes the second to last, and so on.
 *
 * @static
 * @example
 * 1 2 3 → reverse → 3 2 1
 */
function* reverse() {
  yield* this.spread(require('lodash').reverse((yield* this.collect())));
}

/**
 * Creates a slice of the input stream from `start` up to, but not including, `end`.
 *
 * @static
 * @param {number} [start=0] The start position.
 * @param {number} [end=array.length] The end position.
 * @example
 * 1 2 3 4 → slice 1 3 → 2 3
 */
function* slice(start, end) {
  yield* this.spread(require('lodash').slice((yield* this.collect()), start, end));
}

/**
 * Uses a binary search to determine the lowest index at which `value`
 * should be inserted into the input stream in order to maintain its sort order.
 *
 * @static
 * @param {*} value The value to evaluate.
 *  into the input stream.
 * @example
 * 30 50 → sortedIndex 40 → 1
 */
function* sortedIndex(value) {
  yield* this.push(require('lodash').sortedIndex((yield* this.collect()), value));
}

/**
 * This method is like `sortedIndex` except that it accepts `iteratee`
 * which is invoked for `value` and each element of the input stream to compute their
 * sort ranking. The iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {*} value The value to evaluate.
 * @param {Function} [iteratee=_.identity]
 *  The iteratee invoked per element.
 *  into the input stream.
 * @example
 * {"x": 4} {"x": 5} → sortedIndexBy {"x": 4} (o)=>{o.x} → 0
 * // The `property` iteratee shorthand.
 * {"x": 4} {"x": 5} → sortedIndexBy {"x": 4} "x" → 0
 */
function* sortedIndexBy(value, iteratee) {
  yield* this.push(require('lodash').sortedIndexBy((yield* this.collect()), value, iteratee));
}

/**
 * This method is like `indexOf` except that it performs a binary
 * search on a sorted the input stream.
 *
 * @static
 * @param {*} value The value to search for.
 * @example
 * 4 5 5 5 6 → sortedIndexOf 5 → 1
 */
function* sortedIndexOf(value) {
  yield* this.push(require('lodash').sortedIndexOf((yield* this.collect()), value));
}

/**
 * This method is like `sortedIndex` except that it returns the highest
 * index at which `value` should be inserted into the input stream in order to
 * maintain its sort order.
 *
 * @static
 * @param {*} value The value to evaluate.
 *  into the input stream.
 * @example
 * 4 5 5 5 6 → sortedLastIndex 5 → 4
 */
function* sortedLastIndex(value) {
  yield* this.push(require('lodash').sortedLastIndex((yield* this.collect()), value));
}

/**
 * This method is like `sortedLastIndex` except that it accepts `iteratee`
 * which is invoked for `value` and each element of the input stream to compute their
 * sort ranking. The iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {*} value The value to evaluate.
 * @param {Function} [iteratee=_.identity]
 *  The iteratee invoked per element.
 *  into the input stream.
 * @example
 * {"x": 4} {"x": 5} → sortedLastIndexBy {"x": 4} (o)=>{o.x} → 1
 * // The `property` iteratee shorthand.
 * {"x": 4} {"x": 5} → sortedLastIndexBy {"x": 4} "x" → 1
 */
function* sortedLastIndexBy(value, iteratee) {
  yield* this.push(require('lodash').sortedLastIndexBy((yield* this.collect()), value, iteratee));
}

/**
 * This method is like `lastIndexOf` except that it performs a binary
 * search on a sorted the input stream.
 *
 * @static
 * @param {*} value The value to search for.
 * @example
 * 4 5 5 5 6 → sortedLastIndexOf 5 → 3
 */
function* sortedLastIndexOf(value) {
  yield* this.push(require('lodash').sortedLastIndexOf((yield* this.collect()), value));
}

/**
 * This method is like `uniq` except that it's designed and optimized
 * for sorted arrays.
 *
 * @static
 * @example
 * 1 1 2 → sortedUniq → 1 2
 */
function* sortedUniq() {
  yield* this.spread(require('lodash').sortedUniq((yield* this.collect())));
}

/**
 * This method is like `uniqBy` except that it's designed and optimized
 * for sorted arrays.
 *
 * @static
 * @param {Function} [iteratee] The iteratee invoked per element.
 * @example
 * 1.1 1.2 2.3 2.4 → sortedUniqBy (x)=>{Math.floor(x)} → 1.1 2.3
 */
function* sortedUniqBy(iteratee) {
  yield* this.spread(require('lodash').sortedUniqBy((yield* this.collect()), iteratee));
}

/**
 * Gets all but the first element of the input stream.
 *
 * @static
 * @example
 * 1 2 3 → tail → 2 3
 */
function* tail() {
  yield* this.spread(require('lodash').tail((yield* this.collect())));
}

/**
 * Creates a slice of the input stream with `n` elements taken from the beginning.
 *
 * @static
 * @param {number} [n=1] The number of elements to take.
 * @param- {Object} [guard] Enables use as an iteratee for methods like `map`.
 * @example
 * 1 2 3 → take   → 1
 * 1 2 3 → take 2 → 1 2
 * 1 2 3 → take 5 → 1 2 3
 * 1 2 3 → take 0 → (empty)
 */
function* take(n) {
  yield* this.spread(require('lodash').take((yield* this.collect()), n));
}

/**
 * Creates a slice of the input stream with `n` elements taken from the end.
 *
 * @static
 * @param {number} [n=1] The number of elements to take.
 * @param- {Object} [guard] Enables use as an iteratee for methods like `map`.
 * @example
 * 1 2 3 → takeRight   → 3
 * 1 2 3 → takeRight 2 → 2 3
 * 1 2 3 → takeRight 5 → 1 2 3
 * 1 2 3 → takeRight 0 → (empty)
 */
function* takeRight(n) {
  yield* this.spread(require('lodash').takeRight((yield* this.collect()), n));
}

/**
 * Creates a slice of the input stream with elements taken from the end. Elements are
 * taken until `predicate` returns falsey. The predicate is invoked with
 * three arguments: (value, index, array).
 *
 * @static
 * @param {Function} [predicate=_.identity]
 *  The function invoked per iteration.
 * @example
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → takeRightWhile (o)=>{!o.a} → {"u": "f", "a": false} {"u": "p", "a": false}
 * // The `matches` iteratee shorthand.
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → takeRightWhile {"u": "p", "a": false} → {"u": "p", "a": false}
 * // The `matchesProperty` iteratee shorthand.
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → takeRightWhile ["a", false] → {"u": "f", "a": false} {"u": "p", "a": false}
 * // The `property` iteratee shorthand.
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → takeRightWhile "a" → (empty)
 */
function* takeRightWhile(predicate) {
  yield* this.spread(require('lodash').takeRightWhile((yield* this.collect()), predicate));
}

/**
 * Creates a slice of the input stream with elements taken from the beginning. Elements
 * are taken until `predicate` returns falsey. The predicate is invoked with
 * three arguments: (value, index, array).
 *
 * @static
 * @param {Function} [predicate=_.identity]
 *  The function invoked per iteration.
 * @example
 * {"u": "b", "a": false} {"u": "f", "a": false} {"u": "p", "a": true} → takeWhile (o)=>{!o.a} → {"u": "b", "a": false} {"u": "f", "a": false}
 * // The `matches` iteratee shorthand.
 * {"u": "b", "a": false} {"u": "f", "a": false} {"u": "p", "a": true} → takeWhile {"u": "b", "a": false} → {"u": "b", "a": false}
 * // The `matchesProperty` iteratee shorthand.
 * {"u": "b", "a": false} {"u": "f", "a": false} {"u": "p", "a": true} → takeWhile ["a", false] → {"u": "b", "a": false} {"u": "f", "a": false}
 * // The `property` iteratee shorthand.
 * {"u": "b", "a": false} {"u": "f", "a": false} {"u": "p", "a": true} → takeWhile "a" → (empty)
 */
function* takeWhile(predicate) {
  yield* this.spread(require('lodash').takeWhile((yield* this.collect()), predicate));
}

/**
 * Creates a stream of unique values, in order, from all given values using
 * [`SameValueZero`](http://ecma-international.org/ecma-262/6.0/#sec-samevaluezero)
 * for equality comparisons.
 *
 * @static
 * @param {Array} [values] The values to inspect.
 * @example
 * 2 → union [1, 2] → 2 1
 */
function* union(values) {
  yield* this.spread(require('lodash').union((yield* this.collect()), values));
}

/**
 * This method is like `union` except that it accepts `iteratee` which is
 * invoked for each element of `values` to generate the criterion by
 * which uniqueness is computed. Result values are chosen from the input stream.
 * The iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {Array} [values] The values to inspect.
 * @param {Function} [iteratee=_.identity]
 *  The iteratee invoked per element.
 * @example
 * 2.1 → unionBy [1.2, 2.3] (x)=>{Math.floor(x)} → 2.1 1.2
 * // The `property` iteratee shorthand.
 * {"x": 1} → unionBy [{"x": 2}, {"x": 1}] "x" → {"x": 1} {"x": 2}
 */
function* unionBy(values, iteratee) {
  yield* this.spread(require('lodash').unionBy((yield* this.collect()), values, iteratee));
}

/**
 * This method is like `union` except that it accepts `comparator` which
 * is invoked to compare elements of `values`. Result values are chosen from
 * the input stream. The comparator is invoked with two arguments: (arrVal, othVal).
 *
 * @static
 * @param {Array} [values] The values to inspect.
 * @param {Function} [comparator] The comparator invoked per element.
 * @example
 * {"x": 1, "y": 2} {"x": 2, "y": 1} → unionWith [{"x": 1, "y": 1}, {"x": 1, "y": 2}] (a, b)=>{_.isEqual(a, b)} → {"x": 1, "y": 2} {"x": 2, "y": 1} {"x": 1, "y": 1}
 */
function* unionWith(values, comparator) {
  yield* this.spread(require('lodash').unionWith((yield* this.collect()), values, comparator));
}

/**
 * Creates a duplicate-free version of the input stream, using
 * [`SameValueZero`](http://ecma-international.org/ecma-262/6.0/#sec-samevaluezero)
 * for equality comparisons, in which only the first occurrence of each
 * element is kept.
 *
 * @static
 * @example
 * 2 1 2 → uniq → 2 1
 */
function* uniq() {
  yield* this.spread(require('lodash').uniq((yield* this.collect())));
}

/**
 * This method is like `uniq` except that it accepts `iteratee` which is
 * invoked for each element in the input stream to generate the criterion by which
 * uniqueness is computed. The iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {Function} [iteratee=_.identity]
 *  The iteratee invoked per element.
 * @example
 * 2.1 1.2 2.3 → uniqBy (x)=>{Math.floor(x)} → 2.1 1.2
 * // The `property` iteratee shorthand.
 * {"x": 1} {"x": 2} {"x": 1} → uniqBy "x" → {"x": 1} {"x": 2}
 */
function* uniqBy(iteratee) {
  yield* this.spread(require('lodash').uniqBy((yield* this.collect()), iteratee));
}

/**
 * This method is like `uniq` except that it accepts `comparator` which
 * is invoked to compare elements of the input stream. The comparator is invoked with
 * two arguments: (arrVal, othVal).
 *
 * @static
 * @param {Function} [comparator] The comparator invoked per element.
 * @example
 * {"x": 1, "y": 2} {"x": 2, "y": 1} {"x": 1, "y": 2} → uniqWith (a, b)=>{_.isEqual(a, b)} → {"x": 1, "y": 2} {"x": 2, "y": 1}
 */
function* uniqWith(comparator) {
  yield* this.spread(require('lodash').uniqWith((yield* this.collect()), comparator));
}

/**
 * This method is like `zip` except that it accepts a stream of grouped
 * elements and creates an array regrouping the elements to their pre-zip
 * configuration.
 *
 * @static
 * @example
 * ["a", 1, true] ["b", 2, false] → unzip → ["a", "b"] [1, 2] [true, false]
 */
function* unzip() {
  yield* this.spread(require('lodash').unzip((yield* this.collect())));
}

/**
 * This method is like `unzip` except that it accepts `iteratee` to specify
 * how regrouped values should be combined. The iteratee is invoked with the
 * elements of each group: (...group).
 *
 * @static
 * @param {Function} [iteratee=_.identity] The function to combine
 *  regrouped values.
 * @example
 * [1, 10, 100] [2, 20, 200] → unzipWith (a, b)=>{_.add(a, b)} → 3 30 300
 */
function* unzipWith(iteratee) {
  yield* this.spread(require('lodash').unzipWith((yield* this.collect()), iteratee));
}

/**
 * Creates a stream excluding all given values using
 * [`SameValueZero`](http://ecma-international.org/ecma-262/6.0/#sec-samevaluezero)
 * for equality comparisons.
 *
 * @static
 * @param {...*} [values] The values to exclude.
 * @see _.difference, _.xor
 * @example
 * 2 1 2 3 → without 1 2 → 3
 */
function* without(values) {
  var args = Array.prototype.slice.call(arguments);
  args.unshift((yield* this.collect()));
  yield* this.spread(require('lodash').without.apply(null, args));
}

/**
 * Creates a stream of unique values that is the
 * [symmetric difference](https://en.wikipedia.org/wiki/Symmetric_difference)
 * of the given values. The order of result values is determined by the order
 * they occur in the input stream.
 *
 * @static
 * @param {Array} [values] The values to inspect.
 * @see _.difference, _.without
 * @example
 * 2 1 → xor [2, 3] → 1 3
 */
function* xor(values) {
  yield* this.spread(require('lodash').xor((yield* this.collect()), values));
}

/**
 * This method is like `xor` except that it accepts `iteratee` which is
 * invoked for each element of each `values` to generate the criterion by
 * which by which they're compared. The iteratee is invoked with one argument:
 * (value).
 *
 * @static
 * @param {Array} [values] The arrays to inspect.
 * @param {Function} [iteratee=_.identity]
 *  The iteratee invoked per element.
 * @example
 * 2.1 1.2 → xorBy [2.3, 3.4] (x)=>{Math.floor(x)} → 1.2 3.4
 * // The `property` iteratee shorthand.
 * {"x": 1} → xorBy [{"x": 2}, {"x": 1}] "x" → {"x": 2}
 */
function* xorBy(values, iteratee) {
  yield* this.spread(require('lodash').xorBy((yield* this.collect()), values, iteratee));
}

/**
 * This method is like `xor` except that it accepts `comparator` which is
 * invoked to compare elements of `values`. The comparator is invoked with
 * two arguments: (arrVal, othVal).
 *
 * @static
 * @param {Array} [values] The values to inspect.
 * @param {Function} [comparator] The comparator invoked per element.
 * @example
 * {"x": 1, "y": 2} {"x": 2, "y": 1} → xorWith [{"x": 1, "y": 1}, {"x": 1, "y": 2}] (a, b)=>{_.isEqual(a, b)} → {"x": 2, "y": 1} {"x": 1, "y": 1}
 */
function* xorWith(values, comparator) {
  yield* this.spread(require('lodash').xorWith((yield* this.collect()), values, comparator));
}

/**
 * Creates a stream of grouped elements, the first of which contains the
 * first elements of the given arrays, the second of which contains the
 * second elements of the given arrays, and so on.
 *
 * @static
 * @example
 * ["a", "b"] [1, 2] [true, false] → zip → ["a", 1, true] ["b", 2, false]
 */
function* zip() {
  yield* this.spread(require('lodash').zip.apply(null, (yield* this.collect())));
}

// zipObject and zipObjectDeep don't make sense

/**
 * This method is like `zip` except that it accepts `iteratee` to specify
 * how grouped values should be combined. The iteratee is invoked with the
 * elements of each group: (...group).
 *
 * @static
 * @param {Function} [iteratee=_.identity] The function to combine grouped values.
 * @example
 *
 * [1, 2] [10, 20] [100, 200] → zipWith (a, b, c)=>{a + b + c} → 111 222
 */
function* zipWith(iteratee) {
  var args = [];
  while (yield* this.pull()) {
    args.push(this.value);
  }
  args.push(iteratee);
  yield* this.spread(require('lodash').zipWith.apply(null, args));
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// lodash wrappers - Collection                                                                 ///
///                                                                                              ///
/// NOTE: These are not streaming!                                                               ///
////////////////////////////////////////////////////////////////////////////////////////////////////

/**
 * Checks if `predicate` returns truthy for **all** elements of the input stream.
 * Iteration is stopped once `predicate` returns falsey. The predicate is
 * invoked with two arguments: (value, index).
 *
 * @static
 * @param {Function} [predicate=_.identity]
 *  The function invoked per iteration.
 * @example
 * true 1 null "yes" → every (x)=>{Boolean(x)} → false
 * // With index
 * 1 2 3 → every (x, i) => { i + 1 == x } → true
 * // The `matches` iteratee shorthand.
 * {"u": "b", "g": 36, "a": false} {"u": "f", "g": 40, "a": false} → every {"u": "b", "a": false} → false
 * // The `matchesProperty` iteratee shorthand.
 * {"u": "b", "g": 36, "a": false} {"u": "f", "g": 40, "a": false} → every ["a", false] → true
 * // The `property` iteratee shorthand.
 * {"u": "b", "g": 36, "a": false} {"u": "f", "g": 40, "a": false} → every "a" → false
 */
function* every(predicate) {
  predicate = _.iteratee(predicate);

  var i = 0;
  while (yield* this.pull()) {
    if (!predicate(this.value, i)) {
      yield* this.push(false);
      return;
    }
    i++;
  }

  yield* this.push(true);
}

/**
 * Creates an object composed of keys generated from the results of running
 * each element of the input stream thru `iteratee`. The corresponding value of
 * each key is the number of times the key was returned by `iteratee`. The
 * iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {Function} [iteratee=_.identity]
 *  The iteratee to transform keys.
 * @example
 *
 * 6.1 4.2 6.3         → countBy (x)=>{Math.floor(x)} → {"4": 1, "6": 2}
 * "one" "two" "three" → countBy "length"             → {"3": 2, "5": 1}
 */
function* countBy(iteratee) {
  yield* this.push(require('lodash').countBy((yield* this.collect()), iteratee));
}

/**
 * Iterates over elements of the input stream, returning an array of all elements
 * `predicate` returns truthy for. The predicate is invoked with two
 * arguments: (value, index).
 *
 * **Note:** Unlike `remove`, this method returns a new array.
 *
 * @static
 * @param {Function} [predicate=_.identity]
 *  The function invoked per iteration.
 * @see _.reject
 * @example
 * "a" "b"  "c"   → filter (s)=>{"a" === s}     → "a"
 * "a" "ab" "abc" → filter (s)=>{s.length == 2} → "ab"
 * "a" "ab" "abc" → filter (s, i)=>{i % 2 == 0} → "a" "abc"
 * {"u": "b", "g": 36, "a": true} {"u": "f", "g": 40, "a": false} → filter (o)=>{!o.a} → {"u": "f", "g": 40, "a": false}
 * // The `matches` iteratee shorthand.
 * {"u": "b", "g": 36, "a": true} {"u": "f", "g": 40, "a": false} → filter {"g": 36, "a": true} → {"u": "b", "g": 36, "a": true}
 * // The `matchesProperty` iteratee shorthand.
 * {"u": "b", "g": 36, "a": true} {"u": "f", "g": 40, "a": false} → filter ["a", false] → {"u": "f", "g": 40, "a": false}
 * // The `property` iteratee shorthand.
 * {"u": "b", "g": 36, "a": true} {"u": "f", "g": 40, "a": false} → filter "a" → {"u": "b", "g": 36, "a": true}
 */
function* filter(predicate) {
  predicate = _.iteratee(predicate);

  var i = 0;
  while (yield* this.pull()) {
    if (predicate(this.value, i)) {
      yield* this.push(this.value);
    }
    i++;
  }
}

/**
 * Iterates over elements of the input stream, returning the first element
 * `predicate` returns truthy for. The predicate is invoked with two
 * arguments: (value, index).
 *
 * @static
 * @param {Function} [predicate=_.identity]
 *  The function invoked per iteration.
 * @example
 * {"u": "b", "g": 36, "a": true} {"u": "f", "g": 40, "a": false} {"u": "p", "g": 1, "a": true} → find (o)=>{o.g < 40} → {"u": "b", "g": 36, "a": true}
 * // The `matches` iteratee shorthand.
 * {"u": "b", "g": 36, "a": true} {"u": "f", "g": 40, "a": false} {"u": "p", "g": 1, "a": true} → find {"g": 1, "a": true} → {"u": "p", "g": 1, "a": true}
 * // The `matchesProperty` iteratee shorthand.
 * {"u": "b", "g": 36, "a": true} {"u": "f", "g": 40, "a": false} {"u": "p", "g": 1, "a": true} → find ["a", false] → {"u": "f", "g": 40, "a": false}
 * // The `property` iteratee shorthand.
 * {"u": "b", "g": 36, "a": true} {"u": "f", "g": 40, "a": false} {"u": "p", "g": 1, "a": true} → find "a" → {"u": "b", "g": 36, "a": true}
 */
function* find(predicate) {
  predicate = _.iteratee(predicate);

  var i = 0;
  while (yield* this.pull()) {
    if (predicate(this.value, i)) {
      yield* this.push(this.value);
      return;
    }
    i++;
  }
}

/**
 * This method is like `find` except that it iterates over elements of
 * the input stream from right to left.
 *
 * @static
 * @param {Function} [predicate=_.identity]
 *  The function invoked per iteration.
 * @param {number} [fromIndex=collection.length-1] The index to search from.
 * @example
 * 1 2 3 4 → findLast (n)=>{n % 2 == 1} → 3
 */
function* findLast(predicate, fromIndex) {
  yield* this.push(require('lodash').findLast((yield* this.collect()), predicate, fromIndex));
}
/**
 * Creates a flattened array of values by running each element in the input stream
 * thru `iteratee` and flattening the mapped results. The iteratee is invoked
 * with three arguments: (value, index|key, collection).
 *
 * @static
 * @param {Function} [iteratee=_.identity]
 *  The function invoked per iteration.
 * @example
 * 1 2 → flatMap (n)=>{[n, n]} → 1 1 2 2
 */
function* flatMap(iteratee) {
  yield* this.spread(require('lodash').flatMap((yield* this.collect()), iteratee));
}

/**
 * This method is like `flatMap` except that it recursively flattens the
 * mapped results.
 *
 * @static
 * @param {Function} [iteratee=_.identity]
 *  The function invoked per iteration.
 * @example
 * 1 2 → flatMapDeep (n)=>{[[[n, n]]]} → 1 1 2 2
 */
function* flatMapDeep(iteratee) {
  yield* this.spread(require('lodash').flatMapDeep((yield* this.collect()), iteratee));
}

/**
 * This method is like `flatMap` except that it recursively flattens the
 * mapped results up to `depth` times.
 *
 * @static
 * @param {Function} [iteratee=_.identity]
 *  The function invoked per iteration.
 * @param {number} [depth=1] The maximum recursion depth.
 * @example
 * 1 2 → flatMapDepth (n)=>{[[[n, n]]]} 2 → [1, 1] [2, 2]
 */
function* flatMapDepth(iteratee, depth) {
  yield* this.spread(require('lodash').flatMapDepth((yield* this.collect()), iteratee, depth));
}

// forEach and forEachRight make no sense

/**
 * Creates an object composed of keys generated from the results of running
 * each element of the input stream thru `iteratee`. The order of grouped values
 * is determined by the order they occur in the input stream. The corresponding
 * value of each key is an array of elements responsible for generating the
 * key. The iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {Function} [iteratee=_.identity]
 *  The iteratee to transform keys.
 * @example
 *
 * 6.1 4.2 6.3 → groupBy (x)=>{Math.floor(x)} → {"4": [4.2], "6": [6.1, 6.3]}
 * // The `property` iteratee shorthand.
 * "one" "two" "three" → groupBy "length" → {"3": ["one", "two"], "5": ["three"]}
 */
function* groupBy(iteratee) {
  yield* this.push(require('lodash').groupBy((yield* this.collect()), iteratee));
}

/**
 * Checks if `value` is in the input stream. If the input stream is a string, it's
 * checked for a substring of `value`, otherwise
 * [`SameValueZero`](http://ecma-international.org/ecma-262/6.0/#sec-samevaluezero)
 * is used for equality comparisons. If `fromIndex` is negative, it's used as
 * the offset from the end of the input stream.
 *
 * @static
 * @param {*} value The value to search for.
 * @param {number} [fromIndex=0] The index to search from.
 * @param- {Object} [guard] Enables use as an iteratee for methods like `reduce`.
 * @example
 *
 * 1 2 3 → includes 1   → true
 * 1 2 3 → includes 1 2 → false
 */
function* includes(value, fromIndex) {
  yield* this.push(require('lodash').includes((yield* this.collect()), value, fromIndex));
}

/**
 * Invokes the method at `path` of each element in the input stream, returning
 * an array of the results of each invoked method. Any additional arguments
 * are provided to each invoked method. If `path` is a function, it's invoked
 * for, and `this` bound to, each element in the input stream.
 *
 * @static
 * @param {Array|Function|string} path The path of the method to invoke or
 *  the function invoked per iteration.
 * @param {...*} [args] The arguments to invoke each method with.
 * @example
 * [5, 1, 7] [3, 2, 1] → invokeMap "sort" → [1, 5, 7] [1, 2, 3]
 * "123" "456" → invokeMap "split" "" → ["1", "2", "3"] ["4", "5", "6"]
 */
function* invokeMap(path, args) {
  var fullArgs = Array.prototype.slice.call(arguments);
  fullArgs.unshift((yield* this.collect()));
  yield* this.spread(require('lodash').invokeMap.apply(null, fullArgs));
}

/**
 * Creates an object composed of keys generated from the results of running
 * each element of the input stream thru `iteratee`. The corresponding value of
 * each key is the last element responsible for generating the key. The
 * iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {Function} [iteratee=_.identity]
 *  The iteratee to transform keys.
 * @example
 * {"dir": "left", "code": 97} {"dir": "right", "code": 100} → keyBy (o)=>{String.fromCharCode(o.code)} → {"a": {"dir": "left", "code": 97}, "d": {"dir": "right", "code": 100}}
 * {"dir": "left", "code": 97} {"dir": "right", "code": 100} → keyBy "dir" → {"left": {"dir": "left", "code": 97}, "right": {"dir": "right", "code": 100}}
 */
function* keyBy(iteratee) {
  yield* this.push(require('lodash').keyBy((yield* this.collect()), iteratee));
}

/**
 * Creates a stream of values by running each element in the input stream thru
 * `iteratee`. The iteratee is invoked with two arguments:
 * (value, index).
 *
 * Many lodash methods are guarded to work as iteratees for methods like
 * `every`, `filter`, `map`, `mapValues`, `reject`, and `some`.
 *
 * The guarded methods are:
 * `ary`, `chunk`, `curry`, `curryRight`, `drop`, `dropRight`, `every`,
 * `fill`, `invert`, `parseInt`, `random`, `range`, `rangeRight`, `repeat`,
 * `sampleSize`, `slice`, `some`, `sortBy`, `split`, `take`, `takeRight`,
 * `template`, `trim`, `trimEnd`, `trimStart`, and `words`
 *
 * @static
 * @param {Function} [iteratee=_.identity] The function invoked per iteration.
 * @example
 * 4 8 → map (x)=>{x*x} → 16 64
 * // With index
 * 4 8 → map (x, i)=>{x + i} → 4 9
 * // The `property` iteratee shorthand.
 * {"u": "b"} {"u": "f"} → map "u" → "b" "f"
 */
function* map(iteratee) {
  iteratee = _.iteratee(iteratee);

  var i = 0;
  while (yield* this.pull()) {
    yield* this.push(iteratee(this.value, i));
    i++;
  }
}

/**
 * This method is like `sortBy` except that it allows specifying the sort
 * orders of the iteratees to sort by. If `orders` is unspecified, all values
 * are sorted in ascending order. Otherwise, specify an order of "desc" for
 * descending or "asc" for ascending sort order of corresponding values.
 *
 * @static
 * @param {Array[]|Function[]|Object[]|string[]} [iteratees=[_.identity]]
 *  The iteratees to sort by.
 * @param {string[]} [orders] The sort orders of `iteratees`.
 * @param- {Object} [guard] Enables use as an iteratee for methods like `reduce`.
 * @example
 * {"u": "f", "g": 48} {"u": "b", "g": 34} {"u": "f", "g": 40} {"u": "b", "g": 36} → orderBy ["u", "g"] ["asc", "desc"] → {"u": "b", "g": 36} {"u": "b", "g": 34} {"u": "f", "g": 48} {"u": "f", "g": 40}
 */
function* orderBy(iteratees, orders) {
  yield* this.spread(require('lodash').orderBy((yield* this.collect()), iteratees, orders));
}

/**
 * Creates a stream of elements split into two groups, the first of which
 * contains elements `predicate` returns truthy for, the second of which
 * contains elements `predicate` returns falsey for. The predicate is
 * invoked with one argument: (value).
 *
 * @static
 * @param {Function} [predicate=_.identity] The function invoked per iteration.
 * @example
 * {"u": "b", "g": 36, "a": false} {"u": "f", "g": 40, "a": true} {"u": "p", "g": 1, "a": false} → partition (o)=>{o.a} → [{"u": "f", "g": 40, "a": true}] [{"u": "b", "g": 36, "a": false}, {"u": "p", "g": 1, "a": false}]
 * // The `matches` iteratee shorthand.
 * {"u": "b", "g": 36, "a": false} {"u": "f", "g": 40, "a": true} {"u": "p", "g": 1, "a": false} → partition {"g": 1, "a": false} → [{"u": "p", "g": 1, "a": false}] [{"u": "b", "g": 36, "a": false}, {"u": "f", "g": 40, "a": true}]
 * // The `matchesProperty` iteratee shorthand.
 * {"u": "b", "g": 36, "a": false} {"u": "f", "g": 40, "a": true} {"u": "p", "g": 1, "a": false} → partition ["a", false] → [{"u": "b", "g": 36, "a": false}, {"u": "p", "g": 1, "a": false}] [{"u": "f", "g": 40, "a": true}]
 * // The `property` iteratee shorthand.
 * {"u": "b", "g": 36, "a": false} {"u": "f", "g": 40, "a": true} {"u": "p", "g": 1, "a": false} → partition "a" → [{"u": "f", "g": 40, "a": true}] [{"u": "b", "g": 36, "a": false}, {"u": "p", "g": 1, "a": false}]
 */
function* partition(predicate) {
  yield* this.spread(require('lodash').partition((yield* this.collect()), predicate));
}

/**
 * Reduces the input stream to a value which is the accumulated result of running
 * each element in the input stream thru `iteratee`, where each successive
 * invocation is supplied the return value of the previous. If `accumulator`
 * is not given, the first element of the input stream is used as the initial
 * value. The iteratee is invoked with four arguments:
 * (accumulator, value, index|key, collection).
 *
 * Many lodash methods are guarded to work as iteratees for methods like
 * `reduce`, `reduceRight`, and `transform`.
 *
 * The guarded methods are:
 * `assign`, `defaults`, `defaultsDeep`, `includes`, `merge`, `orderBy`,
 * and `sortBy`
 *
 * @static
 * @param {Function} [iteratee=_.identity] The function invoked per iteration.
 * @param {*} [accumulator] The initial value.
 * @see _.reduceRight
 * @example
 * 1 2 → reduce (sum, n)=>{sum + n} 0 → 3
 */
function* reduce(iteratee, accumulator) {
  yield* this.push(require('lodash').reduce((yield* this.collect()), iteratee, accumulator));
}

/**
 * This method is like `reduce` except that it iterates over elements of
 * the input stream from right to left.
 *
 * @static
 * @param {Function} [iteratee=_.identity] The function invoked per iteration.
 * @param {*} [accumulator] The initial value.
 * @see _.reduce
 * @example
 * [0, 1] [2, 3] [4, 5] → reduceRight (flattened, other)=>{flattened.concat(other)} [] → [4, 5, 2, 3, 0, 1]
 */
function* reduceRight(iteratee, accumulator) {
  yield* this.push(require('lodash').reduceRight((yield* this.collect()), iteratee, accumulator));
}

/**
 * The opposite of `filter`; this method returns the elements of the input stream
 * that `predicate` does **not** return truthy for.
 *
 * @static
 * @param {Function} [predicate=_.identity] The function invoked per iteration.
 * @see _.filter
 * @example
 * {"u": "b", "g": 36, "a": false} {"u": "f", "g": 40, "a": true} → reject (o)=>{!o.a} → {"u": "f", "g": 40, "a": true}
 * // The `matches` iteratee shorthand.
 * {"u": "b", "g": 36, "a": false} {"u": "f", "g": 40, "a": true} → reject {"g": 40, "a": true} → {"u": "b", "g": 36, "a": false}
 * // The `matchesProperty` iteratee shorthand.
 * {"u": "b", "g": 36, "a": false} {"u": "f", "g": 40, "a": true} → reject ["a", false] → {"u": "f", "g": 40, "a": true}
 * // The `property` iteratee shorthand.
 * {"u": "b", "g": 36, "a": false} {"u": "f", "g": 40, "a": true} → reject "a" → {"u": "b", "g": 36, "a": false}
 */
function* reject(predicate) {
  predicate = _.iteratee(predicate);

  var i = 0;
  while (yield* this.pull()) {
    if (!predicate(this.value, i)) {
      yield* this.push(this.value);
    }
    i++;
  }
}

/**
 * Gets a random element from the input stream.
 *
 * @static
 * @example
 * 1 2 3 4 → sample → 2 (not tested)
 */
function* sample() {
  yield* this.push(require('lodash').sample((yield* this.collect())));
}

/**
 * Gets `n` random elements at unique keys from the input stream up to the
 * size of the input stream.
 *
 * @static
 * @param {number} [n=1] The number of elements to sample.
 * @param- {Object} [guard] Enables use as an iteratee for methods like `map`.
 * @example
 * 1 2 3 → sampleSize 2 → 3 1 (not tested)
 * 1 2 3 → sampleSize 4 → 2 3 1 (not tested)
 */
function* sampleSize(n) {
  yield* this.push(require('lodash').sampleSize((yield* this.collect()), n));
}

/**
 * Creates a stream of shuffled values, using a version of the
 * [Fisher-Yates shuffle](https://en.wikipedia.org/wiki/Fisher-Yates_shuffle).
 *
 * @static
 * @example
 * 1 2 3 4 → shuffle → 4 1 3 2 (not tested)
 */
function* shuffle() {
  yield* this.spread(require('lodash').shuffle((yield* this.collect())));
}

/**
 * Gets the size of the input stream by returning its length.
 *
 * @static
 * @example
 * 1 2 3 → size → 3
 */
function* size() {
  yield* this.push(require('lodash').size((yield* this.collect())));
}

/**
 * Checks if `predicate` returns truthy for **any** element of the input stream.
 * Iteration is stopped once `predicate` returns truthy. The predicate is
 * invoked with two arguments: (value, index).
 *
 * @static
 * @param {Function} [predicate=_.identity] The function invoked per iteration.
 * @example
 * null 0 "yes" false → some (x)=>{Boolean(x)} → true
 * // With index
 * 5 1 8 → some (x, i) => { i == x } → true
 * // The `matches` iteratee shorthand.
 * {"u": "b", "a": true} {"u": "f", "a": false} → some {"u": "b", "a": false} → false
 * // The `matchesProperty` iteratee shorthand.
 * {"u": "b", "a": true} {"u": "f", "a": false} → some ["a", false] → true
 * // The `property` iteratee shorthand.
 * {"u": "b", "a": true} {"u": "f", "a": false} → some "a" → true
 */
function* some(predicate) {
  predicate = _.iteratee(predicate);

  var i = 0;
  while (yield* this.pull()) {
    if (predicate(this.value, i)) {
      yield* this.push(true);
      return;
    }
    i++;
  }

  yield* this.push(false);
}

/**
 * Creates a stream of elements, sorted in ascending order by the results of
 * running each element in a collection thru each iteratee. This method
 * performs a stable sort, that is, it preserves the original sort order of
 * equal elements. The iteratees are invoked with one argument: (value).
 *
 * @static
 * @param {...(Function|Function[])} [iteratees=[_.identity]]
 *  The iteratees to sort by.
 * @example
 * {"u": "f", "g": 48} {"u": "b", "g": 36} {"u": "f", "g": 40} {"u": "b", "g": 34} → sortBy (o)=>{o.u} → {"u": "b", "g": 36} {"u": "b", "g": 34} {"u": "f", "g": 48} {"u": "f", "g": 40}
 * {"u": "f", "g": 48} {"u": "b", "g": 36} {"u": "f", "g": 40} {"u": "b", "g": 34} → sortBy ["u", "g"] → {"u": "b", "g": 34} {"u": "b", "g": 36} {"u": "f", "g": 40} {"u": "f", "g": 48}
 * {"u": "f", "g": 48} {"u": "b", "g": 36} {"u": "f", "g": 40} {"u": "b", "g": 34} → sortBy "u" (o)=>{o.a/10} → {"u": "b", "g": 36} {"u": "b", "g": 34} {"u": "f", "g": 48} {"u": "f", "g": 40}
 */
function* sortBy(iteratees) {
  yield* this.spread(require('lodash').orderBy((yield* this.collect()), iteratees));
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// lodash wrappers - Date                                                                       ///
///                                                                                              ///
/// NOTE: These are not streaming!                                                               ///
////////////////////////////////////////////////////////////////////////////////////////////////////

/**
 * Gets the timestamp of the number of milliseconds that have elapsed since
 * the Unix epoch (1 January 1970 00:00:00 UTC).
 *
 * @static
 * @example
 * (empty) → now → 1470104632000 (not tested)
 */
function* now() {
  yield* this.push(require('lodash').now());
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// lodash wrappers - Function/Lang don't make sense                                             ///
////////////////////////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////////////////////////
/// lodash wrappers - Math                                                                       ///
///                                                                                              ///
/// NOTE: These are not streaming!                                                               ///
////////////////////////////////////////////////////////////////////////////////////////////////////

// add, ceil, divide, floor don't make sense

/**
 * Computes the maximum value of the input stream. If the input stream is empty or falsey,
 * `undefined` is returned.
 *
 * @static
 * @example
 * 4 2 8 6 → max → 8
 * (empty) → max → null
 */
function* max() {
  yield* this.push(require('lodash').max((yield* this.collect())));
}

/**
 * This method is like `max` except that it accepts `iteratee` which is
 * invoked for each element in the input stream to generate the criterion by which
 * the value is ranked. The iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {Function} [iteratee=_.identity] The iteratee invoked per element.
 * @example
 * {"n": 1} {"n": 2} → maxBy (o)=>{o.n} → {"n": 2}
 * // The `property` iteratee shorthand.
 * {"n": 1} {"n": 2} → maxBy "n" → {"n": 2}
 */
function* maxBy(iteratee) {
  yield* this.push(require('lodash').maxBy((yield* this.collect()), iteratee));
}

/**
 * Computes the mean of the values in the input stream.
 *
 * @static
 * @example
 * 4 2 8 6 → mean → 5
 * (empty) → mean → null
 */
function* mean() {
  yield* this.push(require('lodash').mean((yield* this.collect())));
}

/**
 * This method is like `mean` except that it accepts `iteratee` which is
 * invoked for each element in the input stream to generate the value to be averaged.
 * The iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {Function} [iteratee=_.identity] The iteratee invoked per element.
 * @example
 * {"n": 4} {"n": 2} {"n": 8} {"n": 6} → meanBy (o)=>{o.n} → 5
 * // The `property` iteratee shorthand.
 * {"n": 4} {"n": 2} {"n": 8} {"n": 6} → meanBy "n" → 5
 */
function* meanBy(iteratee) {
  yield* this.push(require('lodash').meanBy((yield* this.collect()), iteratee));
}

/**
 * Computes the minimum value of the input stream. If the input stream is empty or falsey,
 * `undefined` is returned.
 *
 * @static
 * @example
 * 4 2 8 6 → min → 2
 * (empty) → min → null
 */
function* min() {
  yield* this.push(require('lodash').min((yield* this.collect())));
}

/**
 * This method is like `min` except that it accepts `iteratee` which is
 * invoked for each element in the input stream to generate the criterion by which
 * the value is ranked. The iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {Function} [iteratee=_.identity] The iteratee invoked per element.
 * @example
 * {"n": 1} {"n": 2} → minBy (o)=>{o.n} → {"n": 1}
 * // The `property` iteratee shorthand.
 * {"n": 1} {"n": 2} → minBy "n" → {"n": 1}
 */
function* minBy(iteratee) {
  yield* this.push(require('lodash').minBy((yield* this.collect()), iteratee));
}

// multiply, round, subtract don't make sense

/**
 * Computes the sum of the values in the input stream.
 *
 * @static
 * @example
 * 4 2 8 6 → sum → 20
 * (empty) → sum → 0
 */
function* sum() {
  yield* this.push(require('lodash').sum((yield* this.collect())));
}

/**
 * This method is like `sum` except that it accepts `iteratee` which is
 * invoked for each element in the input stream to generate the value to be summed.
 * The iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {Function} [iteratee=_.identity] The iteratee invoked per element.
 * @example
 * {"n": 4} {"n": 2} {"n": 8} {"n": 6} → sumBy (o)=>{o.n} → 20
 * // The `property` iteratee shorthand.
 * {"n": 4} {"n": 2} {"n": 8} {"n": 6} → sumBy "n" → 20
 */
function* sumBy(iteratee) {
  yield* this.push(require('lodash').sumBy((yield* this.collect()), iteratee));
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// lodash wrappers - Number                                                                     ///
///                                                                                              ///
/// NOTE: These are not streaming!                                                               ///
////////////////////////////////////////////////////////////////////////////////////////////////////

// clamp, inRange don't make sense

/**
 * Produces a random number between the inclusive `lower` and `upper` bounds.
 * If only one argument is provided a number between `0` and the given number
 * is returned. If `floating` is `true`, or either `lower` or `upper` are
 * floats, a floating-point number is returned instead of an integer.
 *
 * **Note:** JavaScript follows the IEEE-754 standard for resolving
 * floating-point values which can produce unexpected results.
 *
 * @static
 * @param {number} [lower=0] The lower bound.
 * @param {number} [upper=1] The upper bound.
 * @param {boolean} [floating] Specify returning a floating-point number.
 * @example
 * (empty) → random 0 5 → 2 (not tested)
 * (empty) → random 5 → 3 (not tested)
 * (empty) → random 5 true → 3.2 (not tested)
 * (empty) → random 1.2 1.5 → 1.3 (not tested)
 */
function* random(lower, upper, floating) {
  yield* this.push(require('lodash').random(lower, upper, floating));
}
