use nom;

#[derive(Debug, Eq, PartialEq)]
pub enum Query {
    Chain {
        elements: Vec<Query>,
    },
    Function {
        name: String,
        args: Vec<Expression>,
    },
}

impl Query {
    pub fn parse(raw: &str) -> Query {
        match query(raw) {
            nom::IResult::Done("", r) => r,
            e => panic!("{:?}", e),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Expression {
    String(String),
}

fn is_space(c: char) -> bool {
    c.is_whitespace()
}

fn is_alpha(c: char) -> bool {
    c.is_alphabetic()
}

named!(space<&str, &str>, take_while_s!(is_space));
named!(alpha<&str, &str>, take_while_s!(is_alpha));

named!(string_lit<&str, String>,
       chain!(tag_s!("a"),
              ||{ "a".to_owned() }));

named!(expr<&str, Expression>,
       alt!(map!(string_lit, Expression::String)));

named!(args<&str, Vec<Expression> >, many0!(expr));

named!(query<&str, Query>,
       chain!(name: alpha ~ space ~ args: args,
              ||{ Query::Function { name: name.to_owned(), args: args } }));

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn function() {
        let q = Query::parse("select a");

        assert_eq!(q,
                   Query::Function {
                       name: "select".to_owned(),
                       args: vec![Expression::String("a".to_owned())],
                   });
    }
}
