use crate::types::{Expr, Error};

pub fn parse1(tokens: &mut Vec<(String, usize)>) -> Result<Expr, Error> {
    let token = tokens.remove(0);
    let mut chars = token.0.chars();
    let first_char = chars.next().unwrap();
    let line = token.1;
    if first_char.is_numeric() || (token.0.len() > 1 && first_char == '-' && chars.nth(0).unwrap().is_numeric()) {
        if token.0.contains('.') {
            match token.0.parse::<f64>() {
                Ok(value) => Ok(Expr::SourceInfo(Box::new(Expr::Float(value)), line)),
                Err(err) => Err(Error::FailedToParseNumber { number: token.0 }),
            }
        } else {
            match token.0.parse::<i64>() {
                Ok(value) => Ok(Expr::SourceInfo(Box::new(Expr::Int(value)), line)),
                Err(err) => Err(Error::FailedToParseNumber { number: token.0 }),
            }
        }
    } else if token.0 == "(" {
        let mut items = Vec::<Expr>::new();
        while !tokens.is_empty() && tokens[0].0 != ")" {
            items.push(parse1(tokens)?);
        }

        if tokens.is_empty() {
            return Err(Error::UnclosedListExpr)
        }

        tokens.remove(0); // remove )
        Ok(Expr::List(items))
    } else if first_char == '"' {
        Ok(Expr::String(token.0.chars().skip(1).take(token.0.chars().count() - 2).collect()))
    } else if token.0 == "$" {
        Ok(Expr::List(vec![Expr::Symbol(String::from("col")), parse1(tokens)?]))
    } else {
        Ok(Expr::Symbol(token.0))
    }
}

pub fn parse(tokens: &mut Vec<(String, usize)>) -> Result<Vec<Expr>, Error> {
    let mut exprs = Vec::<Expr>::new();
    while !tokens.is_empty() {
        exprs.push(parse1(tokens)?);
    }
    Ok(exprs)
}