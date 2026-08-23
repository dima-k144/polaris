use crate::types::{Expr, Error};

pub fn parse1(tokens: &mut Vec<(String, usize)>) -> Result<Expr, Error> {
    if tokens.is_empty() {
        return Err(Error::Custom("Unexpected end of input".to_string()));
    }

    let token = tokens.remove(0);
    let mut chars = token.0.chars();
    let first_char = match chars.next() {
        Some(c) => c,
        None => return Err(Error::Custom("Empty token".to_string())),
    };
    let line = token.1;

    if first_char.is_ascii_digit() || (token.0.len() > 1 && first_char == '-' && token.0.chars().nth(1).unwrap().is_ascii_digit()) {
        if token.0.contains('.') {
            match token.0.parse::<f64>() {
                Ok(value) => Ok(Expr::SourceInfo(Box::new(Expr::Float(value)), line)),
                Err(_) => Err(Error::FailedToParseNumber { number: token.0 }),
            }
        } else {
            match token.0.parse::<i64>() {
                Ok(value) => Ok(Expr::SourceInfo(Box::new(Expr::Int(value)), line)),
                Err(_) => Err(Error::FailedToParseNumber { number: token.0 }),
            }
        }
    } else if token.0 == "(" || token.0 == "[" || token.0 == "{" {
        let closing = match token.0.as_str() {
            "(" => ")",
            "[" => "]",
            "{" => "}",
            _ => unreachable!(),
        };

        let mut items = Vec::<Expr>::new();
        while !tokens.is_empty() && tokens[0].0 != closing {
            items.push(parse1(tokens)?);
        }

        if tokens.is_empty() {
            return Err(Error::UnclosedListExpr);
        }

        tokens.remove(0); // consume closing paren/bracket/brace
        Ok(Expr::List(items))
    } else if first_char == '"' {
        let inner = &token.0[1..token.0.len().saturating_sub(1)];
        Ok(Expr::String(inner.to_string()))
    } else if token.0 == "$" {
        Ok(Expr::List(vec![Expr::Symbol(String::from("col")), parse1(tokens)?]))
    } else if token.0 == "'" {
        Ok(Expr::List(vec![Expr::Symbol(String::from("quote")), parse1(tokens)?]))
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