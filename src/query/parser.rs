use std::char;
use std::collections;
use std::fmt;
use std::iter;
use std::str;

use pest::prelude::*;

use error;
use query;
use value;

impl_rdp! {
    grammar! {
        query   = { process ~ (["|"] ~ process)* ~ eoi }
        process = { ident ~ expression* }

        expression = { value | lambda }

        ident = @{
            (['a'..'z'] | ['A'..'Z'] | ["_"] | ["-"]) ~ (['a'..'z'] | ['A'..'Z'] | ["_"] | ["-"] | ['0'..'9'])*
        }

        lambda = { args ~ ["=>"] ~ body }
        args = { ["("] ~ ident ~ ([","] ~ ident)* ~ [")"] }
        body = { ["{"] ~ (body | !["}"] ~ any)* ~ ["}"] }

        object = { ["{"] ~ pair ~ ([","] ~ pair)* ~ ["}"] | ["{"] ~ ["}"] }
        pair   = { (string | ident) ~ [":"] ~ value }

        array = { ["["] ~ value ~ ([","] ~ value)* ~ ["]"] | ["["] ~ ["]"] }

        value = { string | number | object | array | _true | _false | _null | ident }

        _true = { ["true"] }
        _false = { ["false"] }
        _null = { ["null"] }

        string  = @{ ["\""] ~ (escape | !(["\""] | ["\\"]) ~ any)* ~ ["\""] }
        escape  = _{ ["\\"] ~ (["\""] | ["\\"] | ["/"] | ["b"] | ["f"] | ["n"] | ["r"] | ["t"] | unicode) }
        unicode = _{ ["u"] ~ hex ~ hex ~ hex ~ hex }
        hex     = _{ ['0'..'9'] | ['a'..'f'] | ['A'..'F'] }

        number = @{ ["-"]? ~ int ~ (["."] ~ ['0'..'9']+ ~ exp? | exp)? }
        int    = _{ ["0"] | ['1'..'9'] ~ ['0'..'9']* }
        exp    = _{ (["E"] | ["e"]) ~ (["+"] | ["-"])? ~ int }

        whitespace = _{ [" "] | ["\t"] | ["\r"] | ["\n"] }
    }

    process! {
        build_query(&self) -> query::Query {
            (_: query, processes: build_processes()) => {
                let processes = processes.into_iter().collect();
                trace!("build_query processes={:?}", processes);
                query::Query(processes)
            },
        }
        build_processes(&self) -> collections::LinkedList<query::Process> {
            (_: process, process: build_process(), mut tail: build_processes()) => {
                trace!("build_processes process={:?} tail={:?}", process, tail);
                tail.push_front(process);
                tail
            },
            () => {
                collections::LinkedList::new()
            },
        }
        build_process(&self) -> query::Process {
            (&id: ident, args: build_expressions()) => {
                let id = id.to_owned();
                let args = args.into_iter().collect();
                trace!("build_process id={:?} args={:?}", id, args);
                query::Process(id, args)
            },
        }
        build_expressions(&self) -> collections::LinkedList<query::Expression> {
            (_: expression, expression: build_expression(), mut tail: build_expressions()) => {
                trace!("build_expressions expression={:?} tail={:?}", expression, tail);
                tail.push_front(expression);
                tail
            },
            () => {
                collections::LinkedList::new()
            },
        }
        build_expression(&self) -> query::Expression {
            (_: value, value: build_value()) => {
                trace!("build_expression value={:?}", value);
                query::Expression::Value(value)
            },
            (_: lambda, _: args, args: build_args(), &body: body) => {
                let args = args.into_iter().collect();
                let body = body[1..body.len() - 1].to_owned();
                trace!("build_expression args={:?} body={:?}", args, body);
                query::Expression::Function(args, body)
            },
        }
        build_value(&self) -> value::Value {
            (&string: string) => {
                let string = unescape_string(string);
                trace!("build_value string={:?}", string);
                value::Value::String(string)
            },
            (&number: number) => {
                let number = number.parse().unwrap();
                trace!("build_value number={:?}", number);
                value::Value::from_f64(number)
            },
            (&ident: ident) => {
                let ident = ident.to_owned();
                trace!("build_value ident={:?}", ident);
                value::Value::String(ident)
            },
            (_: object, object: build_object()) => {
                trace!("build_value object={:?}", object);
                value::Value::Map(object)
            },
            (_: array, array: build_array()) => {
                let array = array.into_iter().collect();
                trace!("build_value array={:?}", array);
                value::Value::Sequence(array)
            },
            (_: _true) => {
                trace!("build_value bool=true");
                value::Value::Bool(true)
            },
            (_: _false) => {
                trace!("build_value bool=false");
                value::Value::Bool(false)
            },
            (_: _null) => {
                trace!("build_value null");
                value::Value::Unit
            },
        }
        build_args(&self) -> collections::LinkedList<String> {
            (&arg: ident, mut tail: build_args()) => {
                trace!("build_args arg={:?} tail={:?}", arg, tail);
                tail.push_front(arg.to_owned());
                tail
            },
            () => {
                collections::LinkedList::new()
            },
        }
        build_object(&self) -> collections::BTreeMap<value::Value, value::Value> {
            (_: pair, pair: build_pair(), mut tail: build_object()) => {
                trace!("build_object pair={:?} tail={:?}", pair, tail);
                tail.insert(pair.0, pair.1);
                tail
            },
            () => {
                collections::BTreeMap::new()
            },
        }
        build_pair(&self) -> (value::Value, value::Value) {
            (&key: ident, _: value, value: build_value()) => {
                let key = key.to_owned();
                trace!("build_pair key={:?} value={:?}", key, value);
                (value::Value::String(key), value)
            },
            (&key: string, _: value, value: build_value()) => {
                let key = unescape_string(key);
                trace!("build_pair key={:?} value={:?}", key, value);
                (value::Value::String(key), value)
            },
        }
        build_array(&self) -> collections::LinkedList<value::Value> {
            (_: value, value: build_value(), mut tail: build_array()) => {
                trace!("build_array value={:?} tail={:?}", value, tail);
                tail.push_front(value);
                tail
            },
            () => {
                collections::LinkedList::new()
            },
        }
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let d = match *self {
            Rule::query => "query",
            Rule::process => "process",
            Rule::expression => "expression",
            Rule::ident => "identifier",
            Rule::lambda => "lambda",
            Rule::args => "lambda argument list",
            Rule::body => "lambda body",
            Rule::object => "object literal",
            Rule::pair => "object key/value pair",
            Rule::array => "array literal",
            Rule::value => "value",
            Rule::_true => "\"true\"",
            Rule::_false => "\"false\"",
            Rule::_null => "\"null\"",
            Rule::string => "string literal",
            Rule::number => "number",

            Rule::any => "any character",
            Rule::soi => "start of input",
            Rule::eoi => "end of input",
        };

        f.write_str(d)
    }
}

pub fn parse_query(input: &str) -> error::Result<query::Query> {
    let mut parser = Rdp::new(StringInput::new(input));
    if parser.query() {
        Ok(parser.build_query())
    } else {
        let (ref rules, pos) = parser.expected();
        let description = if rules.len() == 1 {
            format!("unexpected input at {}, expected {}", pos, rules[0])
        } else {
            let rule_strings = rules.iter()
                .map(|r| format!("{}", r))
                .collect::<Vec<_>>();
            let rule_desc = format!("{} or {}",
                                    rule_strings[0..rule_strings.len() - 1].join(", "),
                                    rule_strings[rule_strings.len() - 1]);
            format!("unexpected input at {}; expected one of {}", pos, rule_desc)
        };

        let spaces = iter::repeat(' ').take(pos).collect::<String>();
        let msg = format!("{}\n{}\n{}^", description, input, spaces);

        Err(error::ErrorKind::SyntaxError(msg).into())
    }
}

fn unescape_string(string: &str) -> String {
    let mut result = String::with_capacity(string.len());
    let mut chars = string[1..string.len() - 1].chars();

    while let Some(c) = chars.next() {
        let r = match c {
            '\\' => {
                let e = chars.next().unwrap();
                match e {
                    '"' | '\\' | '/' => e,
                    'b' => '\x08',
                    'f' => '\x0c',
                    'n' => '\x0a',
                    'r' => '\x0d',
                    't' => '\x09',
                    'u' => decode_hex_escape(&mut chars),
                    _ => unreachable!(),
                }
            },
            _ => c,
        };
        result.push(r);
    }

    result
}

fn decode_hex_escape(chars: &mut str::Chars) -> char {
    let p1 = hex_chars([chars.next().unwrap(),
                        chars.next().unwrap(),
                        chars.next().unwrap(),
                        chars.next().unwrap()]);

    // TODO: raise error instead
    match p1 {
        0xdc00...0xdfff => panic!("Leading surrogate"),
        0xd800...0xdbff => {
            if '\\' != chars.next().unwrap() {
                panic!("Expected another escape sequence");
            }
            if 'u' != chars.next().unwrap() {
                panic!("Expected another Unicode escape sequence");
            }
            let p2 = hex_chars([chars.next().unwrap(),
                                chars.next().unwrap(),
                                chars.next().unwrap(),
                                chars.next().unwrap()]);

            let p = (((p1 - 0xD800) as u32) << 10 | (p2 - 0xDC00) as u32) + 0x1_0000;
            match char::from_u32(p as u32) {
                Some(c) => c,
                None => panic!("Illegal Unicode code point {}", p),
            }
        },
        _ => {
            match char::from_u32(p1 as u32) {
                Some(c) => c,
                None => panic!("Illegal Unicode code point {}", p1),
            }
        },
    }
}

fn hex_chars(hs: [char; 4]) -> u16 {
    let mut code_point = 0u16;
    for h in hs.iter() {
        let h = *h;
        let n = match h {
            '0'...'9' => '0' as u16 - h as u16,
            'a' | 'A' => 0xa,
            'b' | 'B' => 0xb,
            'c' | 'C' => 0xc,
            'd' | 'D' => 0xd,
            'e' | 'E' => 0xe,
            'f' | 'F' => 0xf,
            _ => unreachable!(),
        };
        code_point = code_point * 16 + n;
    }
    code_point
}
