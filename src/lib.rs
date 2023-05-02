#![feature(type_alias_impl_trait)]
#![feature(fn_traits)]


#[derive(Debug)]
#[derive(PartialEq)]
enum ParseError {
    EOF(String),
    Mismatch(String, String),
    Multiple(Vec<ParseError>),
}


type Parser<A, E> = dyn Fn(&str) -> Result<(A, &str), E>;

// like >>=
fn bind<A: 'static, B: 'static, E: 'static>(a: Box<Parser<A, E>>, f: fn(A) -> Box<Parser<B, E>>) -> Box<Parser<B, E>> {
    Box::new(move |inp: &str| {
        let x: Result<(B, &str), E> = match a.call((inp, )) {
            Ok((res, rest)) => f(res).call((rest, )),
            Err(e) => Err(e)
        };
        return x;
    })
}

// like *>
fn then<A: 'static, B: 'static, E: 'static>(a: Box<Parser<A, E>>, b: Box<Parser<B, E>>) -> Box<Parser<B, E>> {
    Box::new(move |inp: &str| {
        let x: Result<(B, &str), E> = match a.call((inp, )) {
            Ok((_, rest)) => b.call((rest, )),
            Err(e) => Err(e)
        };
        return x;
    })
}

// like <$>
fn map<A: 'static, B: 'static, E: 'static>(a: Box<Parser<A, E>>, f: fn(A) -> B) -> Box<Parser<B, E>> {
    Box::new(move |inp: &str| {
        match a.call((inp, )) {
            Ok((r, remaining)) => Ok((f.call((r, )), remaining)),
            Err(e) => Err(e)
        }
    })
}

// like $>
fn p_as<A: 'static, B: 'static + Copy, E: 'static>(a: Box<Parser<A, E>>, b: B) -> Box<Parser<B, E>> {
    Box::new(move |inp: &str| {
        match a.call((inp, )) {
            Ok((r, remaining)) => Ok((b, remaining)),
            Err(e) => Err(e)
        }
    })
}


// primitives

fn p_char(c: char) -> Box<Parser<char, ParseError>> {
    Box::new(move |inp: &str| {
        let mut chars = inp.chars();
        let next = chars.next();
        match next {
            Some(c_) if c_ == c => Ok((c, chars.as_str())),
            Some(wrong) => Err(ParseError::Mismatch(c.to_string(), wrong.to_string())),
            None => Err(ParseError::EOF(c.to_string()))
        }
    })
}

fn p_str(s: String) -> Box<Parser<String, ParseError>> {
    Box::new(move |inp: &str| {
        match inp.strip_prefix(&s.to_string()) {
            Some(remaining) => Ok((s.to_string(), remaining)),
            None => Err(ParseError::Mismatch(s.to_string(), inp.to_string()))
        }
    })
}

fn p_or<A: 'static>(left: Box<Parser<A, ParseError>>, right: Box<Parser<A, ParseError>>) -> Box<Parser<A, ParseError>> {
    Box::new(move |inp: &str| {
        // try left branch
        match left.call((inp, )) {
            Ok(a) => Ok(a),
            Err(e) => {
                // try right branch
                match right.call((inp, )) {
                    Ok(b) => Ok(b),
                    Err(e2) => Err(ParseError::Multiple(vec![e, e2]))
                }
            }
        }
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheme() {
        #[derive(PartialEq)]
        #[derive(Debug)]
        #[derive(Clone, Copy)]
        enum Scheme {
            HTTP,
            HTTPS,
        }
        let scheme =
            p_or(
                p_as(p_str("https".to_string()), Scheme::HTTPS),
                p_as(p_str("http".to_string()), Scheme::HTTP),
            );

        assert_eq!(Ok(((Scheme::HTTP), "")), scheme.call(("http", )));
        assert_eq!(Ok(((Scheme::HTTPS), "")), scheme.call(("https", )))
    }


    #[test]
    fn it_works() {
        let char = p_or(
            p_char('c'), p_char('h'),
        );
        assert_eq!(Ok(('h', "ello")), char.call(("hello", )));
        assert_eq!(Ok(('c', "ello")), char.call(("cello", )));

        let full = then(char, p_str("ello".to_string()))
            .call(("hello", ));
        println!("{:?}", full)
    }
}
