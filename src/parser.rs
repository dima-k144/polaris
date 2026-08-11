use crate::types::Expr;

pub fn parse1(tokens: &mut Vec<(String, usize)>, exprs: &mut Vec<Expr>) {
    let token = tokens.remove(0);
    let mut chars = token.0.chars();
    let first_char = chars.next().unwrap();
    if first_char.is_numeric() {
        exprs.push(Expr::Number(token.0.parse::<f64>().expect("Failed to parse a number")));
    } else if token.0 == "(" {
        let mut size = 0;
        let index = exprs.len();
        while tokens[0].0 != ")" {
            parse1(tokens, exprs);
            size += 1;
        }
        tokens.remove(0);
        exprs.insert(index, Expr::FlatList(size));
    } else {
        exprs.push(Expr::Symbol(token.0));
    }
}

pub fn parse(tokens: &mut Vec<(String, usize)>) -> Vec<Expr> {
    let mut exprs = Vec::<Expr>::new();
    while !tokens.is_empty() {
        parse1(tokens, &mut exprs);
    }
    exprs
}