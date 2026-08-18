use crate::types::{Env, Expr, Error};

pub struct State {
    pub line: usize
}

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

pub fn eval(expr: Expr, env: &mut Env, state: &mut State) -> Result<Expr, Error> {
    match expr.clone() {
        Expr::SourceInfo(expr, line) => { state.line = line; eval(*expr, env, state) },
        Expr::Symbol(symbol) => match lookup(&symbol, env) {
            Some(value) => Ok(value),
            None => Err(Error::LookupNotFound { symbol }),
        },
        Expr::List(list) => {
            let symbol = list[0].clone().expect_symbol()?;

            match symbol.as_str() {
                "define" => { 
                    let name = list[1].clone().expect_symbol()?;
                    let value = eval(list[2].clone(), env, state)?;
                    define(&name, value, env);
                    return Ok(lookup(&String::from("nil"), env).unwrap());
                },
                "->" => {
                    let mut var = eval(list[1].clone(), env, state)?;
                    for i in 2..list.len() {
                        let mut expr2 = list[i].clone().expect_list()?;
                        expr2.insert(1, var.clone());
                        var = eval(Expr::List(expr2), env, state)?;
                    }
                    return Ok(var);
                }
                _ => {}
            }

            let func = match lookup(&symbol, env) {
                Some(val) => val,
                None => { return Err(Error::LookupNotFound { symbol }) },
            };

            let mut args = Vec::<Expr>::new();
            for j in 1..list.len() {
                args.push(eval(list[j].clone(), env, state)?);
            }

            match func {
                Expr::NativeFunction(native_func) => {
                    return native_func(args);
                },
                _ => Err(Error::WrongType { expected: String::from("callable"), got: func })
            }
        },
        _ => Ok(expr)
    }
}

pub fn eval_string(source: String, env: &mut Env, state: &mut State) -> Result<Expr, Error> {
    let mut tokens = crate::lexer::scan(&source); 
    let exprs = crate::parser::parse(&mut tokens);
    let mut result = None;
    for expr in exprs {
        result = Some(crate::interpreter::eval(expr, env, state));
    }
    result.unwrap()
}
