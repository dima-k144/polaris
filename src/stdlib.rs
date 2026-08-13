use std::collections::HashMap;
use crate::types::{Env, Expr};

fn plus(args: Vec<Expr>) -> Expr {
    let mut sum = 0.0;
    for arg in args {
        sum += arg.expect_number();
    }
    Expr::Number(sum)
}

pub fn std_env() -> Env {
    let mut defines = HashMap::new();

    defines.insert("nil".to_string(), Expr::Symbol(String::from("nil")));

    defines.insert("+".to_string(), Expr::NativeFunction(plus));

    Env {
        defines: defines,
        outer: None,
    }
}
