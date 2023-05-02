#![feature(type_alias_impl_trait)]
#![feature(fn_traits)]

use std::ops::{BitOr, BitXor, BitXorAssign};
use std::rc::Rc;

#[derive(Debug, PartialEq)]
enum ParseError {
    EOF(String),
    Mismatch(String, String),
    Multiple(Vec<ParseError>),
}

struct Parser<A, E>(Box<dyn Fn(&str) -> Result<(A, &str), E>>);

impl<A, E> Parser<A, E> {
    fn run(self, inp: &str) -> Result<(A, &str), E> {
        self.0.call((inp,))
    }
}

impl<A: 'static> BitOr for Parser<A, ParseError> {
    type Output = Parser<A, ParseError>;

    fn bitor(self, rhs: Self) -> Self::Output {
        p_or(self, rhs)
    }
}

impl<A: 'static, B: Copy + 'static> BitXor<B> for Parser<A, ParseError> {
    type Output = Parser<B, ParseError>;

    fn bitxor(self, rhs: B) -> Self::Output {
        p_as(self, rhs)
    }
}

// like >>=
fn bind<A: 'static, B: 'static, E: 'static>(
    a: Parser<A, E>,
    f: fn(A) -> Parser<B, E>,
) -> Parser<B, E> {
    Parser(Box::new(move |inp: &str| {
        let x: Result<(B, &str), E> = match a.0.call((inp,)) {
            Ok((res, rest)) => f(res).run((rest)),
            Err(e) => Err(e),
        };
        return x;
    }))
}

// like *>
fn then<A: 'static, B: 'static, E: 'static>(a: Parser<A, E>, b: Parser<B, E>) -> Parser<B, E> {
    Parser(Box::new(move |inp: &str| {
        let x: Result<(B, &str), E> = match a.0.call((inp,)) {
            Ok((_, rest)) => b.0.call((rest,)),
            Err(e) => Err(e),
        };
        return x;
    }))
}

// like <$>
fn map<A: 'static, B: 'static, E: 'static>(a: Parser<A, E>, f: fn(A) -> B) -> Parser<B, E> {
    Parser(Box::new(move |inp: &str| match a.0.call((inp,)) {
        Ok((r, remaining)) => Ok((f.call((r,)), remaining)),
        Err(e) => Err(e),
    }))
}

// like $>
fn p_as<A: 'static, B: 'static + Copy, E: 'static>(a: Parser<A, E>, b: B) -> Parser<B, E> {
    Parser(Box::new(move |inp: &str| match a.0.call((inp,)) {
        Ok((r, remaining)) => Ok((b, remaining)),
        Err(e) => Err(e),
    }))
}

// primitives

fn p_char(c: char) -> Parser<char, ParseError> {
    Parser(Box::new(move |inp: &str| {
        let mut chars = inp.chars();
        let next = chars.next();
        match next {
            Some(c_) if c_ == c => Ok((c, chars.as_str())),
            Some(wrong) => Err(ParseError::Mismatch(c.to_string(), wrong.to_string())),
            None => Err(ParseError::EOF(c.to_string())),
        }
    }))
}

fn p_str(s: String) -> Parser<String, ParseError> {
    Parser(Box::new(move |inp: &str| {
        match inp.strip_prefix(&s.to_string()) {
            Some(remaining) => Ok((s.to_string(), remaining)),
            None => Err(ParseError::Mismatch(s.to_string(), inp.to_string())),
        }
    }))
}

fn p_or<A: 'static>(
    left: Parser<A, ParseError>,
    right: Parser<A, ParseError>,
) -> Parser<A, ParseError> {
    Parser(Box::new(move |inp: &str| {
        // try left branch
        match left.0.call((inp,)) {
            Ok(a) => Ok(a),
            Err(e) => {
                // try right branch
                match right.0.call((inp,)) {
                    Ok(b) => Ok(b),
                    Err(e2) => Err(ParseError::Multiple(vec![e, e2])),
                }
            }
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheme() {
        #[derive(PartialEq, Debug, Clone, Copy)]
        enum Scheme {
            HTTP,
            HTTPS,
        }
        let scheme = (p_str("https".to_string()) ^ Scheme::HTTPS)
            | (p_str("http".to_string()) ^ Scheme::HTTP);

        assert_eq!(Ok(((Scheme::HTTP), "")), scheme.run("http"));
        assert_eq!(Ok(((Scheme::HTTPS), "")), scheme.run("https"))
    }

    #[test]
    fn it_works() {
        let char = p_or(p_char('c'), p_char('h'));
        assert_eq!(Ok(('h', "ello")), char.run(("hello")));
        assert_eq!(Ok(('c', "ello")), char.run(("cello")));

        let full = then(char, p_str("ello".to_string())).run(("hello"));
        println!("{:?}", full)
    }
}
