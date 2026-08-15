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

pub fn eval(expr: Expr, env: &mut Env) -> Expr {
    match expr.clone() {
        Expr::Symbol(symbol) => lookup(&symbol, env).expect(&format!("'{}' is not defined", symbol).to_string()),
        Expr::List(list) => {
            let symbol = list[0].clone().expect_symbol();
            match symbol.as_str() {
                "define" => { 
                    let name = list[1].clone().expect_symbol();
                    let value = eval(list[2].clone(), env);
                    define(&name, value, env);
                    return lookup(&String::from("nil"), env).unwrap();
                }
                _ => {}
            }

            let func = lookup(&symbol, env).expect("function not found");

            let mut args = Vec::<Expr>::new();
            for j in 1..list.len() {
                args.push(eval(list[j].clone(), env));
            }

            match func {
                Expr::NativeFunction(native_func) => {
                    return native_func(args);
                },
                _ => panic!("{} is not callable", symbol)
            }
        },
        _ => expr
    }
}
