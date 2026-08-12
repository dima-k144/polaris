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

pub fn eval_next(tree: &Vec<Expr>, i: &mut usize, env: &mut Env) -> Expr {
    let expr = &tree[*i];
    match expr {
        Expr::Symbol(symbol) => { *i += 1; lookup(&symbol, env).expect(&format!("'{}' is not defined", symbol).to_string()) },
        Expr::FlatList(size) => {
            let symbol = tree[*i + 1].clone().expect_symbol();
            let func = lookup(&symbol, env).expect("function not found");
            *i += 2;

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
