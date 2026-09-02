use crate::types::{Error, Expr, ExprValue};

fn transform_infix(items: Vec<Expr>) -> Result<Expr, Error> {
    if items.len() < 3 {
        return Err(Error::Custom(
            "Infix expression requires at least 3 elements".to_string(),
        ));
    }

    let default_line = items.first().and_then(|e| e.line);
    let mut iter = items.into_iter();

    let left = iter.next().unwrap();
    let op = iter.next().unwrap();
    let right = iter.next().unwrap();
    let line = op.line.or(left.line).or(default_line);

    let mut result = Expr {
        value: ExprValue::List(vec![op, left, right]),
        line,
    };

    while let Some(op) = iter.next() {
        let right = match iter.next() {
            Some(right) => right,
            None => {
                return Err(Error::Custom(
                    "Infix expression ends with an operator".to_string(),
                ));
            }
        };

        let line = op.line.or(right.line).or(result.line);
        result = Expr {
            value: ExprValue::List(vec![op, result, right]),
            line,
        };
    }

    Ok(result)
}

#[inline]
pub fn is_dot_list(info: &Expr) -> bool {
    match &info.value {
        ExprValue::List(list) => {
            matches!(list.first(), Some(Expr { value: ExprValue::Symbol(s), .. }) if s == ".")
        }
        _ => false,
    }
}

pub fn parse1(tokens: &[(String, usize)], cursor: &mut usize) -> Result<Expr, Error> {
    if *cursor >= tokens.len() {
        return Err(Error::Custom("Unexpected end of input".to_string()));
    }

    let (token_str, line) = &tokens[*cursor];
    *cursor += 1;

    let mut chars = token_str.chars();
    let first_char = match chars.next() {
        Some(c) => c,
        None => return Err(Error::Custom("Empty token".to_string())),
    };
    let line = *line;

    if first_char.is_ascii_digit()
        || (token_str.len() > 1
            && first_char == '-'
            && token_str.as_bytes()[1].is_ascii_digit())
    {
        if token_str.contains('.') {
            match token_str.parse::<f64>() {
                Ok(value) => Ok(Expr {
                    value: ExprValue::Float(value),
                    line: Some(line),
                }),
                Err(_) => Err(Error::FailedToParseNumber {
                    number: token_str.clone(),
                }),
            }
        } else {
            match token_str.parse::<i64>() {
                Ok(value) => Ok(Expr {
                    value: ExprValue::Int(value),
                    line: Some(line),
                }),
                Err(_) => Err(Error::FailedToParseNumber {
                    number: token_str.clone(),
                }),
            }
        }
    } else if token_str == "(" || token_str == "[" || token_str == "{" {
        let is_infix = token_str == "{";
        let closing = match token_str.as_str() {
            "(" => ")",
            "[" => "]",
            "{" => "}",
            _ => unreachable!(),
        };

        let mut items = Vec::<Expr>::new();
        while *cursor < tokens.len() && tokens[*cursor].0 != closing {
            let item = parse1(tokens, cursor)?;
            if is_dot_list(&item) {
                if let ExprValue::List(mut list) = item.value {
                    // Drain and move elements without cloning
                    items.extend(list.drain(1..));
                }
            } else {
                items.push(item);
            }
        }

        if *cursor >= tokens.len() {
            return Err(Error::UnclosedListExpr);
        }

        *cursor += 1; // Consume closing delimiter

        if is_infix {
            return transform_infix(items);
        }

        Ok(Expr {
            value: ExprValue::List(items),
            line: Some(line),
        })
    } else if first_char == '"' {
        let inner = &token_str[1..token_str.len().saturating_sub(1)];
        Ok(Expr {
            value: ExprValue::String(inner.to_string()),
            line: Some(line),
        })
    } else if token_str == "$" {
        let first = Expr {
            value: ExprValue::Symbol(String::from("col")),
            line: Some(line),
        };
        let mut second = parse1(tokens, cursor)?;
        if let ExprValue::Symbol(symbol) = second.value {
            second.value = ExprValue::String(symbol);
            if second.line.is_none() {
                second.line = Some(line);
            }
        }
        Ok(Expr {
            value: ExprValue::List(vec![first, second]),
            line: Some(line),
        })
    } else if token_str == "'" {
        let first = Expr {
            value: ExprValue::Symbol(String::from("quote")),
            line: Some(line),
        };
        let second = parse1(tokens, cursor)?;
        let line_val = second.line.or(Some(line));
        Ok(Expr {
            value: ExprValue::List(vec![first, second]),
            line: line_val,
        })
    } else if token_str == "#" {
        let first = Expr {
            value: ExprValue::Symbol(String::from("lookup")),
            line: Some(line),
        };
        let second = parse1(tokens, cursor)?;
        let line_val = second.line.or(Some(line));
        Ok(Expr {
            value: ExprValue::List(vec![first, second]),
            line: line_val,
        })
    } else if token_str == ")" {
        Err(Error::UnexpectedClosedParen)
    } else {
        Ok(Expr {
            value: ExprValue::Symbol(token_str.clone()),
            line: Some(line),
        })
    }
}

pub fn parse(tokens: &[(String, usize)]) -> Result<Vec<Expr>, Error> {
    let mut cursor = 0;
    let mut exprs = Vec::<Expr>::new();
    while cursor < tokens.len() {
        exprs.push(parse1(tokens, &mut cursor)?);
    }
    Ok(exprs)
}
