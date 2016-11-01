mod context;
mod parser;

use error;

pub use self::context::Context;
use value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Query(Vec<Process>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Process(String, Vec<Expression>);

#[derive(Debug)]
struct RunningProcess<'a>(&'a Process, context::Process<'a>);

#[derive(Debug)]
pub struct Output<'a, S>
    where S: value::Source
{
    source: S,
    processes: Vec<Option<RunningProcess<'a>>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    Value(value::Value),
    Function(Vec<String>, String),
}

impl Query {
    pub fn empty() -> Query {
        Query(Vec::new())
    }

    pub fn parse(raw: &str) -> error::Result<Query> {
        parser::parse_query(raw)
    }

    pub fn evaluate<'a, S>(&'a self,
                           context: &'a context::Context,
                           source: S)
                           -> error::Result<Output<'a, S>>
        where S: value::Source
    {
        let mut processes = Vec::with_capacity(self.0.len());

        for def in self.0.iter() {
            processes.push(Some(RunningProcess(def, try!(context.process(&def.0)))));
        }

        Ok(Output {
            processes: processes,
            source: source,
        })
    }
}

impl<'a, S> Output<'a, S>
    where S: value::Source
{
    fn run_process_idx(&mut self, idx: usize) -> error::Result<Option<value::Value>> {
        match self.processes[idx].take() {
            None => Ok(None),
            Some(mut process) => {
                let result = try!(self.run_process(idx, &mut process));
                self.processes[idx] = Some(process);
                Ok(result)
            },
        }
    }

    fn run_process(&mut self,
                   idx: usize,
                   process: &mut RunningProcess<'a>)
                   -> error::Result<Option<value::Value>> {
        let RunningProcess(ref decl, ref mut instance) = *process;
        loop {
            instance.context().run_enqueued_tasks();

            match instance.state.resume() {
                context::Resume::Start(s) => {
                    trace!("Process moving out of start: {} {:?}", idx, decl);
                    instance.state = try!(s.run(&instance, &decl.1));
                    trace!("Process moved out of start: {} {:?}", idx, decl);
                },
                context::Resume::Pending(p) => {
                    trace!("Process moving out of pending: {} {:?}", idx, decl);
                    instance.state = try!(p.run(&instance));
                    trace!("Process moved out of pending: {} {:?}", idx, decl);
                },
                context::Resume::Await(a) => {
                    let value = try!(if idx == 0 {
                        self.source.read()
                    } else {
                        self.run_process_idx(idx - 1)
                    });
                    trace!("Process moving out of await: {} {:?}", idx, decl);
                    instance.state = try!(a.run(&instance, value));
                    trace!("Process moved out of await: {} {:?}", idx, decl);
                },
                context::Resume::Emit(e) => {
                    trace!("Process moving out of emit: {} {:?}", idx, decl);
                    let (state, value) = try!(e.run());
                    instance.state = state;
                    trace!("Process moved out of emit: {} {:?}", idx, decl);
                    return Ok(Some(value));
                },
                context::Resume::End => {
                    trace!("Process ended: {} {:?}", idx, decl);
                    return Ok(None);
                },
            }
        }
    }
}

impl<'a, S> value::Source for Output<'a, S>
    where S: value::Source
{
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        if self.processes.is_empty() {
            self.source.read()
        } else {
            let last_idx = self.processes.len() - 1;
            self.run_process_idx(last_idx)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use value;

    #[test]
    fn parse_kitchen_sink() {
        let expected =
            Query(vec![Process("dostuff".to_owned(),
                               vec![Expression::Value(value::Value::String("foo".to_owned())),
                                    Expression::Function(vec!["x".to_owned()],
                                                         "x+3".to_owned()),
                                    Expression::Function(vec!["a".to_owned(),
                                                              "b".to_owned(),
                                                              "c".to_owned()],
                                                         "a + b - c".to_owned())]),
                       Process("other".to_owned(),
                               vec![Expression::Value(value::Value::String("xyz".to_owned())),
                                    Expression::Value(value::Value::from_f64(2.0))]),
                       Process("bar".to_owned(), vec![])]);
        let actual = Query::parse("dostuff foo (x)=>{x+3} (a, b, c) => {a + b - c} | other xyz 2 \
                                   | bar")
            .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_bare_process() {
        let expected = Query(vec![Process("select".to_owned(), vec![])]);
        let actual = Query::parse("select").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_two_processes() {
        let expected = Query(vec![Process("select".to_owned(), vec![]),
                                  Process("id".to_owned(), vec![])]);
        let actual = Query::parse("select|id").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_process_one_arg() {
        let expected = Query(vec![Process("select".to_owned(),
                               vec![Expression::Value(value::Value::String("a".to_owned()))])]);
        let actual = Query::parse("select a").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_process_one_arg_ident_numbers() {
        let expected = Query(vec![Process("select".to_owned(),
                                          vec![Expression::Value(value::Value::String("abc123"
                                                   .to_owned()))])]);
        let actual = Query::parse("select abc123").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_process_one_arg_integer() {
        let expected = Query(vec![Process("select".to_owned(),
                                          vec![Expression::Value(value::Value::from_f64(52.0))])]);
        let actual = Query::parse("select 52").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_process_one_arg_negative_integer() {
        let expected = Query(vec![Process("select".to_owned(),
                                          vec![Expression::Value(value::Value::from_f64(-52.0))])]);
        let actual = Query::parse("select -52").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_process_one_arg_underscore() {
        let expected = Query(vec![Process("select".to_owned(),
                                          vec![Expression::Value(value::Value::String("abc_def"
                                                   .to_owned()))])]);
        let actual = Query::parse("select abc_def").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_process_one_arg_dash() {
        let expected = Query(vec![Process("select".to_owned(),
                                          vec![Expression::Value(value::Value::String("abc-def"
                                                   .to_owned()))])]);
        let actual = Query::parse("select abc-def").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_process_two_args() {
        let expected = Query(vec![Process("select".to_owned(),
                                          vec![
                Expression::Value(value::Value::String("abc-def".to_owned())),
                Expression::Value(value::Value::String("ghi_123".to_owned())),
            ])]);
        let actual = Query::parse("select abc-def ghi_123").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_process_function_arg() {
        let expected = Query(vec![Process("map".to_owned(),
                                          vec![Expression::Function(vec!["x".to_owned()],
                                                                    "2 + x".to_owned())])]);
        let actual = Query::parse("map (x)=>{2 + x}").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_process_function_arg_two_named_params() {
        let expected = Query(vec![Process("map".to_owned(),
                                          vec![Expression::Function(vec!["a".to_owned(),
                                                                         "b".to_owned()],
                                                                    "a + b".to_owned())])]);
        let actual = Query::parse("map (a, b) => {a + b}").unwrap();

        assert_eq!(expected, actual);
    }
}
