use crate::types::{Env, Expr};

pub fn lookup(symbol: &String, env: &Env) -> Option<Expr> {
    if env.defines.contains_key(symbol) {
        Some(env.defines[symbol].clone())
    } else if let Some(outer) = &env.outer {
        lookup(symbol, outer)
    } else {
        None
    }
}

pub fn define(symbol: &String, value: Expr, env: &mut Env) {
    env.defines.insert(symbol.to_string(), value);
}

pub fn expect_flat_list_size(actual: usize, expected: usize) {
    if actual != expected {
        panic!("expected {} arguments, got {}", expected - 1, actual - 1)
    }
}

pub fn quote_next(tree: &Vec<Expr>, i: &mut usize, env: &mut Env) -> Expr {
    *i += 1; tree[*i - 1].clone()
}

pub fn eval_next(tree: &Vec<Expr>, i: &mut usize, env: &mut Env) -> Expr {
    let expr = &tree[*i];
    match expr {
        Expr::Symbol(symbol) => { *i += 1; lookup(&symbol, env).expect(&format!("'{}' is not defined", symbol).to_string()) },
        Expr::FlatList(size) => {
            *i += 1;

            let symbol = tree[*i].clone().expect_symbol();
            *i += 1;

            match symbol.as_str() {
                "define" => {
                    expect_flat_list_size(*size, 3);
                    let name = quote_next(tree, i, env).expect_symbol();
                    let value = eval_next(tree, i, env);
                    define(&name, value, env);
                    return lookup(&String::from("nil"), env).unwrap();
                }
                _ => {}
            }

            let func = lookup(&symbol, env).expect("function not found");

            let mut args = Vec::<Expr>::new();
            for _ in 0..size - 1 {
                args.push(eval_next(tree, i, env));
            }

            match func {
                Expr::NativeFunction(native_func) => {
                    return native_func(args);
                },
                _ => panic!("{} is not callable", symbol)
            }
        },
        _ => {
            *i += 1; expr.clone()
        }
    }
}
