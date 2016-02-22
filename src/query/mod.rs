pub mod context;
#[cfg_attr(rustfmt, rustfmt_skip)]
mod_path! parser (concat!(env!("OUT_DIR"), "/query/parser.rs"));

use value;
pub use self::context::Context;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Query {
    Chain(Vec<Query>),
    Function(String, Vec<Expression>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    String(String),
    Integer(i64),
}

impl Query {
    pub fn parse(raw: &str) -> Query {
        parser::parse_query(raw).unwrap()
    }

    pub fn evaluate(&self, context: &context::Context, input: value::Value) -> value::Value {
        match *self {
            Query::Chain(ref queries) => apply_chain(context, input, queries),
            Query::Function(ref name, ref args) => apply_function(context, input, name, args),
        }
    }
}

impl Expression {
    fn to_value(&self) -> value::Value {
        match *self {
            Expression::String(ref s) => value::Value::String(s.clone()),
            Expression::Integer(i) => value::Value::I64(i),
        }
    }
}

fn apply_chain(context: &context::Context, input: value::Value, queries: &[Query]) -> value::Value{
    let mut result = input;

    for query in queries {
        result = query.evaluate(context, result);
    }

    result
}

fn apply_function(context: &context::Context, input: value::Value, name: &str, args: &[Expression]) -> value::Value {
    match context.function(name) {
        Some(func) => {
            let mut vals = Vec::with_capacity(args.len() + 1);
            vals.push(input);
            for arg in args {
                vals.push(arg.to_value())
            }
            func(&vals)
        },
        None => value::Value::Unit,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_bare_function() {
        let expected = Query::Function("select".to_owned(), vec![]);
        let actual = Query::parse("select");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_function_one_arg() {
        let expected =
            Query::Function("select".to_owned(),
                            vec![Expression::String("a".to_owned())]);
        let actual = Query::parse("select a");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_function_one_arg_ident_numbers() {
        let expected =
            Query::Function("select".to_owned(),
                            vec![Expression::String("abc123".to_owned())]);
        let actual = Query::parse("select abc123");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_function_one_arg_integer() {
        let expected =
            Query::Function("select".to_owned(),
                            vec![Expression::Integer(52)]);
        let actual = Query::parse("select 52");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_function_one_arg_negative_integer() {
        let expected =
            Query::Function("select".to_owned(),
                            vec![Expression::Integer(-52)]);
        let actual = Query::parse("select -52");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_function_one_arg_negative_integer_spaced() {
        let expected =
            Query::Function("select".to_owned(),
                            vec![Expression::Integer(-52)]);
        let actual = Query::parse("select - 52");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_function_one_arg_underscore() {
        let expected =
            Query::Function("select".to_owned(),
                            vec![Expression::String("abc_def".to_owned())]);
        let actual = Query::parse("select abc_def");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_function_one_arg_dash() {
        let expected =
            Query::Function("select".to_owned(),
                            vec![Expression::String("abc-def".to_owned())]);
        let actual = Query::parse("select abc-def");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_function_two_args() {
        let expected =
            Query::Function("select".to_owned(), vec![
                Expression::String("abc-def".to_owned()),
                Expression::String("ghi_123".to_owned()),
            ]);
        let actual = Query::parse("select abc-def ghi_123");

        assert_eq!(expected, actual);
    }
}
