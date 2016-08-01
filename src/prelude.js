/**
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

/**
 * Passes through all of the values it sees untouched.
 *
 * @static
 * @example
 * {"a": 2, "b": 3} → id → {"a": 2, "b": 3}
 * true             → id → true
 */
function id() {
  while (this.pull()) {
    this.push(this.value);
  }
}

/**
 * Selects the field(s) at the specified path for each value in the stream.
 *
 * @static
 * @example
 * {"a": {"b": {"c": 3} } } → select "/a/b" → {"c": 3}
 * {"a": {"b": {"c": 3} } } → select "/a/x" → (nothing)
 *
 * @param {string} path the field path to follow
 */
function select(path) {
  var self = this;
  while (this.pull()) {
    var lenses = rq.util.path(this.value, path);
    if (lenses.length > 0) {
      for (var i = 0; i < lenses.length; i++) {
        var lens = lenses[i];
        var value = lens.get();
        self.log.debug('selecting', JSON.stringify(value), 'for path', JSON.stringify(path));
        self.push(value);
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
 * {"a": {"b": 2, "c": true} } → modify "/a/b" (n)=>{n + 2} → {"a": {"b": 4, "c": true} }
 * {"a": {"b": 2, "c": true} } → modify "/a/x" (n)=>{n + 2} → {"a": {"b": 2, "c": true} }
 *
 * @param {string} path the field path to follow
 * @param {function(*): *} f the function to apply
 */
function modify(path, f) {
  while (this.pull()) {
    var lenses = rq.util.path(this.value, path);
    for (var i = 0; i < lenses.length; i++) {
      var lens = lenses[i];
      lens.set(f(lens.get()));
    }
    this.push(this.value);
  }
}

/**
 * Logs each value that passes through to the info log.
 *
 * @static
 */
function tee() {
  while (this.pull()) {
    this.log.info(JSON.stringify(this.value));
    this.push(this.value);
  }
}

/**
 * Collects all of the values from the input stream into an array.
 *
 * @static
 * @example
 * true [] 1 → collect → [true, [], 1]
 */
function collect() {
  this.push(this.collect());
}

/**
 * Spreads each array in the input stream into separate output values.
 *
 * @static
 * @example
 * [1, 2] [3, 4] 5 → spread → 1 2 3 4 5
 */
function spread() {
  while (this.pull()) {
    if (Array.isArray(this.value)) {
      this.spread(this.value);
    } else {
      this.push(this.value);
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
 * "a" "b" "c" "d" → chunk 2 → ["a", "b"] ["c", "d"]
 * "a" "b" "c" "d" → chunk 3 → ["a", "b", "c"] ["d"]
 */
function chunk(size) {
  this.spread(require('lodash.js').chunk(this.collect(), size));
}

/**
 * Creates a stream with all falsey values removed. The values `false`, `null`,
 * `0`, `""`, `undefined`, and `NaN` are falsey.
 *
 * @static
 * @example
 * 0 1 false 2 "" 3 → compact → 1 2 3
 */
function compact() {
  this.spread(require('lodash.js').compact(this.collect()));
}

/**
 * Creates a new stream concatenating all input arrays.
 *
 * @static
 * @example
 * [1] 2 [3] [[4]] → concat → [1, 2, 3, [4]]
 */
function concat() {
  this.push(require('lodash.js').concat.apply(null, this.collect()));
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
function difference(values) {
  this.spread(require('lodash.js').difference(this.collect(), values));
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
 *
 * 2.1 1.2 → differenceBy [2.3, 3.4] (x)=>{Math.floor(x)} → 1.2
 *
 * // The `property` iteratee shorthand.
 * {"x": 2} {"x": 1} → differenceBy [{"x": 1}] "x" → {"x": 2}
 */
function differenceBy(values, iteratee) {
  this.spread(require('lodash.js').differenceBy(this.collect(), values, iteratee));
}

/**
 * This method is like `difference` except that it accepts `comparator`
 * which is invoked to compare elements of the input to `values`. The comparator is invoked with
 * two
 * arguments: (inputVal, othVal).
 *
 * @static
 * @param {Array} [values] The values to exclude.
 * @param {Function} [comparator] The comparator invoked per element.
 * @example
 * {"x": 1, "y": 2} {"x": 2, "y": 1} → differenceWith [{"x": 1, "y": 2}] (a, b)=>{_.isEqual(a, b)} → {"x": 2, "y": 1}
 */
function differenceWith(values, comparator) {
  this.spread(require('lodash.js').differenceWith(this.collect(), values, comparator));
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
function drop(n) {
  this.spread(require('lodash.js').drop(this.collect(), n));
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
function dropRight(n) {
  this.spread(require('lodash.js').dropRight(this.collect(), n));
}

/**
 * Creates a slice of the input stream excluding elements dropped from the end.
 * Elements are dropped until `predicate` returns falsey. The predicate is
 * invoked with three arguments: (value, index, array).
 *
 * @static
 * @param {Function} [predicate=_.identity] The function invoked per iteration.
 * @example
 *
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → dropRightWhile (o)=>{!o.a} → {"u": "b", "a": true}
 * // The `matches` iteratee shorthand.
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → dropRightWhile {"u": "p", "a": false} → {"u": "b", "a": true} {"u": "f", "a": false}
 * // The `matchesProperty` iteratee shorthand.
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → dropRightWhile ["a", false] → {"u": "b", "a": true}
 * // The `property` iteratee shorthand.
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → dropRightWhile "a" → {"u": "b", "a": true} → {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false}
 */
function dropRightWhile(predicate) {
  this.spread(require('lodash.js').dropRightWhile(this.collect(), predicate));
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
 *
 * var users = [
 *   { 'user': 'barney',  'active': false },
 *   { 'user': 'fred',    'active': false },
 *   { 'user': 'pebbles', 'active': true }
 * ];
 *
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → dropWhile (o)=>{!o.a} → {"u": "p", "a": false}
 *
 * // The `matches` iteratee shorthand.
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → dropWhile {"u": "b', "a": false} → {"u": "f", "a": false} {"u": "p", "a": false}
 *
 * // The `matchesProperty` iteratee shorthand.
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → dropWhile ["active", false] → {"u": "p", "a": false}
 *
 * // The `property` iteratee shorthand.
 * {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false} → dropWhile "active" → {"u": "b", "a": true} {"u": "f", "a": false} {"u": "p", "a": false}
 */
function dropWhile(predicate) {
  this.spread(require('lodash.js').dropWhile(this.collect(), predicate));
}

/**
 * Fills elements of the input stream with `value` from `start` up to, but not
 * including, `end`.
 *
 * **Note:** This method mutates the input stream.
 *
 * @static
 * @param {*} value The value to fill the input stream with.
 * @param {number} [start=0] The start position.
 * @param {number} [end=array.length] The end position.
 * @example
 * 4 6 8 10 → fill("*", 1, 3) → 4 "*" "*" 10
 */
function fill(value, start, end) {
  this.spread(require('lodash.js').fill(this.collect(), value, start, end));
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
 *
 * var users = [
 *   { 'user': 'barney',  'active': false },
 *   { 'user': 'fred',    'active': false },
 *   { 'user': 'pebbles', 'active': true }
 * ];
 *
 * _.findIndex(users, function(o) { return o.user == 'barney'; });
 * // => 0
 *
 * // The `matches` iteratee shorthand.
 * _.findIndex(users, { 'user': 'fred', 'active': false });
 * // => 1
 *
 * // The `matchesProperty` iteratee shorthand.
 * _.findIndex(users, ['active', false]);
 * // => 0
 *
 * // The `property` iteratee shorthand.
 * _.findIndex(users, 'active');
 * // => 2
 */
function findIndex(predicate, fromIndex) {
  this.push(require('lodash.js').findIndex(this.collect(), predicate, fromIndex));
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
 *
 * var users = [
 *   { 'user': 'barney',  'active': true },
 *   { 'user': 'fred',    'active': false },
 *   { 'user': 'pebbles', 'active': false }
 * ];
 *
 * _.findLastIndex(users, function(o) { return o.user == 'pebbles'; });
 * // => 2
 *
 * // The `matches` iteratee shorthand.
 * _.findLastIndex(users, { 'user': 'barney', 'active': true });
 * // => 0
 *
 * // The `matchesProperty` iteratee shorthand.
 * _.findLastIndex(users, ['active', false]);
 * // => 2
 *
 * // The `property` iteratee shorthand.
 * _.findLastIndex(users, 'active');
 * // => 0
 */
function findLastIndex(predicate, fromIndex) {
  this.push(require('lodash.js').findLastIndex(this.collect(), predicate, fromIndex));
}

/**
 * Flattens the input stream a single level deep.
 *
 * @static
 * @example
 *
 * 1  [2, [3, [4]] 5 → flatten → 1 2 [3, [4]] 5
 */
function flatten() {
  this.spread(require('lodash.js').flatten(this.collect()));
}

/**
 * Recursively flattens the input stream.
 *
 * @static
 * @example
 *
 * 1 [2, [3, [4]] 5] → flattenDeep → 1 2 3 4 5
 */
function flattenDeep() {
  this.spread(require('lodash.js').flattenDeep(this.collect()));
}

/**
 * Recursively flatten the input stream up to `depth` times.
 *
 * @static
 * @param {number} [depth=1] The maximum recursion depth.
 * @example
 *
 * var array = [1, [2, [3, [4]], 5]];
 *
 * _.flattenDepth(array, 1);
 * // => [1, 2, [3, [4]], 5]
 *
 * _.flattenDepth(array, 2);
 * // => [1, 2, 3, [4], 5]
 */
function flattenDepth(depth) {
  this.spread(require('lodash.js').flattenDepth(this.collect(), depth));
}

/**
 * The inverse of `toPairs`; this method returns an object composed
 * from key-value `pairs`.
 *
 * @static
 * @example
 *
 * ["a", 1] ["b", 2] → fromPairs → {"a": 1, "b": 2}
 */
function fromPairs() {
  this.push(require('lodash.js').fromPairs(this.collect()));
}

/**
 * Gets the first element of the input stream.
 *
 * @static
 * @alias first
 * @example
 *
 * 1 2 3   → head → 1
 * (empty) → head → null
 */
function head() {
  this.push(require('lodash.js').head(this.collect()));
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
 *
 * 1 2 1 2 → indexOf 2   → 1
 * // Search from the `fromIndex`.
 * 1 2 1 2 → indexOf 2 2 → 3
 */
function indexOf(value, fromIndex) {
  this.push(require('lodash.js').indexOf(this.collect(), value, fromIndex));
}

/**
 * Gets all but the last element of the input stream.
 *
 * @static
 * @example
 *
 * 1, 2, 3 → initial → 1, 2
 */
function initial() {
  this.push(require('lodash.js').initial(this.collect()));
}

/**
 * Creates a stream of unique values that are included in all given arrays
 * using [`SameValueZero`](http://ecma-international.org/ecma-262/6.0/#sec-samevaluezero)
 * for equality comparisons. The order of result values is determined by the
 * order they occur in the first array.
 *
 * @static
 * @param {...Array} [arrays] The arrays to inspect.
 * @example
 *
 * 2, 1 → intersection([2, 3]) → 2
 */
function intersection(values) {
  this.spread(require('lodash.js').intersection(this.collect(), values));
}

/**
 * This method is like `intersection` except that it accepts `iteratee`
 * which is invoked for each element of each `arrays` to generate the criterion
 * by which they're compared. Result values are chosen from the first array.
 * The iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {...Array} [arrays] The arrays to inspect.
 * @param {Function} [iteratee=_.identity] The iteratee invoked per element.
 * @example
 *
 * 2.1, 1.2 → intersectionBy([2.3, 3.4], Math.floor) → 2.1
 *
 * // The `property` iteratee shorthand.
 * { 'x': 1 } → intersectionBy([{ 'x': 2 }, { 'x': 1 }], 'x') → { 'x': 1 }
 */
function intersectionBy(values, iteratee) {
  this.spread(require('lodash.js').intersectionBy(this.collect(), values, iteratee));
}

/**
 * This method is like `intersection` except that it accepts `comparator`
 * which is invoked to compare elements of `arrays`. Result values are chosen
 * from the first array. The comparator is invoked with two arguments:
 * (arrVal, othVal).
 *
 * @static
 * @param {...Array} [arrays] The arrays to inspect.
 * @param {Function} [comparator] The comparator invoked per element.
 * @example
 *
 * var objects = [{ 'x': 1, 'y': 2 }, { 'x': 2, 'y': 1 }];
 * var others = [{ 'x': 1, 'y': 1 }, { 'x': 1, 'y': 2 }];
 *
 * _.intersectionWith(objects, others, _.isEqual);
 * // => [{ 'x': 1, 'y': 2 }]
 */
function intersectionWith(values, comparator) {
  this.spread(require('lodash.js').intersectionWith(this.collect(), values, comparator));
}

/**
 * Converts all elements in the input stream into a string separated by `separator`.
 *
 * @static
 * @param {string} [separator=','] The element separator.
 * @example
 *
 * _.join(['a', 'b', 'c'], '~');
 * // => 'a~b~c'
 */
function join(separator) {
  this.push(require('lodash.js').join(this.collect(), separator));
}

/**
 * Gets the last element of the input stream.
 *
 * @static
 * @example
 *
 * _.last([1, 2, 3]);
 * // => 3
 */
function last() {
  this.push(require('lodash.js').last(this.collect()));
}

/**
 * This method is like `indexOf` except that it iterates over elements of
 * the input stream from right to left.
 *
 * @static
 * @param {*} value The value to search for.
 * @param {number} [fromIndex=array.length-1] The index to search from.
 * @example
 *
 * _.lastIndexOf([1, 2, 1, 2], 2);
 * // => 3
 *
 * // Search from the `fromIndex`.
 * _.lastIndexOf([1, 2, 1, 2], 2, 2);
 * // => 1
 */
function lastIndexOf(value, fromIndex) {
  this.push(require('lodash.js').lastIndexOf(this.collect(), value, fromIndex));
}

/**
 * Gets the element at index `n` of the input stream. If `n` is negative, the nth
 * element from the end is returned.
 *
 * @static
 * @param {number} [n=0] The index of the element to return.
 * @example
 *
 * var array = ['a', 'b', 'c', 'd'];
 *
 * _.nth(array, 1);
 * // => 'b'
 *
 * _.nth(array, -2);
 * // => 'c';
 */
function nth(n) {
  this.push(require('lodash.js').nth(this.collect(), n));
}

/**
 * Removes all given values from the input stream using
 * [`SameValueZero`](http://ecma-international.org/ecma-262/6.0/#sec-samevaluezero)
 * for equality comparisons.
 *
 * **Note:** Unlike `without`, this method mutates the input stream. Use `remove`
 * to remove elements from an array by predicate.
 *
 * @static
 * @param {...*} [values] The values to remove.
 * @example
 *
 * var array = ['a', 'b', 'c', 'a', 'b', 'c'];
 *
 * _.pull(array, 'a', 'c');
 * console.log(array);
 * // => ['b', 'b']
 */
function pull() {
  var args = Array.prototype.slice.call(arguments);
  args.unshift(this.collect());
  this.spread(require('lodash.js').pull.apply(null, args));
}

/**
 * This method is like `pull` except that it accepts an array of values to remove.
 *
 * **Note:** Unlike `difference`, this method mutates the input stream.
 *
 * @static
 * @param {Array} values The values to remove.
 * @example
 *
 * var array = ['a', 'b', 'c', 'a', 'b', 'c'];
 *
 * _.pullAll(array, ['a', 'c']);
 * console.log(array);
 * // => ['b', 'b']
 */
function pullAll(values) {
  this.spread(require('lodash.js').pullAll(this.collect(), values));
}

/**
 * This method is like `pullAll` except that it accepts `iteratee` which is
 * invoked for each element of the input stream and `values` to generate the criterion
 * by which they're compared. The iteratee is invoked with one argument: (value).
 *
 * **Note:** Unlike `differenceBy`, this method mutates the input stream.
 *
 * @static
 * @param {Array} values The values to remove.
 * @param {Function} [iteratee=_.identity]
 *  The iteratee invoked per element.
 * @example
 *
 * var array = [{ 'x': 1 }, { 'x': 2 }, { 'x': 3 }, { 'x': 1 }];
 *
 * _.pullAllBy(array, [{ 'x': 1 }, { 'x': 3 }], 'x');
 * console.log(array);
 * // => [{ 'x': 2 }]
 */
function pullAllBy(values, iteratee) {
  this.spread(require('lodash.js').pullAllBy(this.collect(), values, iteratee));
}

/**
 * This method is like `pullAll` except that it accepts `comparator` which
 * is invoked to compare elements of the input stream to `values`. The comparator is
 * invoked with two arguments: (arrVal, othVal).
 *
 * **Note:** Unlike `differenceWith`, this method mutates the input stream.
 *
 * @static
 * @param {Array} values The values to remove.
 * @param {Function} [comparator] The comparator invoked per element.
 * @example
 *
 * var array = [{ 'x': 1, 'y': 2 }, { 'x': 3, 'y': 4 }, { 'x': 5, 'y': 6 }];
 *
 * _.pullAllWith(array, [{ 'x': 3, 'y': 4 }], _.isEqual);
 * console.log(array);
 * // => [{ 'x': 1, 'y': 2 }, { 'x': 5, 'y': 6 }]
 */
function pullAllWith(values, comparator) {
  this.spread(require('lodash.js').pullAllWith(this.collect(), values, comparator));
}

/**
 * Removes elements from the input stream corresponding to `indexes` and returns an
 * array of removed elements.
 *
 * **Note:** Unlike `at`, this method mutates the input stream.
 *
 * @static
 * @param {...(number|number[])} [indexes] The indexes of elements to remove.
 * @example
 *
 * var array = ['a', 'b', 'c', 'd'];
 * var pulled = _.pullAt(array, [1, 3]);
 *
 * console.log(array);
 * // => ['a', 'c']
 *
 * console.log(pulled);
 * // => ['b', 'd']
 */
function pullAt(indexes) {
  var result = this.collect();
  require('lodash.js').pullAt(result, indexes);
  this.spread(result);
}

/**
 * Removes all elements from the input stream that `predicate` returns truthy for
 * and returns an array of the removed elements. The predicate is invoked
 * with three arguments: (value, index, array).
 *
 * **Note:** Unlike `filter`, this method mutates the input stream. Use `pull`
 * to pull elements from an array by value.
 *
 * @static
 * @param {Function} [predicate=_.identity]
 *  The function invoked per iteration.
 * @example
 *
 * var array = [1, 2, 3, 4];
 * var evens = _.remove(array, function(n) {
     *   return n % 2 == 0;
     * });
 *
 * console.log(array);
 * // => [1, 3]
 *
 * console.log(evens);
 * // => [2, 4]
 */
function remove(predicate) {
  this.spread(require('lodash.js').remove(this.collect(), predicate));
}

/**
 * Reverses the input stream so that the first element becomes the last, the second
 * element becomes the second to last, and so on.
 *
 * **Note:** This method mutates the input stream and is based on
 * [`Array#reverse`](https://mdn.io/Array/reverse).
 *
 * @static
 * @example
 *
 * var array = [1, 2, 3];
 *
 * _.reverse(array);
 * // => [3, 2, 1]
 *
 * console.log(array);
 * // => [3, 2, 1]
 */
function reverse() {
  this.spread(require('lodash.js').reverse(this.collect()));
}

/**
 * Creates a slice of the input stream from `start` up to, but not including, `end`.
 *
 * **Note:** This method is used instead of
 * [`Array#slice`](https://mdn.io/Array/slice) to ensure dense arrays are
 * returned.
 *
 * @static
 * @param {number} [start=0] The start position.
 * @param {number} [end=array.length] The end position.
 */
function slice(start, end) {
  this.spread(require('lodash.js').slice(this.collect(), start, end));
}

/**
 * Uses a binary search to determine the lowest index at which `value`
 * should be inserted into the input stream in order to maintain its sort order.
 *
 * @static
 * @param {*} value The value to evaluate.
 *  into the input stream.
 * @example
 *
 * _.sortedIndex([30, 50], 40);
 * // => 1
 */
function sortedIndex(value) {
  this.push(require('lodash.js').sortedIndex(this.collect(), value));
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
 *
 * var objects = [{ 'x': 4 }, { 'x': 5 }];
 *
 * _.sortedIndexBy(objects, { 'x': 4 }, function(o) { return o.x; });
 * // => 0
 *
 * // The `property` iteratee shorthand.
 * _.sortedIndexBy(objects, { 'x': 4 }, 'x');
 * // => 0
 */
function sortedIndexBy(value, iteratee) {
  this.push(require('lodash.js').sortedIndexBy(this.collect(), value, iteratee));
}

/**
 * This method is like `indexOf` except that it performs a binary
 * search on a sorted the input stream.
 *
 * @static
 * @param {*} value The value to search for.
 * @example
 *
 * _.sortedIndexOf([4, 5, 5, 5, 6], 5);
 * // => 1
 */
function sortedIndexOf(value) {
  this.push(require('lodash.js').sortedIndexOf(this.collect(), value));
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
 *
 * _.sortedLastIndex([4, 5, 5, 5, 6], 5);
 * // => 4
 */
function sortedLastIndex(value) {
  this.push(require('lodash.js').sortedLastIndex(this.collect(), value));
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
 *
 * var objects = [{ 'x': 4 }, { 'x': 5 }];
 *
 * _.sortedLastIndexBy(objects, { 'x': 4 }, function(o) { return o.x; });
 * // => 1
 *
 * // The `property` iteratee shorthand.
 * _.sortedLastIndexBy(objects, { 'x': 4 }, 'x');
 * // => 1
 */
function sortedLastIndexBy(value, iteratee) {
  this.push(require('lodash.js').sortedLastIndexBy(this.collect(), value, iteratee));
}

/**
 * This method is like `lastIndexOf` except that it performs a binary
 * search on a sorted the input stream.
 *
 * @static
 * @param {*} value The value to search for.
 * @example
 *
 * _.sortedLastIndexOf([4, 5, 5, 5, 6], 5);
 * // => 3
 */
function sortedLastIndexOf(value) {
  this.push(require('lodash.js').sortedLastIndexOf(this.collect(), value));
}

/**
 * This method is like `uniq` except that it's designed and optimized
 * for sorted arrays.
 *
 * @static
 * @example
 *
 * 1, 1, 2 → sortedUniq() → 1, 2
 */
function sortedUniq() {
  this.spread(require('lodash.js').sortedUniq(this.collect()));
}

/**
 * This method is like `uniqBy` except that it's designed and optimized
 * for sorted arrays.
 *
 * @static
 * @param {Function} [iteratee] The iteratee invoked per element.
 * @example
 *
 * 1.1, 1.2, 2.3, 2.4 → sortedUniqBy(Math.floor) → 1.1, 2.3
 */
function sortedUniqBy(iteratee) {
  this.spread(require('lodash.js').sortedUniqBy(this.collect(), iteratee));
}

/**
 * Gets all but the first element of the input stream.
 *
 * @static
 * @example
 *
 * 1, 2, 3 → tail() → 2, 3
 */
function tail() {
  this.spread(require('lodash.js').tail(this.collect()));
}

/**
 * Creates a slice of the input stream with `n` elements taken from the beginning.
 *
 * @static
 * @param {number} [n=1] The number of elements to take.
 * @param- {Object} [guard] Enables use as an iteratee for methods like `map`.
 * @example
 *
 * 1, 2, 3 → take() → 1
 *
 * 1, 2, 3 → take(2) → 1, 2
 *
 * 1, 2, 3 → take(5) → 1, 2, 3
 *
 * 1, 2, 3 → take(0) →
 */
function take(n) {
  this.spread(require('lodash.js').take(this.collect(), n));
}

/**
 * Creates a slice of the input stream with `n` elements taken from the end.
 *
 * @static
 * @param {number} [n=1] The number of elements to take.
 * @param- {Object} [guard] Enables use as an iteratee for methods like `map`.
 * @example
 *
 * 1, 2, 3 → takeRight() → 3
 *
 * 1, 2, 3 → takeRight(2) → 2, 3
 *
 * 1, 2, 3 → takeRight(5) → 1, 2, 3
 *
 * 1, 2, 3 → takeRight(0) →
 */
function takeRight(n) {
  this.spread(require('lodash.js').takeRight(this.collect(), n));
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
 *
 * var users = [
 *   { 'user': 'barney',  'active': true },
 *   { 'user': 'fred',    'active': false },
 *   { 'user': 'pebbles', 'active': false }
 * ];
 *
 * _.takeRightWhile(users, function(o) { return !o.active; });
 * // => objects for ['fred', 'pebbles']
 *
 * // The `matches` iteratee shorthand.
 * _.takeRightWhile(users, { 'user': 'pebbles', 'active': false });
 * // => objects for ['pebbles']
 *
 * // The `matchesProperty` iteratee shorthand.
 * _.takeRightWhile(users, ['active', false]);
 * // => objects for ['fred', 'pebbles']
 *
 * // The `property` iteratee shorthand.
 * _.takeRightWhile(users, 'active');
 * // => []
 */
function takeRightWhile(predicate) {
  this.spread(require('lodash.js').takeRightWhile(this.collect(), predicate));
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
 *
 * var users = [
 *   { 'user': 'barney',  'active': false },
 *   { 'user': 'fred',    'active': false},
 *   { 'user': 'pebbles', 'active': true }
 * ];
 *
 * _.takeWhile(users, function(o) { return !o.active; });
 * // => objects for ['barney', 'fred']
 *
 * // The `matches` iteratee shorthand.
 * _.takeWhile(users, { 'user': 'barney', 'active': false });
 * // => objects for ['barney']
 *
 * // The `matchesProperty` iteratee shorthand.
 * _.takeWhile(users, ['active', false]);
 * // => objects for ['barney', 'fred']
 *
 * // The `property` iteratee shorthand.
 * _.takeWhile(users, 'active');
 * // => []
 */
function takeWhile(predicate) {
  this.spread(require('lodash.js').takeWhile(this.collect(), predicate));
}

/**
 * Creates a stream of unique values, in order, from all given arrays using
 * [`SameValueZero`](http://ecma-international.org/ecma-262/6.0/#sec-samevaluezero)
 * for equality comparisons.
 *
 * @static
 * @param {...Array} [arrays] The arrays to inspect.
 * @example
 *
 * 2 → union([1, 2]) → 2, 1
 */
function union() {
  this.spread(require('lodash.js').union(this.collect()));
}

/**
 * This method is like `union` except that it accepts `iteratee` which is
 * invoked for each element of each `arrays` to generate the criterion by
 * which uniqueness is computed. Result values are chosen from the first
 * array in which the value occurs. The iteratee is invoked with one argument:
 * (value).
 *
 * @static
 * @param {...Array} [arrays] The arrays to inspect.
 * @param {Function} [iteratee=_.identity]
 *  The iteratee invoked per element.
 * @example
 *
 * 2.1 → unionBy([1.2, 2.3], Math.floor) → 2.1, 1.2
 *
 * // The `property` iteratee shorthand.
 * { 'x': 1 } → unionBy([{ 'x': 2 }, { 'x': 1 }], 'x') → { 'x': 1 }, { 'x': 2 }
 */
function unionBy(iteratee) {
  this.spread(require('lodash.js').unionBy(this.collect(), iteratee));
}

/**
 * This method is like `union` except that it accepts `comparator` which
 * is invoked to compare elements of `arrays`. Result values are chosen from
 * the first array in which the value occurs. The comparator is invoked
 * with two arguments: (arrVal, othVal).
 *
 * @static
 * @param {...Array} [arrays] The arrays to inspect.
 * @param {Function} [comparator] The comparator invoked per element.
 * @example
 *
 * var objects = [{ 'x': 1, 'y': 2 }, { 'x': 2, 'y': 1 }];
 * var others = [{ 'x': 1, 'y': 1 }, { 'x': 1, 'y': 2 }];
 *
 * _.unionWith(objects, others, _.isEqual);
 * // => [{ 'x': 1, 'y': 2 }, { 'x': 2, 'y': 1 }, { 'x': 1, 'y': 1 }]
 */
function unionWith(comparator) {
  this.spread(require('lodash.js').unionWith(this.collect(), comparator));
}

/**
 * Creates a duplicate-free version of an array, using
 * [`SameValueZero`](http://ecma-international.org/ecma-262/6.0/#sec-samevaluezero)
 * for equality comparisons, in which only the first occurrence of each
 * element is kept.
 *
 * @static
 * @example
 *
 * 2, 1, 2 → uniq() → 2, 1
 */
function uniq() {
  this.spread(require('lodash.js').uniq(this.collect()));
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
 *
 * 2.1, 1.2, 2.3 → uniqBy(Math.floor) → 2.1, 1.2
 *
 * // The `property` iteratee shorthand.
 * { 'x': 1 }, { 'x': 2 }, { 'x': 1 } → uniqBy('x') → { 'x': 1 }, { 'x': 2 }
 */
function uniqBy(iteratee) {
  this.spread(require('lodash.js').uniqBy(this.collect(), iteratee));
}

/**
 * This method is like `uniq` except that it accepts `comparator` which
 * is invoked to compare elements of the input stream. The comparator is invoked with
 * two arguments: (arrVal, othVal).
 *
 * @static
 * @param {Function} [comparator] The comparator invoked per element.
 * @example
 *
 * var objects = [{ 'x': 1, 'y': 2 }, { 'x': 2, 'y': 1 }, { 'x': 1, 'y': 2 }];
 *
 * _.uniqWith(objects, _.isEqual);
 * // => [{ 'x': 1, 'y': 2 }, { 'x': 2, 'y': 1 }]
 */
function uniqWith(comparator) {
  this.spread(require('lodash.js').uniqWith(this.collect(), comparator));
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
function unzip() {
  this.spread(require('lodash.js').unzip(this.collect()));
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
 *
 * var zipped = 1, 2 → zip([10, 20], [100, 200]) → [1, 10, 100], [2, 20, 200]
 *
 * _.unzipWith(zipped, _.add);
 * // => [3, 30, 300]
 */
function unzipWith(iteratee) {
  this.spread(require('lodash.js').unzipWith(this.collect(), iteratee));
}

/**
 * Creates a stream excluding all given values using
 * [`SameValueZero`](http://ecma-international.org/ecma-262/6.0/#sec-samevaluezero)
 * for equality comparisons.
 *
 * **Note:** Unlike `pull`, this method returns a new array.
 *
 * @static
 * @param {...*} [values] The values to exclude.
 * @see _.difference, _.xor
 * @example
 *
 * 2, 1, 2, 3 → without(1, 2) → 3
 */
function without() {
  var args = Array.prototype.slice.call(arguments);
  args.unshift(this.collect());
  this.spread(require('lodash.js').without.apply(null, args));
}

/**
 * Creates a stream of unique values that is the
 * [symmetric difference](https://en.wikipedia.org/wiki/Symmetric_difference)
 * of the given arrays. The order of result values is determined by the order
 * they occur in the arrays.
 *
 * @static
 * @param {...Array} [arrays] The arrays to inspect.
 * @see _.difference, _.without
 * @example
 *
 * 2, 1 → xor([2, 3]) → 1, 3
 */
function xor() {
  this.spread(require('lodash.js').xor(this.collect()));
}

/**
 * This method is like `xor` except that it accepts `iteratee` which is
 * invoked for each element of each `arrays` to generate the criterion by
 * which by which they're compared. The iteratee is invoked with one argument:
 * (value).
 *
 * @static
 * @param {...Array} [arrays] The arrays to inspect.
 * @param {Function} [iteratee=_.identity]
 *  The iteratee invoked per element.
 * @example
 *
 * 2.1, 1.2 → xorBy([2.3, 3.4], Math.floor) → 1.2, 3.4
 *
 * // The `property` iteratee shorthand.
 * { 'x': 1 } → xorBy([{ 'x': 2 }, { 'x': 1 }], 'x') → { 'x': 2 }
 */
function xorBy(iteratee) {
  this.spread(require('lodash.js').xorBy(this.collect(), iteratee));
}

/**
 * This method is like `xor` except that it accepts `comparator` which is
 * invoked to compare elements of `arrays`. The comparator is invoked with
 * two arguments: (arrVal, othVal).
 *
 * @static
 * @param {...Array} [arrays] The arrays to inspect.
 * @param {Function} [comparator] The comparator invoked per element.
 * @example
 *
 * var objects = [{ 'x': 1, 'y': 2 }, { 'x': 2, 'y': 1 }];
 * var others = [{ 'x': 1, 'y': 1 }, { 'x': 1, 'y': 2 }];
 *
 * _.xorWith(objects, others, _.isEqual);
 * // => [{ 'x': 2, 'y': 1 }, { 'x': 1, 'y': 1 }]
 */
function xorWith(comparator) {
  this.spread(require('lodash.js').xorWith(this.collect(), comparator));
}

/**
 * Creates a stream of grouped elements, the first of which contains the
 * first elements of the given arrays, the second of which contains the
 * second elements of the given arrays, and so on.
 *
 * @static
 * @example
 *
 * ["a", "b"] [1, 2] [true, false] → zip → ["a", 1, true] ["b", 2, false]
 */
function zip() {
  this.spread(require('lodash.js').zip.apply(null, this.collect()));
}

// zipObject and zipObjectDeep don't make sense

/**
 * This method is like `zip` except that it accepts `iteratee` to specify
 * how grouped values should be combined. The iteratee is invoked with the
 * elements of each group: (...group).
 *
 * @static
 * @param {...Array} [arrays] The arrays to process.
 * @param {Function} [iteratee=_.identity] The function to combine grouped values.
 * @example
 *
 * _.zipWith([1, 2], [10, 20], [100, 200], function(a, b, c) {
 *   return a + b + c;
 * });
 * // => [111, 222]
 */
function zipWith(iteratee) {
  var args = Array.prototype.slice.call(arguments);
  args.push(this.collect());
  this.spread(require('lodash.js').zipWith.apply(null, args));
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// lodash wrappers - Collection                                                                 ///
///                                                                                              ///
/// NOTE: These are not streaming!                                                               ///
////////////////////////////////////////////////////////////////////////////////////////////////////

/**
 * Creates an object composed of keys generated from the results of running
 * each element of `collection` thru `iteratee`. The corresponding value of
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
function countBy(iteratee) {
  this.push(require('lodash.js').countBy(this.collect(), iteratee));
}

/**
 * Checks if `predicate` returns truthy for **all** elements of `collection`.
 * Iteration is stopped once `predicate` returns falsey. The predicate is
 * invoked with three arguments: (value, index|key, collection).
 *
 * @static
 * @param {Function} [predicate=_.identity]
 *  The function invoked per iteration.
 * @example
 *
 * _.every([true, 1, null, 'yes'], Boolean);
 * // => false
 *
 * var users = [
 *   { 'user': 'barney', 'age': 36, 'active': false },
 *   { 'user': 'fred',   'age': 40, 'active': false }
 * ];
 *
 * // The `matches` iteratee shorthand.
 * _.every(users, { 'user': 'barney', 'active': false });
 * // => false
 *
 * // The `matchesProperty` iteratee shorthand.
 * _.every(users, ['active', false]);
 * // => true
 *
 * // The `property` iteratee shorthand.
 * _.every(users, 'active');
 * // => false
 */
function every(predicate) {
  this.push(require('lodash.js').every(this.collect(), predicate));
}

/**
 * Iterates over elements of `collection`, returning an array of all elements
 * `predicate` returns truthy for. The predicate is invoked with three
 * arguments: (value, index|key, collection).
 *
 * **Note:** Unlike `remove`, this method returns a new array.
 *
 * @static
 * @param {Function} [predicate=_.identity]
 *  The function invoked per iteration.
 * @see _.reject
 * @example
 *
 * var users = [
 *   { 'user': 'barney', 'age': 36, 'active': true },
 *   { 'user': 'fred',   'age': 40, 'active': false }
 * ];
 *
 * _.filter(users, function(o) { return !o.active; });
 * // => objects for ['fred']
 *
 * // The `matches` iteratee shorthand.
 * _.filter(users, { 'age': 36, 'active': true });
 * // => objects for ['barney']
 *
 * // The `matchesProperty` iteratee shorthand.
 * _.filter(users, ['active', false]);
 * // => objects for ['fred']
 *
 * // The `property` iteratee shorthand.
 * _.filter(users, 'active');
 * // => objects for ['barney']
 */
function filter(predicate) {
  this.spread(require('lodash.js').filter(this.collect(), predicate));
}

/**
 * Iterates over elements of `collection`, returning the first element
 * `predicate` returns truthy for. The predicate is invoked with three
 * arguments: (value, index|key, collection).
 *
 * @static
 * @param {Function} [predicate=_.identity]
 *  The function invoked per iteration.
 * @param {number} [fromIndex=0] The index to search from.
 * @example
 *
 * var users = [
 *   { 'user': 'barney',  'age': 36, 'active': true },
 *   { 'user': 'fred',    'age': 40, 'active': false },
 *   { 'user': 'pebbles', 'age': 1,  'active': true }
 * ];
 *
 * _.find(users, function(o) { return o.age < 40; });
 * // => object for 'barney'
 *
 * // The `matches` iteratee shorthand.
 * _.find(users, { 'age': 1, 'active': true });
 * // => object for 'pebbles'
 *
 * // The `matchesProperty` iteratee shorthand.
 * _.find(users, ['active', false]);
 * // => object for 'fred'
 *
 * // The `property` iteratee shorthand.
 * _.find(users, 'active');
 * // => object for 'barney'
 */
function find(predicate, fromIndex) {
  this.spread(require('lodash.js').find(this.collect(), predicate, fromIndex));
}

/**
 * This method is like `find` except that it iterates over elements of
 * `collection` from right to left.
 *
 * @static
 * @param {Function} [predicate=_.identity]
 *  The function invoked per iteration.
 * @param {number} [fromIndex=collection.length-1] The index to search from.
 * @example
 *
 * _.findLast([1, 2, 3, 4], function(n) {
 *   return n % 2 == 1;
 * });
 * // => 3
 */
function findLast(predicate, fromIndex) {
  this.spread(require('lodash.js').findLast(this.collect(), predicate, fromIndex));
}
/**
 * Creates a flattened array of values by running each element in `collection`
 * thru `iteratee` and flattening the mapped results. The iteratee is invoked
 * with three arguments: (value, index|key, collection).
 *
 * @static
 * @param {Function} [iteratee=_.identity]
 *  The function invoked per iteration.
 * @example
 *
 * function duplicate(n) {
 *   return [n, n];
 * }
 *
 * 1, 2 → flatMap(duplicate) → 1, 1, 2, 2
 */
function flatMap(iteratee) {
  this.spread(require('lodash.js').flatMap(this.collect(), iteratee));
}

/**
 * This method is like `flatMap` except that it recursively flattens the
 * mapped results.
 *
 * @static
 * @param {Function} [iteratee=_.identity]
 *  The function invoked per iteration.
 * @example
 *
 * function duplicate(n) {
 *   return [[[n, n]]];
 * }
 *
 * 1, 2 → flatMapDeep(duplicate) → 1, 1, 2, 2
 */
function flatMapDeep(iteratee) {
  this.spread(require('lodash.js').flatMapDeep(this.collect(), iteratee));
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
 *
 * function duplicate(n) {
 *   return [[[n, n]]];
 * }
 *
 * 1, 2 → flatMapDepth(duplicate, 2) → [1, 1], [2, 2]
 */
function flatMapDepth(iteratee, depth) {
  this.spread(require('lodash.js').flatMapDepth(this.collect(), iteratee, depth));
}

// forEach and forEachRight make no sense

/**
 * Creates an object composed of keys generated from the results of running
 * each element of `collection` thru `iteratee`. The order of grouped values
 * is determined by the order they occur in `collection`. The corresponding
 * value of each key is an array of elements responsible for generating the
 * key. The iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {Function} [iteratee=_.identity]
 *  The iteratee to transform keys.
 * @example
 *
 * _.groupBy([6.1, 4.2, 6.3], Math.floor);
 * // => { '4': [4.2], '6': [6.1, 6.3] }
 *
 * // The `property` iteratee shorthand.
 * _.groupBy(['one', 'two', 'three'], 'length');
 * // => { '3': ['one', 'two'], '5': ['three'] }
 */
function groupBy(iteratee) {
  this.push(require('lodash.js').groupBy(this.collect(), iteratee));
}

/**
 * Checks if `value` is in `collection`. If `collection` is a string, it's
 * checked for a substring of `value`, otherwise
 * [`SameValueZero`](http://ecma-international.org/ecma-262/6.0/#sec-samevaluezero)
 * is used for equality comparisons. If `fromIndex` is negative, it's used as
 * the offset from the end of `collection`.
 *
 * @static
 * @param {Array|Object|string} collection The collection to search.
 * @param {*} value The value to search for.
 * @param {number} [fromIndex=0] The index to search from.
 * @param- {Object} [guard] Enables use as an iteratee for methods like `reduce`.
 * @example
 *
 * _.includes([1, 2, 3], 1);
 * // => true
 *
 * _.includes([1, 2, 3], 1, 2);
 * // => false
 *
 * _.includes({ 'a': 1, 'b': 2 }, 1);
 * // => true
 *
 * _.includes('abcd', 'bc');
 * // => true
 */
function includes(value, fromIndex) {
  this.push(require('lodash.js').includes(this.collect(), value, fromIndex));
}

/**
 * Invokes the method at `path` of each element in `collection`, returning
 * an array of the results of each invoked method. Any additional arguments
 * are provided to each invoked method. If `path` is a function, it's invoked
 * for, and `this` bound to, each element in `collection`.
 *
 * @static
 * @param {Array|Function|string} path The path of the method to invoke or
 *  the function invoked per iteration.
 * @param {...*} [args] The arguments to invoke each method with.
 * @example
 *
 * [5, 1, 7 → invokeMap([3, 2, 1]], 'sort') → [1, 5, 7], [1, 2, 3]
 *
 * 123, 456 → invokeMap(String.prototype.split, '') → ['1', '2', '3'], ['4', '5', '6']
 */
function invokeMap(path) {
  var args = Array.prototype.slice.call(arguments);
  args.unshift(this.collect());
  this.spread(require('lodash.js').invokeMap.apply(null, args));
}

/**
 * Creates an object composed of keys generated from the results of running
 * each element of `collection` thru `iteratee`. The corresponding value of
 * each key is the last element responsible for generating the key. The
 * iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {Function} [iteratee=_.identity]
 *  The iteratee to transform keys.
 * @example
 *
 * var array = [
 *   { 'dir': 'left', 'code': 97 },
 *   { 'dir': 'right', 'code': 100 }
 * ];
 *
 * _.keyBy(array, function(o) {
 *   return String.fromCharCode(o.code);
 * });
 * // => { 'a': { 'dir': 'left', 'code': 97 }, 'd': { 'dir': 'right', 'code': 100 } }
 *
 * _.keyBy(array, 'dir');
 * // => { 'left': { 'dir': 'left', 'code': 97 }, 'right': { 'dir': 'right', 'code': 100 } }
 */
function keyBy(iteratee) {
  this.push(require('lodash.js').keyBy(this.collect(), iteratee));
}

/**
 * Creates a stream of values by running each element in `collection` thru
 * `iteratee`. The iteratee is invoked with three arguments:
 * (value, index|key, collection).
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
 *
 * function square(n) {
 *   return n * n;
 * }
 *
 * 4, 8 → map(square) → 16, 64
 *
 * _.map({ 'a': 4, 'b': 8 }, square);
 * // => [16, 64] (iteration order is not guaranteed)
 *
 * var users = [
 *   { 'user': 'barney' },
 *   { 'user': 'fred' }
 * ];
 *
 * // The `property` iteratee shorthand.
 * _.map(users, 'user');
 * // => ['barney', 'fred']
 */
function map(iteratee) {
  this.spread(require('lodash.js').map(this.collect(), iteratee));
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
 *
 * var users = [
 *   { 'user': 'fred',   'age': 48 },
 *   { 'user': 'barney', 'age': 34 },
 *   { 'user': 'fred',   'age': 40 },
 *   { 'user': 'barney', 'age': 36 }
 * ];
 *
 * // Sort by `user` in ascending order and by `age` in descending order.
 * _.orderBy(users, ['user', 'age'], ['asc', 'desc']);
 * // => objects for [['barney', 36], ['barney', 34], ['fred', 48], ['fred', 40]]
 */
function orderBy(iteratees, orders) {
  this.spread(require('lodash.js').orderBy(this.collect(), iteratees, orders));
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
 *
 * var users = [
 *   { 'user': 'barney',  'age': 36, 'active': false },
 *   { 'user': 'fred',    'age': 40, 'active': true },
 *   { 'user': 'pebbles', 'age': 1,  'active': false }
 * ];
 *
 * _.partition(users, function(o) { return o.active; });
 * // => objects for [['fred'], ['barney', 'pebbles']]
 *
 * // The `matches` iteratee shorthand.
 * _.partition(users, { 'age': 1, 'active': false });
 * // => objects for [['pebbles'], ['barney', 'fred']]
 *
 * // The `matchesProperty` iteratee shorthand.
 * _.partition(users, ['active', false]);
 * // => objects for [['barney', 'pebbles'], ['fred']]
 *
 * // The `property` iteratee shorthand.
 * _.partition(users, 'active');
 * // => objects for [['fred'], ['barney', 'pebbles']]
 */
function partition(predicate) {
  this.spread(require('lodash.js').partition(this.collect(), predicate));
}

/**
 * Reduces `collection` to a value which is the accumulated result of running
 * each element in `collection` thru `iteratee`, where each successive
 * invocation is supplied the return value of the previous. If `accumulator`
 * is not given, the first element of `collection` is used as the initial
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
 *
 * _.reduce([1, 2], function(sum, n) {
 *   return sum + n;
 * }, 0);
 * // => 3
 *
 * _.reduce({ 'a': 1, 'b': 2, 'c': 1 }, function(result, value, key) {
 *   (result[value] || (result[value] = [])).push(key);
 *   return result;
 * }, {});
 * // => { '1': ['a', 'c'], '2': ['b'] } (iteration order is not guaranteed)
 */
function reduce(iteratee, accumulator) {
  this.push(require('lodash.js').reduce(this.collect(), iteratee, accumulator));
}

/**
 * This method is like `reduce` except that it iterates over elements of
 * `collection` from right to left.
 *
 * @static
 * @param {Function} [iteratee=_.identity] The function invoked per iteration.
 * @param {*} [accumulator] The initial value.
 * @see _.reduce
 * @example
 *
 * var array = [[0, 1], [2, 3], [4, 5]];
 *
 * _.reduceRight(array, function(flattened, other) {
     *   return flattened.concat(other);
     * }, []);
 * // => [4, 5, 2, 3, 0, 1]
 */
function reduceRight(iteratee, accumulator) {
  this.push(require('lodash.js').reduceRight(this.collect(), iteratee, accumulator));
}

/**
 * The opposite of `filter`; this method returns the elements of `collection`
 * that `predicate` does **not** return truthy for.
 *
 * @static
 * @param {Function} [predicate=_.identity] The function invoked per iteration.
 * @see _.filter
 * @example
 *
 * var users = [
 *   { 'user': 'barney', 'age': 36, 'active': false },
 *   { 'user': 'fred',   'age': 40, 'active': true }
 * ];
 *
 * _.reject(users, function(o) { return !o.active; });
 * // => objects for ['fred']
 *
 * // The `matches` iteratee shorthand.
 * _.reject(users, { 'age': 40, 'active': true });
 * // => objects for ['barney']
 *
 * // The `matchesProperty` iteratee shorthand.
 * _.reject(users, ['active', false]);
 * // => objects for ['fred']
 *
 * // The `property` iteratee shorthand.
 * _.reject(users, 'active');
 * // => objects for ['barney']
 */
function reject(predicate) {
  this.spread(require('lodash.js').reject(this.collect(), predicate));
}

/**
 * Gets a random element from `collection`.
 *
 * @static
 * @example
 *
 * _.sample([1, 2, 3, 4]);
 * // => 2
 */
function sample() {
  this.push(require('lodash.js').sample(this.collect()));
}

/**
 * Gets `n` random elements at unique keys from `collection` up to the
 * size of `collection`.
 *
 * @static
 * @param {number} [n=1] The number of elements to sample.
 * @param- {Object} [guard] Enables use as an iteratee for methods like `map`.
 * @example
 *
 * 1, 2, 3 → sampleSize(2) → 3, 1
 *
 * 1, 2, 3 → sampleSize(4) → 2, 3, 1
 */
function sampleSize(n) {
  this.push(require('lodash.js').sampleSize(this.collect(), n));
}

/**
 * Creates a stream of shuffled values, using a version of the
 * [Fisher-Yates shuffle](https://en.wikipedia.org/wiki/Fisher-Yates_shuffle).
 *
 * @static
 * @example
 *
 * 1, 2, 3, 4 → shuffle() → 4, 1, 3, 2
 */
function shuffle() {
  this.spread(require('lodash.js').shuffle(this.collect()));
}

/**
 * Gets the size of `collection` by returning its length for array-like
 * values or the number of own enumerable string keyed properties for objects.
 *
 * @static
 * @example
 *
 * _.size([1, 2, 3]);
 * // => 3
 *
 * _.size({ 'a': 1, 'b': 2 });
 * // => 2
 *
 * _.size('pebbles');
 * // => 7
 */
function size() {
  this.push(require('lodash.js').size(this.collect()));
}

/**
 * Checks if `predicate` returns truthy for **any** element of `collection`.
 * Iteration is stopped once `predicate` returns truthy. The predicate is
 * invoked with three arguments: (value, index|key, collection).
 *
 * @static
 * @param {Function} [predicate=_.identity] The function invoked per iteration.
 * @param- {Object} [guard] Enables use as an iteratee for methods like `map`.
 *  else `false`.
 * @example
 *
 * _.some([null, 0, 'yes', false], Boolean);
 * // => true
 *
 * var users = [
 *   { 'user': 'barney', 'active': true },
 *   { 'user': 'fred',   'active': false }
 * ];
 *
 * // The `matches` iteratee shorthand.
 * _.some(users, { 'user': 'barney', 'active': false });
 * // => false
 *
 * // The `matchesProperty` iteratee shorthand.
 * _.some(users, ['active', false]);
 * // => true
 *
 * // The `property` iteratee shorthand.
 * _.some(users, 'active');
 * // => true
 */
function some(predicate) {
  this.push(require('lodash.js').some(this.collect(), predicate));
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
 *
 * var users = [
 *   { 'user': 'fred',   'age': 48 },
 *   { 'user': 'barney', 'age': 36 },
 *   { 'user': 'fred',   'age': 40 },
 *   { 'user': 'barney', 'age': 34 }
 * ];
 *
 * _.sortBy(users, function(o) { return o.user; });
 * // => objects for [['barney', 36], ['barney', 34], ['fred', 48], ['fred', 40]]
 *
 * _.sortBy(users, ['user', 'age']);
 * // => objects for [['barney', 34], ['barney', 36], ['fred', 40], ['fred', 48]]
 *
 * _.sortBy(users, 'user', function(o) {
 *   return Math.floor(o.age / 10);
 * });
 * // => objects for [['barney', 36], ['barney', 34], ['fred', 48], ['fred', 40]]
 */
function sortBy(iteratees) {
  this.spread(require('lodash.js').orderBy(this.collect(), iteratees));
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
 *
 * _.defer(function(stamp) {
 *   console.log(_.now() - stamp);
 * }, _.now());
 * // => Logs the number of milliseconds it took for the deferred invocation.
 */
function now() {
  this.push(require('lodash.js').now());
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
 *
 * _.max([4, 2, 8, 6]);
 * // => 8
 *
 * _.max([]);
 * // => undefined
 */
function max() {
  this.push(require('lodash.js').max(this.collect()));
}

/**
 * This method is like `max` except that it accepts `iteratee` which is
 * invoked for each element in the input stream to generate the criterion by which
 * the value is ranked. The iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {Function} [iteratee=_.identity] The iteratee invoked per element.
 * @example
 *
 * var objects = [{ 'n': 1 }, { 'n': 2 }];
 *
 * _.maxBy(objects, function(o) { return o.n; });
 * // => { 'n': 2 }
 *
 * // The `property` iteratee shorthand.
 * _.maxBy(objects, 'n');
 * // => { 'n': 2 }
 */
function maxBy(iteratee) {
  this.push(require('lodash.js').maxBy(this.collect(), iteratee));
}

/**
 * Computes the mean of the values in the input stream.
 *
 * @static
 * @example
 *
 * _.mean([4, 2, 8, 6]);
 * // => 5
 */
function mean() {
  this.push(require('lodash.js').mean(this.collect()));
}

/**
 * This method is like `mean` except that it accepts `iteratee` which is
 * invoked for each element in the input stream to generate the value to be averaged.
 * The iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {Function} [iteratee=_.identity] The iteratee invoked per element.
 * @example
 *
 * var objects = [{ 'n': 4 }, { 'n': 2 }, { 'n': 8 }, { 'n': 6 }];
 *
 * _.meanBy(objects, function(o) { return o.n; });
 * // => 5
 *
 * // The `property` iteratee shorthand.
 * _.meanBy(objects, 'n');
 * // => 5
 */
function meanBy(iteratee) {
  this.push(require('lodash.js').meanBy(this.collect(), iteratee));
}

/**
 * Computes the minimum value of the input stream. If the input stream is empty or falsey,
 * `undefined` is returned.
 *
 * @static
 * @example
 *
 * _.min([4, 2, 8, 6]);
 * // => 2
 *
 * _.min([]);
 * // => undefined
 */
function min() {
  this.push(require('lodash.js').min(this.collect()));
}

/**
 * This method is like `min` except that it accepts `iteratee` which is
 * invoked for each element in the input stream to generate the criterion by which
 * the value is ranked. The iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {Function} [iteratee=_.identity] The iteratee invoked per element.
 * @example
 *
 * var objects = [{ 'n': 1 }, { 'n': 2 }];
 *
 * _.minBy(objects, function(o) { return o.n; });
 * // => { 'n': 1 }
 *
 * // The `property` iteratee shorthand.
 * _.minBy(objects, 'n');
 * // => { 'n': 1 }
 */
function minBy(iteratee) {
  this.push(require('lodash.js').minBy(this.collect(), iteratee));
}

// multiply, round, subtract don't make sense

/**
 * Computes the sum of the values in the input stream.
 *
 * @static
 * @example
 *
 * _.sum([4, 2, 8, 6]);
 * // => 20
 */
function sum() {
  this.push(require('lodash.js').sum(this.collect()));
}

/**
 * This method is like `sum` except that it accepts `iteratee` which is
 * invoked for each element in the input stream to generate the value to be summed.
 * The iteratee is invoked with one argument: (value).
 *
 * @static
 * @param {Function} [iteratee=_.identity] The iteratee invoked per element.
 * @example
 *
 * var objects = [{ 'n': 4 }, { 'n': 2 }, { 'n': 8 }, { 'n': 6 }];
 *
 * _.sumBy(objects, function(o) { return o.n; });
 * // => 20
 *
 * // The `property` iteratee shorthand.
 * _.sumBy(objects, 'n');
 * // => 20
 */
function sumBy(iteratee) {
  this.push(require('lodash.js').sumBy(this.collect(), iteratee));
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
 *
 * _.random(0, 5);
 * // => an integer between 0 and 5
 *
 * _.random(5);
 * // => also an integer between 0 and 5
 *
 * _.random(5, true);
 * // => a floating-point number between 0 and 5
 *
 * _.random(1.2, 5.2);
 * // => a floating-point number between 1.2 and 5.2
 */
function random(lower, upper, floating) {
  this.push(require('lodash.js').random(lower, upper, floating));
}
