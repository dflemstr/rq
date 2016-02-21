#[cfg_attr(rustfmt, rustfmt_skip)]
mod_path! parser (concat!(env!("OUT_DIR"), "/query/parser.rs"));

#[derive(Debug, Eq, PartialEq)]
pub enum Query {
    Chain(Vec<Query>),
    Function(String, Vec<Expression>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Expression {
    String(String),
}

impl Query {
    pub fn parse(raw: &str) -> Query {
        parser::parse_query(raw).unwrap()
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
}
