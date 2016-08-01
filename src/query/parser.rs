use std::char;
use std::collections;
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

        expression = { value | function }

        ident = @{
            (['a'..'z'] | ['A'..'Z'] | ["_"]) ~ (['a'..'z'] | ['A'..'Z'] | ["_"] | ['0'..'9'])*
        }

        function = { args ~ ["=>"] ~ body }
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
        escape  = { ["\\"] ~ (["\""] | ["\\"] | ["/"] | ["b"] | ["f"] | ["n"] | ["r"] | ["t"] | unicode) }
        unicode = { ["u"] ~ hex ~ hex ~ hex ~ hex }
        hex     = { ['0'..'9'] | ['a'..'f'] | ['A'..'F'] }

        number = @{ ["-"]? ~ int ~ (["."] ~ ['0'..'9']+ ~ exp? | exp)? }
        int    =  { ["0"] | ['1'..'9'] ~ ['0'..'9']* }
        exp    =  { (["E"] | ["e"]) ~ (["+"] | ["-"])? ~ int }

        whitespace = _{ [" "] | ["\t"] | ["\r"] | ["\n"] }
    }

    process! {
        build_query(&self) -> query::Query {
            (_: query, processes: build_processes()) =>
                query::Query(processes.into_iter().collect()),
        }
        build_processes(&self) -> collections::LinkedList<query::Process> {
            (_: process, process: build_process(), mut tail: build_processes()) => {
                tail.push_front(process);
                tail
            },
            () => {
                collections::LinkedList::new()
            },
        }
        build_process(&self) -> query::Process {
            (&id: ident, args: build_expressions()) => {
                query::Process(id.to_owned(), args.into_iter().collect())
            },
        }
        build_expressions(&self) -> collections::LinkedList<query::Expression> {
            (_: expression, expression: build_expression(), mut tail: build_expressions()) => {
                tail.push_front(expression);
                tail
            },
            () => {
                collections::LinkedList::new()
            },
        }
        build_expression(&self) -> query::Expression {
            (_: value, value: build_value()) => {
                query::Expression::Value(value)
            },
            (_: function, _: args, args: build_args(), &body: body) => {
                query::Expression::Function(args.into_iter().collect(), body.to_owned())
            },
        }
        build_value(&self) -> value::Value {
            (&string: string) => {
                value::Value::String(unescape_string(string))
            },
            (&number: number) => {
                value::Value::from_f64(number.parse().unwrap())
            },
            (&ident: ident) => {
                value::Value::String(ident.to_owned())
            },
            (_: object, object: build_object()) => {
                value::Value::Map(object)
            },
            (_: array, array: build_array()) => {
                value::Value::Sequence(array.into_iter().collect())
            },
            (_: _true) => value::Value::Bool(true),
            (_: _false) => value::Value::Bool(false),
            (_: _null) => value::Value::Unit,
        }
        build_args(&self) -> collections::LinkedList<String> {
            (&arg: ident, mut tail: build_args()) => {
                tail.push_front(arg.to_owned());
                tail
            },
            () => {
                collections::LinkedList::new()
            },
        }
        build_object(&self) -> collections::BTreeMap<value::Value, value::Value> {
            (_: pair, pair: build_pair(), mut tail: build_object()) => {
                tail.insert(pair.0, pair.1);
                tail
            },
            () => {
                collections::BTreeMap::new()
            },
        }
        build_pair(&self) -> (value::Value, value::Value) {
            (&key: ident, _: value, value: build_value()) => {
                (value::Value::String(key.to_owned()), value)
            },
            (&key: string, _: value, value: build_value()) => {
                (value::Value::String(unescape_string(key)), value)
            },
        }
        build_array(&self) -> collections::LinkedList<value::Value> {
            (_: value, value: build_value(), mut tail: build_array()) => {
                tail.push_front(value);
                tail
            },
            () => {
                collections::LinkedList::new()
            },
        }
    }
}

pub fn parse_query(input: &str) -> error::Result<query::Query> {
    let mut parser = Rdp::new(StringInput::new(input));
    if parser.query() {
        Ok(parser.build_query())
    } else {
        let (ref rules, pos) = parser.expected();
        let description = if rules.len() == 1 {
            format!("unexpected input at {}, expected {:?}", pos, rules[0])
        } else {
            let rule_strings = rules.iter()
                .map(|r| format!("{:?}", r))
                .collect::<Vec<_>>();
            let rule_desc =
                format!("{} or {}",
                        rule_strings[0..rule_strings.len() - 1].join(", "),
                        rule_strings[rule_strings.len() - 1]);
            format!("unexpected input at {}; expected one of {}",
                    pos, rule_desc)
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
    let p1 = hex_chars(
        [chars.next().unwrap(),
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
            let p2 = hex_chars(
                [chars.next().unwrap(),
                 chars.next().unwrap(),
                 chars.next().unwrap(),
                 chars.next().unwrap()]);

            let p = (((p1 - 0xD800) as u32) << 10 |
                     (p2 - 0xDC00) as u32) + 0x1_0000;
            match char::from_u32(p as u32) {
                Some(c) => c,
                None => panic!("Illegal Unicode code point {}", p),
            }
        }
        _ => {
            match char::from_u32(p1 as u32) {
                Some(c) => c,
                None => panic!("Illegal Unicode code point {}", p1),
            }
        }
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
