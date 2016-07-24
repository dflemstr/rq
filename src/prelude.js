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
