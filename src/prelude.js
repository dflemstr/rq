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
 * @template A
 * @param {string} path the field path to follow
 * @param {function(A): A} f the function to apply
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
