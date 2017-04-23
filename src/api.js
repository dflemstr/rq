"use strict";
/**
 * This file contains the rq JavaScript API implementation.  It's used
 * by for example the `prelude` module.
 * @module api
 * @private
 */

/**
 * The `rq` namespace, containing the `rq` API.
 *
 * @namespace
 * @private
 */
var rq = rq || {};

/**
 * The type of `this` for all of the rq stream processing functions (defined for example in
 * `prelude.js`)
 *
 * @param {rq.Logger} log The logger to use in this context.
 * @constructor
 * @private
 */
rq.Context = function Context(log) {
  /**
   * A logger object that can be used to send log messages to the user.
   *
   * @type {rq.Logger}
   * @private
   */
  this.log = log; // Writable because Process overwrites it.

  /**
   * The current value from the input value stream.  Will be `undefined` until {@link
   * rq.Context#pull} has been called and returned `true`.
   * @private
   */
  this.value = undefined;

  /**
   * Pulls the next value in the input value stream, storing it into `this.value`.
   *
   * @return {boolean} Whether there was another value in the stream.
   * @private
   */
  this.pull = function* pull() {
    var result = yield {type: 'await'};

    if (result.hasNext) {
      this.value = result.next;
      return true;
    } else {
      return false;
    }
  };

  /**
   * Pushes a value into the output value stream.
   *
   * @param {*} value The value to push.
   * @private
   */
  this.push = function* push(value) {
    yield {type: 'emit', value: value};
  };

  /**
   * Collects all values from the input stream, consuming it fully.
   *
   * @returns {Array} The values that were in the input stream.
   * @private
   */
  this.collect = function* collect() {
    var result = [];
    while (yield* this.pull()) {
      result.push(this.value);
    }
    return result;
  };

  /**
   * Spreads the specified values into the output stream, pushing each of them in order.
   *
   * @param {Array} values The values to push to the output stream.
   * @private
   */
  this.spread = function* spread(values) {
    for (var i = 0; i < values.length; i++) {
      yield* this.push(values[i]);
    }
  };

  Object.seal(this);
};

/**
 * A logger that can be used to log messages.
 *
 * @param {string} name The name of the logger.
 * @constructor
 * @private
 */
rq.Logger = function Logger(name) {
  /**
   * Logs something at the trace level.
   *
   * @param {...*} args Arbitrary values to log.
   */
  this.trace = function trace(args) {
    rq.native.log(0, name, ...arguments);
  };

  /**
   * Logs something at the debug level.
   *
   * @param {...*} args Arbitrary values to log.
   * @private
   */
  this.debug = function debug(args) {
    rq.native.log(1, name, ...arguments);
  };

  /**
   * Logs something at the info level.
   *
   * @param {...*} args Arbitrary values to log.
   * @private
   */
  this.info = function info(args) {
    rq.native.log(2, name, ...arguments);
  };

  /**
   * Logs something at the warning level.
   *
   * @param {...*} args Arbitrary values to log.
   * @private
   */
  this.warn = function warn(args) {
    rq.native.log(3, name, ...arguments);
  };

  /**
   * Logs something at the error level.
   *
   * @param {...*} args Arbitrary values to log.
   * @private
   */
  this.error = function error(args) {
    rq.native.log(4, name, ...arguments);
  };

  Object.freeze(this);
};

/**
 * Utility functions used by many rq processes.
 *
 * @namespace
 * @private
 */
rq.util = {};

/**
 * The log object used by this module.
 *
 * @type {rq.Logger}
 * @private
 */
Object.defineProperty(rq.util, 'log', {value: new rq.Logger('rq.util')});

/**
 * A lens that can be used to interact with some encapsulated value.
 *
 * @param {function(): *} get The getter for the value.
 * @param {function(*)} set The setter for the value.
 * @constructor
 * @private
 */
rq.util.Lens = function Lens(get, set) {
  /**
   * Gets the encapsulated value.
   * @return {*} The current value.
   * @private
   */
  this.get = get;

  /**
   * Sets the encapsulated value.
   * @param {*} value The new value to set.
   * @private
   */
  this.set = set;

  Object.freeze(this);
};

/**
 * Evaluates a path into an object, returning an array of `Lens`es with the targets of the path.
 *
 * The supported path syntaxes include [JSONPath][1] and [JSON pointers][2].
 *
 * [1]: https://github.com/dchester/jsonpath
 * [2]: https://tools.ietf.org/html/rfc6901
 *
 * @param {(Object|Array)} obj The object to traverse.
 * @param {string} path The path into the object.
 * @return {Array<rq.util.Lens>} A lens that can be used to manipulate the targeted values.
 * @private
 */
rq.util.path = function path(obj, path) {
  if (typeof path === 'string' && path.length > 0) {
    if (path.charAt(0) === '/') {
      // Assume it's a JSON pointer

      var elems = path.substring(1).split(/\//).map(function unescape(elem) {
        return elem.replace(/~1/g, '/').replace(/~2/g, '~');
      });

      if (elems.length === 0) {
        throw new Error(`Path projection is empty: ${JSON.stringify(path)}`);
      }

      var last = elems.pop();

      elems.forEach(function(elem) {
        if (obj && elem in obj) {
          obj = obj[elem];
        } else {
          obj = undefined;
        }
      });

      if (obj && last in obj) {
        return [new rq.util.Lens(function get() {
          return obj[last];
        }, function set(v) {
          obj[last] = v;
        })];
      } else {
        return [];
      }
    } else {
      throw new Error(`Unrecognized path syntax: ${JSON.stringify(path)}`);
    }
  } else {
    throw new Error(`Cannot be used as a path: ${JSON.stringify(path)}`);
  }
};

Object.freeze(rq.util);

/**
 * An rq process that encapsulates a coroutine.  It's probably not a good idea to construct an
 * instance of this manually.
 *
 * @constructor
 * @private
 */
rq.Process = function Process(fn) {
  var ctx = new rq.Context(new rq.Logger(fn.name));
  var generator = undefined;

  this.resume = function resume(params) {
    switch (params.type) {
    case 'start': {
      // Replace logger by more detailed one
      var name = `${fn.name}(${params.args.map(JSON.stringify).join(', ')})`;
      ctx.log = new rq.Logger(name);
      generator = fn.apply(ctx, params.args);
      break;
    }
    case 'pending': {
      return generator.next().value;
    }
    case 'await': {
      return generator.next(params).value;
    }
    default:
      throw Error(`Unrecognized resume type ${params.type}`);
    }
  };

  Object.freeze(this);
};

Object.freeze(rq);
