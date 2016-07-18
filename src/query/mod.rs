mod context;

mod parser {
    include!(concat!(env!("OUT_DIR"), "/query/parser.rs"));
}

use error;
use value;
pub use self::context::Context;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Query(Vec<Process>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Process(String, Vec<Expression>);

#[derive(Debug)]
pub struct Output<'a, S>
    where S: value::Source
{
    source: S,
    processes: Vec<(&'a Process, context::Process<'a>)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    Value(value::Value),
}

impl Query {
    pub fn parse(raw: &str) -> Query {
        parser::parse_query(raw).unwrap()
    }

    pub fn evaluate<'a, S>(&'a self,
                           context: &'a context::Context,
                           source: S)
                           -> error::Result<Output<'a, S>>
        where S: value::Source
    {
        let mut processes = Vec::with_capacity(self.0.len());

        for def in self.0.iter() {
            processes.push((def, try!(context.process(&def.0))));
        }

        Ok(Output {
            processes: processes,
            source: source,
        })
    }
}

impl<'a, S> Output<'a, S> where S: value::Source {
    fn run_process(&mut self, idx: usize) -> error::Result<Option<value::Value>> {
        // TODO: this is a very procedural thing, maybe make more functional/recursive
        loop {
            if self.processes[idx].1.is_start() {
                let (def, ref mut process) = self.processes[idx];
                try!(process.run_start(&def.1.iter().map(|e| e.to_value()).collect::<Vec<_>>()));
            } else if self.processes[idx].1.is_await() {
                let value = try!(if idx == 0 {
                    self.source.read()
                } else {
                    self.run_process(idx - 1)
                });
                try!(self.processes[idx].1.run_await(value));
            } else if self.processes[idx].1.is_emit() {
                let value = try!(self.processes[idx].1.run_emit());
                return Ok(Some(value))
            } else if self.processes[idx].1.is_end() {
                return Ok(None)
            } else {
                panic!("Process in unknown state: {:?}", self.processes[idx])
            }
        }
    }
}

impl<'a, S> value::Source for Output<'a, S> where S: value::Source {
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        let last_idx = self.processes.len() - 1;
        self.run_process(last_idx)
    }
}

impl Expression {
    fn to_value(&self) -> value::Value {
        match *self {
            Expression::Value(ref v) => v.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use value;

    #[test]
    fn parse_bare_function() {
        let expected = Query(vec![Process("select".to_owned(), vec![])]);
        let actual = Query::parse("select");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_function_one_arg() {
        let expected =
            Query(vec![Process("select".to_owned(),
                               vec![Expression::Value(value::Value::String("a".to_owned()))])]);
        let actual = Query::parse("select a");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_function_one_arg_ident_numbers() {
        let expected =
            Query(vec![Process("select".to_owned(),
                               vec![Expression::Value(value::Value::String("abc123"
                                                                               .to_owned()))])]);
        let actual = Query::parse("select abc123");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_function_one_arg_integer() {
        let expected = Query(vec![Process("select".to_owned(),
                                          vec![Expression::Value(value::Value::I64(52))])]);
        let actual = Query::parse("select 52");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_function_one_arg_negative_integer() {
        let expected = Query(vec![Process("select".to_owned(),
                                          vec![Expression::Value(value::Value::I64(-52))])]);
        let actual = Query::parse("select -52");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_function_one_arg_negative_integer_spaced() {
        let expected = Query(vec![Process("select".to_owned(),
                                          vec![Expression::Value(value::Value::I64(-52))])]);
        let actual = Query::parse("select - 52");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_function_one_arg_underscore() {
        let expected =
            Query(vec![Process("select".to_owned(),
                               vec![Expression::Value(value::Value::String("abc_def"
                                                                               .to_owned()))])]);
        let actual = Query::parse("select abc_def");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_function_one_arg_dash() {
        let expected =
            Query(vec![Process("select".to_owned(),
                               vec![Expression::Value(value::Value::String("abc-def"
                                                                               .to_owned()))])]);
        let actual = Query::parse("select abc-def");

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_function_two_args() {
        let expected = Query(vec![Process("select".to_owned(),
                                          vec![
                Expression::Value(value::Value::String("abc-def".to_owned())),
                Expression::Value(value::Value::String("ghi_123".to_owned())),
            ])]);
        let actual = Query::parse("select abc-def ghi_123");

        assert_eq!(expected, actual);
    }
}
