/**
 * This file contains the rq Javascript API implementation.  It's used by for example `prelude.js`.
 */

/**
 * The `rq` namespace, containing the `rq` API.
 */
rq = {};

/**
 * The type of `this` for all of the rq stream processing functions (defined for example in
 * `prelude.js`)
 *
 * @constructor
 */
rq.Context = function Context() {
};

/**
 * Awaits the next value in the input value stream.
 *
 * @return {boolean} whether there is another value in the stream; this value is available as
 *   `this.value`.
 */
rq.Context.prototype.await = function await() {
  var result = Duktape.Thread.yield({type: 'await'});

  if (result.hasNext) {
    this.value = result.next;
    return true;
  } else {
    return false;
  }
};

/**
 * Emits a value value into the output value stream.
 *
 * @param {*} value The value to emit.
 */
rq.Context.prototype.emit = function emit(value) {
  Duktape.Thread.yield({type: 'emit', value: value});
};

/**
 * The current value from the input value stream.  Will be `undefined` until `this.await()` has been
 * called and returned `true`.
 *
 * @typedef {*}
 */
rq.Context.prototype.value = undefined;

/**
 * An rq process that encapsulates a coroutine.
 *
 * @constructor
 */
rq.Process = function Process(fn) {
  this.run = function run(args) {
    fn.apply(new rq.Context(), args);
  };
  this.resume = function resume(thread, result) {
    Duktape.Thread.resume(thread, result);
  };
};
