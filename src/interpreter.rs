use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::types::{Env, Expr, Error};

pub struct State {
    pub line: usize
}

pub fn lookup(symbol: &str, env: &Rc<RefCell<Env>>) -> Option<Expr> {
    let env_borrow = env.borrow();
    if let Some(val) = env_borrow.defines.get(symbol) {
        Some(val.clone())
    } else if let Some(outer) = &env_borrow.outer {
        lookup(symbol, outer)
    } else {
        None
    }
}

pub fn define(symbol: &String, value: Expr, env: &Rc<RefCell<Env>>) {
    env.borrow_mut().defines.insert(symbol.to_string(), value);
}

pub fn eval(expr: Expr, env: &Rc<RefCell<Env>>, state: &mut State) -> Result<Expr, Error> {
    match expr.clone() {
        Expr::SourceInfo(expr, line) => { state.line = line; eval(*expr, env, state) },
        Expr::Symbol(symbol) => {
            if symbol.starts_with(":") {
                Ok(expr)
            } else {
                match lookup(&symbol, env) {
                    Some(value) => Ok(value),
                    None => Err(Error::LookupNotFound { symbol }),
                }
            }
        }
        Expr::List(list) => {
            let symbol = list[0].clone().expect_symbol()?;

            match symbol.as_str() {
                "define" => { 
                    let name = list[1].clone().expect_symbol()?;
                    let value = eval(list[2].clone(), env, state)?;
                    define(&name, value, env);
                    return Ok(lookup(&String::from("nil"), env).unwrap());
                },
                "lambda" => {
                    let mut params = Vec::<String>::new();
                    for param in list[1].clone().expect_list()? {
                        params.push(param.expect_symbol()?)
                    }
                    let body = list.get(2..).unwrap().to_vec();
                    let func = Expr::UserFunction { body: body, params: params, name: None };
                    return Ok(func)
                },
                "defun" => {
                    let name = list[1].clone().expect_symbol()?;
                    let mut params = Vec::<String>::new();
                    for param in list[2].clone().expect_list()? {
                        params.push(param.expect_symbol()?)
                    }
                    let body = list.get(3..).unwrap().to_vec();
                    let func = Expr::UserFunction { body: body, params: params, name: Some(name.clone()) };
                    define(&name.clone(), func.clone(), env);
                    return Ok(func)
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
                Expr::NativeFunction{ body, min_args, max_args } => {
                    if args.len() < min_args as usize {
                        return Err(Error::WrongNumberOfArguments { expected_min: min_args, expected_max: max_args, got: args.len() as u8 });
                    }
                    
                    return body(args);
                },
                Expr::UserFunction { body, params, name } => {
                    if args.len() < params.len() {
                        return Err(Error::WrongNumberOfArguments { 
                            expected_min: params.len() as u8, expected_max: params.len() as u8, got: args.len() as u8 
                        });
                    }

                    let child_env = Rc::new(RefCell::new(Env::with_parent(Rc::clone(env))));

                    for (param, arg) in params.iter().zip(args) {
                        define(param, arg, &child_env);
                    }

                    let mut result = Expr::Symbol("nil".into());
                    for expr in body {
                        result = eval(expr, &child_env, state)?;
                    }
                    return Ok(result);
                },
                _ => Err(Error::WrongType { expected: String::from("callable"), got: func })
            }
        },
        _ => Ok(expr)
    }
}

pub fn eval_string(source: String, env: &Rc<RefCell<Env>>, state: &mut State) -> Result<Expr, Error> {
    let mut tokens = crate::lexer::scan(&source); 
    let exprs = crate::parser::parse(&mut tokens);
    let mut result = None;
    for expr in exprs {
        result = Some(crate::interpreter::eval(expr, env, state));
    }
    result.unwrap()
}
