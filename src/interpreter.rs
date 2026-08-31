use std::cell::RefCell;
use std::rc::Rc;

use crate::types::{Env, Error, Expr, ExprValue, UserFunctionData};
use crate::{corelib, polarslib};

pub struct State {
    pub line: usize,
    pub file: Option<String>,
}

pub fn invoke_function(
    func: &Expr,
    args: Vec<Expr>,
    env: &Rc<RefCell<Env>>,
    state: &mut State,
) -> Result<Expr, Error> {
    match &func.value {
        ExprValue::NativeFunction(data) => {
            expect_args_count(args.len(), data.min_args as usize, data.max_args as usize)?;
            (data.body)(args, env)
        }
        ExprValue::UserFunction(data) => {
            expect_args_count(args.len(), data.params.len(), usize::MAX)?;
            let child_env = Rc::new(RefCell::new(Env::with_parent(Rc::clone(env))));

            for (param, arg) in data.params.iter().zip(args) {
                define(param, arg, &child_env);
            }

            let mut result = corelib::nil();
            for expr in &data.body {
                result = eval(expr.clone(), &child_env, state)?;
            }
            Ok(result)
        }
        _ => Err(Error::WrongType {
            expected: "callable (user function or native function)".into(),
            got: func.clone(),
        }),
    }
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
    Err(Error::LookupNotFound {
        symbol: symbol.to_string(),
    })
}

pub fn expect_args_count(got: usize, min: usize, max: usize) -> Result<(), Error> {
    if got < min || got > max {
        Err(Error::WrongNumberOfArguments {
            expected_min: min as u8,
            expected_max: max as u8,
            got: got as u8,
        })
    } else {
        Ok(())
    }
}

pub fn eval(expr: Expr, env: &Rc<RefCell<Env>>, state: &mut State) -> Result<Expr, Error> {
    if let Some(line) = expr.line {
        state.line = line
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
            if list.is_empty() {
                return Err(Error::EmptyListEval);
            }

            let symbol = list[0].clone().expect_symbol()?;

            match symbol.as_str() {
                "define" => {
                    expect_args_count(list.len() - 1, 2, 2)?;
                    let name = list[1].clone().expect_symbol()?;
                    let value = eval(list[2].clone(), env, state)?;
                    define(&name, value, env);
                    return Ok(corelib::nil());
                }
                "lambda" => {
                    expect_args_count(list.len() - 1, 2, usize::MAX)?;
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
                    return Ok(Expr {
                        value: func,
                        line: expr.line, // <-- Preserve line number from lambda token
                    });
                }
                "defun" => {
                    expect_args_count(list.len() - 1, 3, usize::MAX)?;
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
                        line: expr.line, // <-- Preserve line number from defun token
                    };

                    define(&name, func.clone(), env);
                    return Ok(func);
                }
                "let" => {
                    expect_args_count(list.len() - 1, 2, usize::MAX)?;

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
                }
                "->" => {
                    expect_args_count(list.len() - 1, 2, usize::MAX)?;
                    let mut var = eval(list[1].clone(), env, state)?;
                    for i in 2..list.len() {
                        let step_expr = &list[i];
                        let step_line = step_expr.line.or(expr.line); // Preserve line info
                        let mut expr2 = step_expr.clone().expect_list()?;
                        expr2.insert(1, var.clone());
                        var = eval(
                            Expr {
                                value: ExprValue::List(expr2),
                                line: step_line, // <-- Pass the pipeline step's line number
                            },
                            env,
                            state,
                        )?;
                    }
                    return Ok(var);
                }
                "quote" => {
                    expect_args_count(list.len() - 1, 1, 1)?;
                    return Ok(list[1].clone());
                }
                "if" => {
                    expect_args_count(list.len() - 1, 3, 3)?;
                    let test = list[1].clone();
                    let on_true = list[2].clone();
                    let on_false = list[3].clone();
                    if eval(test, env, state)?.expect_bool()? {
                        return eval(on_true, env, state);
                    } else {
                        return eval(on_false, env, state);
                    }
                }
                "for" => {
                    expect_args_count(list.len() - 1, 3, usize::MAX)?;

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
                        child_env
                            .borrow_mut()
                            .defines
                            .insert(var_name.clone(), corelib::new_int(i));
                        for expr in body {
                            last_result = eval(expr.clone(), &child_env, state)?;
                        }
                        i += step;
                    }

                    return Ok(last_result);
                }

                "for-each" => {
                    expect_args_count(list.len() - 1, 2, usize::MAX)?;

                    let var_name = list[1].clone().expect_symbol()?;
                    let items = eval(list[2].clone(), env, state)?.expect_list()?;
                    let body = &list[3..];

                    let child_env = Rc::new(RefCell::new(Env::with_parent(Rc::clone(env))));
                    let mut last_result = corelib::nil();

                    for item in items {
                        child_env
                            .borrow_mut()
                            .defines
                            .insert(var_name.clone(), item);
                        for expr in body {
                            last_result = eval(expr.clone(), &child_env, state)?;
                        }
                    }

                    return Ok(last_result);
                }
                "set" => {
                    expect_args_count(list.len() - 1, 2, 2)?;
                    let name = list[1].clone().expect_symbol()?;
                    let value = eval(list[2].clone(), env, state)?;
                    set_symbol(&name, value, env)?;
                    return Ok(corelib::nil());
                }
                "lookup" => {
                    expect_args_count(list.len() - 1, 1, 1)?;
                    let symbol = list[1].clone().expect_symbol()?;
                    return match lookup(&symbol, env) {
                        Some(value) => Ok(value),
                        None => Err(Error::LookupNotFound { symbol }),
                    };
                }
                "and" => {
                    let args = &list[1..];
                    if args.is_empty() {
                        return Ok(corelib::true_());
                    }

                    // 1. Polars context: if any unevaluated arg or function call represents Polars,
                    // or if the first arg evaluates to a PolarsExpr, fall back to building the Polars expression.
                    let first_val = eval(args[0].clone(), env, state)?;
                    if matches!(first_val.value, ExprValue::PolarsExpr(_)) {
                        let mut polars_args = vec![first_val];
                        for arg in &args[1..] {
                            polars_args.push(eval(arg.clone(), env, state)?);
                        }
                        return polarslib::and_polars(polars_args, env);
                    }

                    // 2. Standard Lisp Short-Circuiting
                    if !first_val.expect_bool()? {
                        return Ok(corelib::false_());
                    }

                    for arg in &args[1..] {
                        let val = eval(arg.clone(), env, state)?;
                        if matches!(val.value, ExprValue::PolarsExpr(_)) {
                            // If Polars expression appears later in the args
                            let polars_args = vec![corelib::new_bool(true), val];
                            return polarslib::and_polars(polars_args, env);
                        }
                        if !val.expect_bool()? {
                            return Ok(corelib::false_());
                        }
                    }

                    return Ok(corelib::true_());
                }
                "or" => {
                    let args = &list[1..];
                    if args.is_empty() {
                        return Ok(corelib::false_());
                    }

                    let first_val = eval(args[0].clone(), env, state)?;
                    if matches!(first_val.value, ExprValue::PolarsExpr(_)) {
                        let mut polars_args = vec![first_val];
                        for arg in &args[1..] {
                            polars_args.push(eval(arg.clone(), env, state)?);
                        }
                        return polarslib::or_polars(polars_args, env);
                    }

                    if first_val.expect_bool()? {
                        return Ok(corelib::true_());
                    }

                    for arg in &args[1..] {
                        let val = eval(arg.clone(), env, state)?;
                        if matches!(val.value, ExprValue::PolarsExpr(_)) {
                            let polars_args = vec![corelib::new_bool(false), val];
                            return polarslib::or_polars(polars_args, env);
                        }
                        if val.expect_bool()? {
                            return Ok(corelib::true_());
                        }
                    }

                    return Ok(corelib::false_());
                }
                "load" => {
                    expect_args_count(list.len() - 1, 1, 1)?;
                    let path_expr = eval(list[1].clone(), env, state)?;
                    let path = path_expr.expect_string()?;

                    let contents = std::fs::read_to_string(&path).map_err(|err| {
                        Error::Custom(format!("Failed to load file '{}': {}", path, err))
                    })?;

                    // Track the loaded file's path for error reporting
                    let mut file_state = State {
                        line: 0,
                        file: Some(path),
                    };

                    return eval_string(contents, env, &mut file_state);
                }
                "apply-udf" => {
                    return crate::polarslib::eval_apply_udf(&list[1..], env, state);
                }
                _ => {}
            }

            let func = match lookup(&symbol, env) {
                Some(val) => val,
                None => return Err(Error::LookupNotFound { symbol }),
            };

            let mut args = Vec::<Expr>::new();
            for j in 1..list.len() {
                args.push(eval(list[j].clone(), env, state)?);
            }

            match func.value {
                ExprValue::NativeFunction(data) => {
                    expect_args_count(
                        list.len() - 1,
                        data.min_args as usize,
                        data.max_args as usize,
                    )?;

                    (data.body)(args, env)
                }
                ExprValue::UserFunction(data) => {
                    expect_args_count(list.len() - 1, data.params.len(), usize::MAX)?;

                    let child_env = Rc::new(RefCell::new(Env::with_parent(Rc::clone(env))));

                    for (param, arg) in data.params.iter().zip(args) {
                        define(param, arg, &child_env);
                    }

                    let mut result = corelib::nil();
                    for expr in data.body {
                        result = eval(expr, &child_env, state)?;
                    }
                    Ok(result)
                }
                _ => Err(Error::WrongType {
                    expected: String::from("callable"),
                    got: func,
                }),
            }
        }
        _ => Ok(expr),
    }
}

pub fn pretty_wisp(source: String) -> String {
    source.replace("\n)", ")\n")
}

pub fn eval_string(
    source: String,
    env: &Rc<RefCell<Env>>,
    state: &mut State,
) -> Result<Expr, Error> {
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
