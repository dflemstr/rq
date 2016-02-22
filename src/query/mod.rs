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

    #[test]
    fn parse_function_one_arg_numbers() {
        let expected =
            Query::Function("select".to_owned(),
                            vec![Expression::String("abc123".to_owned())]);
        let actual = Query::parse("select abc123");

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
