use std::{collections::HashMap, ops::Deref};
use std::cell::RefCell;
use std::rc::Rc;

use crate::{corelib, polarslib};
use crate::types::{Env, Expr, ExprValue, Error, UserFunctionData, NativeFunctionData};

pub struct State {
    pub line: usize,
    pub file: Option<String>
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

pub fn set_symbol(symbol: &str, value: Expr, env: &Rc<RefCell<Env>>) -> Result<(), Error> {
    let mut curr = Some(Rc::clone(env));
    while let Some(current_env) = curr {
        let mut borrow = current_env.borrow_mut();
        if borrow.defines.contains_key(symbol) {
            borrow.defines.insert(symbol.to_string(), value);
            return Ok(());
        }
        curr = borrow.outer.as_ref().map(Rc::clone);
    }
    Err(Error::LookupNotFound { symbol: symbol.to_string() })
}

pub fn eval(expr: Expr, env: &Rc<RefCell<Env>>, state: &mut State) -> Result<Expr, Error> {
    if expr.line.is_some() {
        state.line = expr.line.unwrap();
    }

    match expr.value.clone() {
        ExprValue::Symbol(symbol) => {
            if symbol.starts_with(":") {
                Ok(expr)
            } else {
                match lookup(&symbol, env) {
                    Some(value) => Ok(value),
                    None => Err(Error::LookupNotFound { symbol }),
                }
            }
        }
        ExprValue::List(list) => {
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
                        params.push(param.expect_symbol()?);
                    }
                    let body = list.get(2..).unwrap().to_vec();
                
                    let func_data = UserFunctionData {
                        body,
                        params,
                        name: None,
                    };
                
                    let func = ExprValue::UserFunction(Box::new(func_data));
                    return Ok(Expr { value: func, line: None });
                },
                "defun" => {
                    let name = list[1].clone().expect_symbol()?;
                    let mut params = Vec::<String>::new();
                    for param in list[2].clone().expect_list()? {
                        params.push(param.expect_symbol()?);
                    }
                    let body = list.get(3..).unwrap().to_vec();

                    let func_data = UserFunctionData {
                        body,
                        params,
                        name: Some(name.clone()),
                    };

                    let func = Expr {
                        value: ExprValue::UserFunction(Box::new(func_data)),
                        line: None,
                    };

                    define(&name, func.clone(), env);
                    return Ok(func);
                },
                "let" => {
                    if list.len() < 3 {
                        return Err(Error::WrongNumberOfArguments {
                            expected_min: 2,
                            expected_max: u8::MAX,
                            got: (list.len().saturating_sub(1)) as u8,
                        });
                    }

                    let bindings = list[1].clone().expect_list()?;
                    if bindings.len() % 2 != 0 {
                        return Err(Error::WrongType {
                            expected: "an even number of binding pairs (key value ...)".into(),
                            got: list[1].clone(),
                        });
                    }

                    // Create a new child lexical scope
                    let child_env = Rc::new(RefCell::new(Env::with_parent(Rc::clone(env))));

                    // Evaluate values in the outer/current env, then bind them into child_env
                    // (Use child_env for eval if you prefer `let*` semantics where later bindings see earlier ones)
                    for pair in bindings.chunks(2) {
                        let var_name = pair[0].clone().expect_symbol()?;
                        let var_val = eval(pair[1].clone(), env, state)?;
                        define(&var_name, var_val, &child_env);
                    }

                    // Evaluate all body expressions sequentially and return the result of the last one
                    let body = &list[2..];
                    let mut result = corelib::nil();
                    for expr in body {
                        result = eval(expr.clone(), &child_env, state)?;
                    }

                    return Ok(result);
                },
                "->" => {
                    let mut var = eval(list[1].clone(), env, state)?;
                    for i in 2..list.len() {
                        let mut expr2 = list[i].clone().expect_list()?;
                        expr2.insert(1, var.clone());
                        var = eval(Expr { value: ExprValue::List(expr2), line: None }, env, state)?;
                    }
                    return Ok(var);
                },
                "quote" => {
                    return Ok(list[1].clone());
                },
                "if" => {
                    let test = list[1].clone();
                    let on_true = list[2].clone();
                    let on_false = list[3].clone();
                    if eval(test, env, state)?.expect_bool()? {
                        return Ok(eval(on_true, env, state)?);
                    } else {
                        return Ok(eval(on_false, env, state)?);
                    }
                },
                "for" => {
                    if list.len() < 4 {
                        return Err(Error::WrongNumberOfArguments {
                            expected_min: 3,
                            expected_max: u8::MAX,
                            got: (list.len().saturating_sub(1)) as u8,
                        });
                    }

                    let var_name = list[1].clone().expect_symbol()?;
                    let start = eval(list[2].clone(), env, state)?.expect_int()?;
                    let end = eval(list[3].clone(), env, state)?.expect_int()?;

                    let (step, body_start_idx) = if list.len() >= 5 {
                        match &list[4].value {
                            ExprValue::Int(s) => (*s, 5),
                            _ => (if start <= end { 1 } else { -1 }, 4),
                        }
                    } else {
                        (if start <= end { 1 } else { -1 }, 4)
                    };

                    if step == 0 {
                        return Err(Error::Custom("Step cannot be zero in 'for' loop".into()));
                    }

                    let body = &list[body_start_idx..];
                    let child_env = Rc::new(RefCell::new(Env::with_parent(Rc::clone(env))));
                    let mut last_result = corelib::nil();

                    let mut i = start;
                    while (step > 0 && i < end) || (step < 0 && i > end) {
                        child_env.borrow_mut().defines.insert(var_name.clone(), corelib::new_int(i));
                        for expr in body {
                            last_result = eval(expr.clone(), &child_env, state)?;
                        }
                        i += step;
                    }

                    return Ok(last_result);
                },

                "for-each" => {
                    if list.len() < 3 {
                        return Err(Error::WrongNumberOfArguments {
                            expected_min: 2,
                            expected_max: u8::MAX,
                            got: (list.len().saturating_sub(1)) as u8,
                        });
                    }

                    let var_name = list[1].clone().expect_symbol()?;
                    let items = eval(list[2].clone(), env, state)?.expect_list()?;
                    let body = &list[3..];

                    let child_env = Rc::new(RefCell::new(Env::with_parent(Rc::clone(env))));
                    let mut last_result = corelib::nil();

                    for item in items {
                        child_env.borrow_mut().defines.insert(var_name.clone(), item);
                        for expr in body {
                            last_result = eval(expr.clone(), &child_env, state)?;
                        }
                    }

                    return Ok(last_result);
                },
                "set" => {
                    if list.len() != 3 {
                        return Err(Error::WrongNumberOfArguments {
                            expected_min: 2,
                            expected_max: 2,
                            got: (list.len().saturating_sub(1)) as u8,
                        });
                    }
                    let name = list[1].clone().expect_symbol()?;
                    let value = eval(list[2].clone(), env, state)?;
                    set_symbol(&name, value, env)?;
                    return Ok(lookup(&String::from("nil"), env).unwrap());
                },
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

            match func.value {
                ExprValue::NativeFunction(data) => {
                    if args.len() < data.min_args as usize {
                        return Err(Error::WrongNumberOfArguments { expected_min: data.min_args, expected_max: data.max_args, got: args.len() as u8 });
                    }
                    
                    return (data.body)(args);
                },
                ExprValue::UserFunction(data) => {
                    if args.len() < data.params.len() {
                        return Err(Error::WrongNumberOfArguments { 
                            expected_min: data.params.len() as u8, expected_max: data.params.len() as u8, got: args.len() as u8 
                        });
                    }

                    let child_env = Rc::new(RefCell::new(Env::with_parent(Rc::clone(env))));

                    for (param, arg) in data.params.iter().zip(args) {
                        define(param, arg, &child_env);
                    }

                    let mut result = corelib::nil();
                    for expr in data.body {
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

pub fn pretty_wisp(source: String) -> String {
    source.replace("\n)", ")\n")
}

pub fn eval_string(source: String, env: &Rc<RefCell<Env>>, state: &mut State) -> Result<Expr, Error> {
    let s_expr_source = crate::wisp::preprocess_wisp3(&source)?;
    //println!("wisp: {}", s_expr_source.clone());
    //println!("end wisp");
    let mut tokens = crate::lexer::scan(&s_expr_source)?;
    let exprs = crate::parser::parse(&mut tokens)?;
    
    let mut last_res = corelib::nil();
    for expr in exprs {
        // Return immediately if an expression fails
        last_res = crate::interpreter::eval(expr, env, state)?;
    }
    Ok(last_res)
}
