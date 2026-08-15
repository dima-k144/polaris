use crate::types::Expr;

pub fn parse1(tokens: &mut Vec<(String, usize)>) -> Expr {
    let token = tokens.remove(0);
    let mut chars = token.0.chars();
    let first_char = chars.next().unwrap();
    if first_char.is_numeric() {
        Expr::Number(token.0.parse::<f64>().expect("Failed to parse a number"))
    } else if token.0 == "(" {
        let mut items = Vec::<Expr>::new();
        while tokens[0].0 != ")" {
            items.push(parse1(tokens));
        }
        tokens.remove(0);
        Expr::List(items)
    } else if first_char == '"' {
        Expr::String(token.0.chars().skip(1).take(token.0.chars().count() - 2).collect())
    } else {
        Expr::Symbol(token.0)
    }
}

pub fn parse(tokens: &mut Vec<(String, usize)>) -> Vec<Expr> {
    let mut exprs = Vec::<Expr>::new();
    while !tokens.is_empty() {
        exprs.push(parse1(tokens));
    }
    exprs
}