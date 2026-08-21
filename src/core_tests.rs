mod tests {
    use crate::interpreter::{eval_string, State};
    use crate::stdlib::std_env;
    use polars::prelude::*;

    #[test]
    fn test_defun_basic() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (defun add (a b) (+ a b))
            (add 10 25)
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval defun basic failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 35);
    }

    #[test]
    fn test_defun_multi_expression_body() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (defun compute (x y)
                (+ x 1)
                (* x y))
            (compute 4 5)
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval defun multi-body failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 20);
    }

    #[test]
    fn test_lambda_bound_with_define() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (define square (lambda (x) (* x x)))
            (square 6)
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval lambda bound failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 36);
    }

    #[test]
    fn test_lambda_multiple_parameters() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (define calc (lambda (a b c) (- (+ a b) c)))
            (calc 10 20 5)
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval multi-param lambda failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 25);
    }

    #[test]
    fn test_defun_scoping_isolation() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (define x 100)
            (defun override_param (x) (+ x 5))
            (override_param 10)
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval defun scope failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 15);
    }

    #[test]
    fn test_let_basic() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (let (a 1 b 2 c 3) (+ a b c))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval let basic failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 6);
    }

    #[test]
    fn test_let_scoping_isolation() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (define x 100)
            (let (x 10 y 20) (+ x y))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval let scope failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 30);

        // Verify outer scope was not mutated
        let outer_res = eval_string("x".to_string(), &env, &mut state)
            .expect("eval outer lookup failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(outer_res, 100);
    }

    #[test]
    fn test_let_multi_expression_body() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (let (a 5 b 10)
                (+ a b)
                (* a b))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval multi-expression let failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 50);
    }

    #[test]
    fn test_let_nested() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (let (a 2 b 3)
                (let (c 4 d 5)
                    (+ (* a b) (* c d))))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval nested let failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 26);
    }

    #[test]
    fn test_let_odd_bindings_error() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (let (a 1 b) (+ a 1))
        "#.to_string();

        let res = eval_string(code, &env, &mut state);
        assert!(res.is_err());
    }
}