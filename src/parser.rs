use crate::types::{Expr, ExprValue, Error};

fn transform_infix(items: Vec<Expr>) -> Result<Expr, Error> {
    if items.len() < 3 {
        return Err(Error::Custom(
            "Infix expression requires at least 3 elements".to_string()
        ));
    }

    // Version 1:
    // {a + b} -> (+ a b)

    if items.len() == 3 {
        let mut iter = items.into_iter();

        let left = iter.next().unwrap();
        let op = iter.next().unwrap();
        let right = iter.next().unwrap();

        return Ok(Expr {
            value: ExprValue::List(vec![op, left, right]),
            line: None,
        });
    }

    // Version 2:
    // {a + b + c} -> (+ (+ a b) c)

    let mut iter = items.into_iter();

    let mut result = {
        let left = iter.next().unwrap();
        let op = iter.next().unwrap();
        let right = iter.next().unwrap();

        Expr {
            value: ExprValue::List(vec![op, left, right]),
            line: None,
        }
    };

    loop {
        let op = match iter.next() {
            Some(op) => op,
            None => break,
        };

        let right = match iter.next() {
            Some(right) => right,
            None => {
                return Err(Error::Custom(
                    "Infix expression ends with an operator".to_string()
                ));
            }
        };

        result = Expr {
            value: ExprValue::List(vec![op, result, right]),
            line: None,
        };
    }

    Ok(result)
}

pub fn is_dot_list(info: &Expr) -> bool {
    match info.value.clone() {
        ExprValue::List(list) => {
            if list.is_empty() { return false; }
            match &list[0].value {
                ExprValue::Symbol(symbol) => {
                    if symbol == "." {
                        true
                    } else {
                        false
                    }
                },
                _ => false
            }
        },
        _ => false
    }
}

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
                Ok(value) => Ok(Expr{value: ExprValue::Float(value), line: Some(line)}),
                Err(_) => Err(Error::FailedToParseNumber { number: token.0 }),
            }
        } else {
            match token.0.parse::<i64>() {
                Ok(value) => Ok(Expr{value: ExprValue::Int(value), line: Some(line)}),
                Err(_) => Err(Error::FailedToParseNumber { number: token.0 }),
            }
        }
    } else if token.0 == "(" || token.0 == "[" || token.0 == "{" {
        let is_infix = token.0 == "{";

        let closing = match token.0.as_str() {
            "(" => ")",
            "[" => "]",
            "{" => "}",
            _ => unreachable!(),
        };

        let mut items = Vec::<Expr>::new();
        while !tokens.is_empty() && tokens[0].0 != closing {
            let item = parse1(tokens)?;
            if is_dot_list(&item) {
                for x in item.expect_list()?.iter().skip(1) {
                    items.push(x.clone());
                }
            } else {
                items.push(item);
            }
        }

        if tokens.is_empty() {
            return Err(Error::UnclosedListExpr);
        }

        tokens.remove(0); // consume closing paren/bracket/brace

        if is_infix {
            return transform_infix(items);
        }

        Ok(Expr { value: ExprValue::List(items), line: Some(line) })
    } else if first_char == '"' {
        let inner = &token.0[1..token.0.len().saturating_sub(1)];
        Ok(Expr { value: ExprValue::String(inner.to_string()), line: Some(line) })
    } else if token.0 == "$" {
        let first = Expr { value: ExprValue::Symbol(String::from("col")), line: Some(line) };
        let mut second = parse1(tokens)?;
        match second.value {
            ExprValue::Symbol(symbol) => { second.value = ExprValue::String(symbol) }
            _ => {}
        }
        Ok(Expr { value: ExprValue::List(vec![first, second]), line: Some(line) })
    } else if token.0 == "'" {
        let first = Expr { value: ExprValue::Symbol(String::from("quote")), line: Some(line) };
        let second = parse1(tokens)?;
        Ok(Expr { value: ExprValue::List(vec![first, second]), line: Some(line) })
    } else {
        Ok(Expr { value: ExprValue::Symbol(token.0), line: Some(line) })
    }
}

pub fn parse(tokens: &mut Vec<(String, usize)>) -> Result<Vec<Expr>, Error> {
    let mut exprs = Vec::<Expr>::new();
    while !tokens.is_empty() {
        exprs.push(parse1(tokens)?);
    }
    Ok(exprs)
}