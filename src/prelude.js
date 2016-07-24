/**
 * This is the rq standard library as implemented in Javascript.
 */

/**
 * Passes through all of the values it sees untouched.
 *
 * ```
 * {a: 2, b: 3} → id() → {a: 2, b: 3}
 * true         → id() → true
 * ```
 *
 * @this {rq.Context}
 */
function id() {
  while (this.await()) {
    this.emit(this.value);
  }
}

/**
 * Selects the field at the specified path for each value in the stream.
 *
 * ```
 * {a: {b: {c: 3} } } → select('/a/b') → {c: 3}
 * ```
 *
 * @param {string} path the field path to follow
 * @this {rq.Context}
 */
function select(path) {
  var self = this;
  while (this.await()) {
    var lenses = rq.util.path(this.value, path);
    if (lenses.length > 0) {
      for (var i = 0; i < lenses.length; i++) {
        var lens = lenses[i];
        var value = lens.get();
        self.log.debug('selecting', JSON.stringify(value), 'for path', JSON.stringify(path));
        self.emit(value);
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
 * ```
 * {a: {b: 2, c: true} } → modify('/a/b', n => n + 2) → {a: {b: 4, c: true} }
 * ```
 *
 * @param {string} path the field path to follow
 * @param {function(*): *} f the function to apply
 * @this {rq.Context}
 */
function modify(path, f) {
  while (this.await()) {
    var lenses = rq.util.path(this.value, path);
    for (var i = 0; i < lenses.length; i++) {
      var lens = lenses[i];
      lens.set(f(lens.get()));
    }
    this.emit(this.value);
  }
}

function tee() {
  while (this.await()) {
    this.log.info(JSON.stringify(this.value));
    this.emit(this.value);
  }
}

function collect() {
  this.emit(this.collect());
}

function spread() {
  while (this.await()) {
    if (Array.isArray(this.value)) {
      this.spread(this.value);
    } else {
      this.emit(this.value);
    }
  }
}

///
/// lodash wrappers - Array
///
/// NOTE: These are not streaming!
///

function chunk(size) {
  this.spread(require('lodash.js').chunk(this.collect(), size));
}

function compact() {
  this.spread(require('lodash.js').compact(this.collect()));
}

function concat() {
  this.spread(require('lodash.js').concat.apply(null, this.collect()));
}

function difference(values) {
  this.spread(require('lodash.js').difference(this.collect(), values));
}

function differenceBy(values, iteratee) {
  this.spread(require('lodash.js').differenceBy(this.collect(), values, iteratee));
}

function differenceWith(values, comparator) {
  this.spread(require('lodash.js').differenceWith(this.collect(), values, comparator));
}

function drop(n) {
  this.spread(require('lodash.js').drop(this.collect(), n));
}

function dropRight(n) {
  this.spread(require('lodash.js').dropRight(this.collect(), n));
}

function dropRightWhile(n, predicate) {
  this.spread(require('lodash.js').dropRightWhile(this.collect(), n, predicate));
}

function dropWhile(n, predicate) {
  this.spread(require('lodash.js').dropWhile(this.collect(), n, predicate));
}

function fill(value, start, end) {
  this.spread(require('lodash.js').fill(this.collect(), value, start, end));
}

function findIndex(predicate, fromIndex) {
  this.emit(require('lodash.js').findIndex(this.collect(), predicate, fromIndex));
}

function findLastIndex(predicate, fromIndex) {
  this.emit(require('lodash.js').findLastIndex(this.collect(), predicate, fromIndex));
}

function flatten() {
  this.spread(require('lodash.js').flatten(this.collect()));
}

function flattenDeep() {
  this.spread(require('lodash.js').flattenDeep(this.collect()));
}

function flattenDepth(n) {
  this.spread(require('lodash.js').flattenDepth(this.collect(), n));
}

function fromPairs() {
  this.emit(require('lodash.js').fromPairs(this.collect()));
}

function head() {
  this.emit(require('lodash.js').head(this.collect()));
}

function indexOf(value, fromIndex) {
  this.emit(require('lodash.js').indexOf(this.collect(), value, fromIndex));
}

function initial() {
  this.emit(require('lodash.js').initial(this.collect()));
}

function intersection(values) {
  this.spread(require('lodash.js').intersection(this.collect(), values));
}

function intersectionBy(values, iteratee) {
  this.spread(require('lodash.js').intersectionBy(this.collect(), values, iteratee));
}

function intersectionWith(values, comparator) {
  this.spread(require('lodash.js').intersectionWith(this.collect(), values, comparator));
}

function join(separator) {
  this.emit(require('lodash.js').join(this.collect(), separator));
}

function last() {
  this.emit(require('lodash.js').last(this.collect()));
}

function lastIndexOf(value, fromIndex) {
  this.emit(require('lodash.js').lastIndexOf(this.collect(), value, fromIndex));
}

function nth(n) {
  this.emit(require('lodash.js').nth(this.collect(), n));
}

function pull() {
  var args = Array.prototype.slice.call(arguments);
  args.unshift(this.collect());
  this.spread(require('lodash.js').pull.apply(null, args));
}

function pullAll(values) {
  this.spread(require('lodash.js').pullAll(this.collect(), values));
}

function pullAllBy(values, iteratee) {
  this.spread(require('lodash.js').pullAllBy(this.collect(), values, iteratee));
}

function pullAllWith(values, comparator) {
  this.spread(require('lodash.js').pullAllWith(this.collect(), values, comparator));
}

function pullAt(indexes) {
  var result = this.collect();
  require('lodash.js').pullAt(result, indexes);
  this.spread(result);
}

function remove(predicate) {
  this.spread(require('lodash.js').remove(this.collect(), predicate));
}

function reverse() {
  this.spread(require('lodash.js').reverse(this.collect()));
}

function slice(start, end) {
  this.spread(require('lodash.js').slice(this.collect(), start, end));
}

function sortedIndex(value) {
  this.emit(require('lodash.js').sortedIndex(this.collect(), value));
}

function sortedIndexBy(value, iteratee) {
  this.emit(require('lodash.js').sortedIndexBy(this.collect(), value, iteratee));
}

function sortedIndexOf(value) {
  this.emit(require('lodash.js').sortedIndexOf(this.collect(), value));
}

function sortedLastIndex(value) {
  this.emit(require('lodash.js').sortedLastIndex(this.collect(), value));
}

function sortedLastIndexBy(value) {
  this.emit(require('lodash.js').sortedLastIndexBy(this.collect(), value));
}

function sortedLastIndexOf(value) {
  this.emit(require('lodash.js').sortedLastIndexOf(this.collect(), value));
}

function sortedUniq() {
  this.spread(require('lodash.js').sortedUniq(this.collect()));
}

function sortedUniqBy(iteratee) {
  this.spread(require('lodash.js').sortedUniqBy(this.collect(), iteratee));
}

function tail() {
  this.spread(require('lodash.js').tail(this.collect()));
}

function take(n) {
  this.spread(require('lodash.js').take(this.collect(), n));
}

function takeRight(n) {
  this.spread(require('lodash.js').takeRight(this.collect(), n));
}

function takeRightWhile(predicate) {
  this.spread(require('lodash.js').takeRightWhile(this.collect(), predicate));
}

function takeWhile(predicate) {
  this.spread(require('lodash.js').takeWhile(this.collect(), predicate));
}

function union() {
  this.spread(require('lodash.js').union(this.collect()));
}

function unionBy(iteratee) {
  this.spread(require('lodash.js').unionBy(this.collect(), iteratee));
}

function unionWith(comparator) {
  this.spread(require('lodash.js').unionWith(this.collect(), comparator));
}

function uniq() {
  this.spread(require('lodash.js').uniq(this.collect()));
}

function uniqBy(iteratee) {
  this.spread(require('lodash.js').uniqBy(this.collect(), iteratee));
}

function uniqWith(comparator) {
  this.spread(require('lodash.js').uniqWith(this.collect(), comparator));
}

function unzip() {
  this.spread(require('lodash.js').unzip(this.collect()));
}

function unzipWith(iteratee) {
  this.spread(require('lodash.js').unzipWith(this.collect(), iteratee));
}

function without() {
  var args = Array.prototype.slice.call(arguments);
  args.unshift(this.collect());
  this.spread(require('lodash.js').without.apply(null, args));
}

function xor() {
  this.spread(require('lodash.js').xor(this.collect()));
}

function xorBy(iteratee) {
  this.spread(require('lodash.js').xorBy(this.collect(), iteratee));
}

function xorWith(comparator) {
  this.spread(require('lodash.js').xorWith(this.collect(), comparator));
}

function zip() {
  this.spread(require('lodash.js').zip.apply(null, this.collect()));
}

// zipObject and zipObjectDeep don't make sense

function zipWith(iteratee) {
  var args = Array.prototype.slice.call(arguments);
  args.push(this.collect());
  this.spread(require('lodash.js').zipWith.apply(null, args));
}

///
/// lodash wrappers - Collection
///
/// NOTE: These are not streaming!
///

function countBy(iteratee) {
  this.emit(require('lodash.js').countBy(this.collect(), iteratee));
}

function every(predicate) {
  this.emit(require('lodash.js').every(this.collect(), predicate));
}

function filter(predicate) {
  this.spread(require('lodash.js').filter(this.collect(), predicate));
}

function find(predicate, fromIndex) {
  this.spread(require('lodash.js').find(this.collect(), predicate, fromIndex));
}

function findLast(predicate, fromIndex) {
  this.spread(require('lodash.js').findLast(this.collect(), predicate, fromIndex));
}

function flatMap(iteratee) {
  this.spread(require('lodash.js').flatMap(this.collect(), iteratee));
}

function flatMapDeep(iteratee) {
  this.spread(require('lodash.js').flatMapDeep(this.collect(), iteratee));
}

function flatMapDepth(iteratee, depth) {
  this.spread(require('lodash.js').flatMapDepth(this.collect(), iteratee, depth));
}

// forEach and forEachRight make no sense

function groupBy(iteratee) {
  this.emit(require('lodash.js').groupBy(this.collect(), iteratee));
}

function includes(value, fromIndex) {
  this.emit(require('lodash.js').includes(this.collect(), value, fromIndex));
}

function invokeMap(path) {
  var args = Array.prototype.slice.call(arguments);
  args.unshift(this.collect());
  this.spread(require('lodash.js').invokeMap.apply(null, args));
}

function keyBy(iteratee) {
  this.emit(require('lodash.js').keyBy(this.collect(), iteratee));
}

function map(iteratee) {
  this.spread(require('lodash.js').map(this.collect(), iteratee));
}

function orderBy(iteratees, orders) {
  this.spread(require('lodash.js').orderBy(this.collect(), iteratees, orders));
}

function partition(predicate) {
  this.spread(require('lodash.js').partition(this.collect(), predicate));
}

function reduce(iteratee, accumulator) {
  this.emit(require('lodash.js').reduce(this.collect(), iteratee, accumulator));
}

function reduceRight(iteratee, accumulator) {
  this.emit(require('lodash.js').reduceRight(this.collect(), iteratee, accumulator));
}

function reject(predicate) {
  this.spread(require('lodash.js').reject(this.collect(), predicate));
}

function sample() {
  this.emit(require('lodash.js').sample(this.collect()));
}

function sampleSize(n) {
  this.emit(require('lodash.js').sampleSize(this.collect(), n));
}

function shuffle() {
  this.spread(require('lodash.js').shuffle(this.collect()));
}

function size() {
  this.emit(require('lodash.js').size(this.collect()));
}

function some(predicate) {
  this.emit(require('lodash.js').some(this.collect(), predicate));
}

function sortBy(iteratees) {
  this.spread(require('lodash.js').orderBy(this.collect(), iteratees));
}

///
/// lodash wrappers - Date
///
/// NOTE: These are not streaming!
///

function now() {
  this.emit(require('lodash.js').now());
}

///
/// lodash wrappers - Function/Lang don't make sense
///

///
/// lodash wrappers - Math
///
/// NOTE: These are not streaming!
///

// add, ceil, divide, floor don't make sense

function max() {
  this.emit(require('lodash.js').max(this.collect()));
}

function maxBy(iteratee) {
  this.emit(require('lodash.js').maxBy(this.collect(), iteratee));
}

function mean() {
  this.emit(require('lodash.js').mean(this.collect()));
}

function meanBy(iteratee) {
  this.emit(require('lodash.js').meanBy(this.collect(), iteratee));
}

function min() {
  this.emit(require('lodash.js').min(this.collect()));
}

function minBy(iteratee) {
  this.emit(require('lodash.js').minBy(this.collect(), iteratee));
}

// multiply, round, subtract don't make sense

function sum() {
  this.emit(require('lodash.js').sum(this.collect()));
}

function sumBy(iteratee) {
  this.emit(require('lodash.js').sumBy(this.collect(), iteratee));
}

///
/// lodash wrappers - Number
///
/// NOTE: These are not streaming!
///

// clamp, inRange don't make sense

function random(lower, upper, floating) {
  this.emit(require('lodash.js').random(lower, upper, floating));
}
