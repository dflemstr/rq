use std::collections;
use std::iter;

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
        body = { ["{"] ~ (body | any)* ~ ["}"] }

        object = { ["{"] ~ pair ~ ([","] ~ pair)* ~ ["}"] | ["{"] ~ ["}"] }
        pair   = { string ~ [":"] ~ value }

        array = { ["["] ~ value ~ ([","] ~ value)* ~ ["]"] | ["["] ~ ["]"] }

        value = { string | number | object | array | ["true"] | ["false"] | ["null"] }

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
            (_: function, _: args, args: build_args(), _: body, body: build_body()) => {
                query::Expression::Function(args.into_iter().collect(), body)
            },
        }
        build_value(&self) -> value::Value {
            (_: string, string: build_string()) => {
                value::Value::String(string)
            },
            (&number: number) => {
                value::Value::from_f64(number.parse().unwrap())
            },
            (_: object, object: build_object()) => {
                value::Value::Map(object)
            },
            (_: array, array: build_array()) => {
                value::Value::Sequence(array.into_iter().collect())
            },
            (&val) => {
                match val {
                    "true" => value::Value::Bool(true),
                    "false" => value::Value::Bool(false),
                    "null" => value::Value::Unit,
                    _ => unreachable!(),
                }
            },
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
        build_body(&self) -> String {
            (&body) => {
                body.to_owned()
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
            (key: build_string(), value: build_value()) => {
                (value::Value::String(key), value)
            },
        }
        build_array(&self) -> collections::LinkedList<value::Value> {
            (value: build_value(), mut tail: build_array()) => {
                tail.push_front(value);
                tail
            },
            () => {
                collections::LinkedList::new()
            },
        }
        build_string(&self) -> String {
            (&string: string) => string.parse::<String>().unwrap(),
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
            let rule_desc = rules.iter()
                .map(|r| format!("{:?}", r))
                .collect::<Vec<_>>()
                .join(", ");
            format!("unexpected input at {}; expected one of {}",
                    pos, rule_desc)
        };

        let spaces = iter::repeat(' ').take(pos).collect::<String>();
        let msg = format!("{}\n{}\n{}^", description, input, spaces);

        Err(error::ErrorKind::SyntaxError(msg).into())
    }
}
