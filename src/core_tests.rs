mod tests {
    use crate::interpreter::{eval_string, State};
    use crate::stdlib::std_env;
    use polars::prelude::*;

    #[test]
    fn test_defun_basic() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

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
        let mut state = State { line: 0, file: None };

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
        let mut state = State { line: 0, file: None };

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
        let mut state = State { line: 0, file: None };

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
        let mut state = State { line: 0, file: None };

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
        let mut state = State { line: 0, file: None };

        let code = r#"
            (let (a 1 b 2 c 3) (+ a b c))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval let basic failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 6);
    }

    //#[test]
    //fn test_let_scoping_isolation() {
    //    let env = std_env();
    //    let mut state = State { line: 0, file: None };
//
    //    let code = r#"
    //        (define x 100)
    //        (let (x 10 y 20) (+ x y))
    //    "#.to_string();
//
    //    let res = eval_string(code, &env, &mut state)
    //        .expect("eval let scope failed")
    //        .expect_int()
    //        .expect("expected number");
//
    //    assert_eq!(res, 30);
//
    //    // Verify outer scope was not mutated
    //    let outer_res = eval_string("x".to_string(), &env, &mut state)
    //        .expect("eval outer lookup failed")
    //        .expect_int()
    //        .expect("expected number");
//
    //    assert_eq!(outer_res, 100);
    //}

    #[test]
    fn test_let_multi_expression_body() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

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
        let mut state = State { line: 0, file: None };

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
        let mut state = State { line: 0, file: None };

        let code = r#"
            (let (a 1 b) (+ a 1))
        "#.to_string();

        let res = eval_string(code, &env, &mut state);
        assert!(res.is_err());
    }

    #[test]
    fn test_core_comparisons() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Greater than (>)
        assert_eq!(
            eval_string("(> 5 3)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            true
        );
        assert_eq!(
            eval_string("(> 3 5)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            false
        );
        assert_eq!(
            eval_string("(> 5.5 5.2)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            true
        );
        assert_eq!(
            eval_string("(> 5.5 5)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            true
        );

        // Less than (<)
        assert_eq!(
            eval_string("(< 2 4)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            true
        );
        assert_eq!(
            eval_string("(< 4 2)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            false
        );
        assert_eq!(
            eval_string("(< 2.5 3)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            true
        );

        // Greater than or equal (>=)
        assert_eq!(
            eval_string("(>= 10 10)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            true
        );
        assert_eq!(
            eval_string("(>= 11 10)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            true
        );
        assert_eq!(
            eval_string("(>= 9 10)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            false
        );
        assert_eq!(
            eval_string("(>= 10.0 10)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            true
        );

        // Less than or equal (<=)
        assert_eq!(
            eval_string("(<= 7 7)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            true
        );
        assert_eq!(
            eval_string("(<= 6 7)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            true
        );
        assert_eq!(
            eval_string("(<= 8 7)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            false
        );
        assert_eq!(
            eval_string("(<= 7.0 7)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            true
        );
    }

    #[test]
    fn test_core_logical_operators() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // NOT
        assert_eq!(
            eval_string("(not true)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            false
        );
        assert_eq!(
            eval_string("(not false)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            true
        );

        // AND
        assert_eq!(
            eval_string("(and true true)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            true
        );
        assert_eq!(
            eval_string("(and true false)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            false
        );
        assert_eq!(
            eval_string("(and false false)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            false
        );

        // OR
        assert_eq!(
            eval_string("(or true false)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            true
        );
        assert_eq!(
            eval_string("(or false false)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            false
        );
        assert_eq!(
            eval_string("(or true true)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            true
        );
    }

    #[test]
    fn test_core_logic_with_control_flow() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (let (x 15 y 20)
                (if (and (> x 10) (<= y 20))
                    "in-range"
                    "out-of-range"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .unwrap()
            .expect_string()
            .unwrap();

        assert_eq!(res, "in-range");
    }

    #[test]
    fn test_core_math_abs() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        assert_eq!(
            eval_string("(abs -5)".to_string(), &env, &mut state)
                .unwrap()
                .expect_int()
                .unwrap(),
            5
        );
        assert_eq!(
            eval_string("(abs 42)".to_string(), &env, &mut state)
                .unwrap()
                .expect_int()
                .unwrap(),
            42
        );
        assert_eq!(
            eval_string("(abs -3.14)".to_string(), &env, &mut state)
                .unwrap()
                .expect_float()
                .unwrap(),
            3.14
        );
    }

    #[test]
    fn test_core_math_sqrt_and_pow() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        assert_eq!(
            eval_string("(sqrt 16)".to_string(), &env, &mut state)
                .unwrap()
                .expect_float()
                .unwrap(),
            4.0
        );
        assert_eq!(
            eval_string("(pow 2 3)".to_string(), &env, &mut state)
                .unwrap()
                .expect_int()
                .unwrap(),
            8
        );
        assert_eq!(
            eval_string("(pow 2.0 3.0)".to_string(), &env, &mut state)
                .unwrap()
                .expect_float()
                .unwrap(),
            8.0
        );
        assert_eq!(
            eval_string("(pow 4 0.5)".to_string(), &env, &mut state)
                .unwrap()
                .expect_float()
                .unwrap(),
            2.0
        );
    }

    #[test]
    fn test_core_math_exp_and_logs() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        assert_eq!(
            eval_string("(exp 0)".to_string(), &env, &mut state)
                .unwrap()
                .expect_float()
                .unwrap(),
            1.0
        );

        let ln_e = eval_string("(log E)".to_string(), &env, &mut state)
            .unwrap()
            .expect_float()
            .unwrap();
        assert!((ln_e - 1.0).abs() < 1e-10);

        let log2_8 = eval_string("(log 8 2)".to_string(), &env, &mut state)
            .unwrap()
            .expect_float()
            .unwrap();
        assert!((log2_8 - 3.0).abs() < 1e-10);

        let log10_100 = eval_string("(log10 100)".to_string(), &env, &mut state)
            .unwrap()
            .expect_float()
            .unwrap();
        assert!((log10_100 - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_core_math_round_floor_ceil() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        assert_eq!(
            eval_string("(round 3.4)".to_string(), &env, &mut state)
                .unwrap()
                .expect_int()
                .unwrap(),
            3
        );
        assert_eq!(
            eval_string("(round 3.6)".to_string(), &env, &mut state)
                .unwrap()
                .expect_int()
                .unwrap(),
            4
        );
        assert_eq!(
            eval_string("(round 3.14159 2)".to_string(), &env, &mut state)
                .unwrap()
                .expect_float()
                .unwrap(),
            3.14
        );

        assert_eq!(
            eval_string("(floor 3.9)".to_string(), &env, &mut state)
                .unwrap()
                .expect_int()
                .unwrap(),
            3
        );
        assert_eq!(
            eval_string("(floor -3.1)".to_string(), &env, &mut state)
                .unwrap()
                .expect_int()
                .unwrap(),
            -4
        );

        assert_eq!(
            eval_string("(ceil 3.1)".to_string(), &env, &mut state)
                .unwrap()
                .expect_int()
                .unwrap(),
            4
        );
        assert_eq!(
            eval_string("(ceil -3.9)".to_string(), &env, &mut state)
                .unwrap()
                .expect_int()
                .unwrap(),
            -3
        );
    }

    #[test]
    fn test_core_math_trig() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let sin_zero = eval_string("(sin 0)".to_string(), &env, &mut state)
            .unwrap()
            .expect_float()
            .unwrap();
        assert!(sin_zero.abs() < 1e-10);

        let cos_zero = eval_string("(cos 0)".to_string(), &env, &mut state)
            .unwrap()
            .expect_float()
            .unwrap();
        assert!((cos_zero - 1.0).abs() < 1e-10);

        let tan_zero = eval_string("(tan 0)".to_string(), &env, &mut state)
            .unwrap()
            .expect_float()
            .unwrap();
        assert!(tan_zero.abs() < 1e-10);

        let sin_half_pi = eval_string("(sin (/ PI 2))".to_string(), &env, &mut state)
            .unwrap()
            .expect_float()
            .unwrap();
        assert!((sin_half_pi - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_infix_list1() {
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let code = "{1 + 2}".to_string();
        let result = eval_string(code, &env, &mut state).unwrap().expect_int().unwrap();
        assert!(result == 3)
    }

    #[test]
    fn test_infix_list2() {
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let code = "{1 + 2 * 3}".to_string();
        let result = eval_string(code, &env, &mut state).unwrap().expect_int().unwrap();
        assert!(result == 9)
    }

    #[test]
    fn test_core_shell() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Test running echo with multiple string arguments
        let code = r#"(shell "echo" "hello" "world")"#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval shell failed")
            .expect_string()
            .expect("expected string result");

        assert_eq!(res.trim(), "hello world");
    }

    #[test]
    fn test_core_list_operations() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // car / first
        assert_eq!(
            eval_string("(first '(1 2 3))".to_string(), &env, &mut state)
                .unwrap()
                .expect_int()
                .unwrap(),
            1
        );

        // cons
        assert_eq!(
            eval_string("(first (cons 99 '(1 2)))".to_string(), &env, &mut state)
                .unwrap()
                .expect_int()
                .unwrap(),
            99
        );

        // cdr / rest
        assert_eq!(
            eval_string("(first (rest '(1 2 3)))".to_string(), &env, &mut state)
                .unwrap()
                .expect_int()
                .unwrap(),
            2
        );

        // len
        assert_eq!(
            eval_string("(len '(10 20 30 40))".to_string(), &env, &mut state)
                .unwrap()
                .expect_int()
                .unwrap(),
            4
        );

        // nth
        assert_eq!(
            eval_string("(nth '(10 20 30) 2)".to_string(), &env, &mut state)
                .unwrap()
                .expect_int()
                .unwrap(),
            30
        );

        // reverse
        assert_eq!(
            eval_string("(first (reverse '(1 2 3)))".to_string(), &env, &mut state)
                .unwrap()
                .expect_int()
                .unwrap(),
            3
        );

        // empty?
        assert_eq!(
            eval_string("(empty? '())".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            true
        );
    }

    #[test]
    fn test_core_type_conversions() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // to-str
        assert_eq!(
            eval_string("(to-str 42)".to_string(), &env, &mut state)
                .unwrap()
                .expect_string()
                .unwrap(),
            "42"
        );
        assert_eq!(
            eval_string("(to-str true)".to_string(), &env, &mut state)
                .unwrap()
                .expect_string()
                .unwrap(),
            "true"
        );

        // to-int & to-float
        assert_eq!(
            eval_string(r#"(to-int "123")"#.to_string(), &env, &mut state)
                .unwrap()
                .expect_int()
                .unwrap(),
            123
        );
        assert_eq!(
            eval_string("(to-int 4.9)".to_string(), &env, &mut state)
                .unwrap()
                .expect_int()
                .unwrap(),
            4
        );
        assert_eq!(
            eval_string(r#"(to-float "3.14")"#.to_string(), &env, &mut state)
                .unwrap()
                .expect_float()
                .unwrap(),
            3.14
        );

        // to-bool
        assert_eq!(
            eval_string("(to-bool 0)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            false
        );
        assert_eq!(
            eval_string("(to-bool nil)".to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            false
        );
        assert_eq!(
            eval_string(r#"(to-bool "hello")"#.to_string(), &env, &mut state)
                .unwrap()
                .expect_bool()
                .unwrap(),
            true
        );

        // to-list
        let list_res = eval_string(r#"(to-list "ab")"#.to_string(), &env, &mut state)
            .unwrap()
            .expect_list()
            .unwrap();
        assert_eq!(list_res.len(), 2);
        assert_eq!(list_res[0].clone().expect_string().unwrap(), "a");
        assert_eq!(list_res[1].clone().expect_string().unwrap(), "b");
    }

    #[test]
    fn test_core_type_of() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        assert_eq!(
            eval_string("(type-of 42)".to_string(), &env, &mut state)
                .unwrap()
                .expect_string()
                .unwrap(),
            "int"
        );
        assert_eq!(
            eval_string("(type-of 3.14)".to_string(), &env, &mut state)
                .unwrap()
                .expect_string()
                .unwrap(),
            "float"
        );
        assert_eq!(
            eval_string(r#"(type-of "hello")"#.to_string(), &env, &mut state)
                .unwrap()
                .expect_string()
                .unwrap(),
            "string"
        );
        assert_eq!(
            eval_string("(type-of 'foo)".to_string(), &env, &mut state)
                .unwrap()
                .expect_string()
                .unwrap(),
            "symbol"
        );
        assert_eq!(
            eval_string("(type-of '(1 2 3))".to_string(), &env, &mut state)
                .unwrap()
                .expect_string()
                .unwrap(),
            "list"
        );
        assert_eq!(
            eval_string("(type-of true)".to_string(), &env, &mut state)
                .unwrap()
                .expect_string()
                .unwrap(),
            "bool"
        );
        assert_eq!(
            eval_string("(type-of nil)".to_string(), &env, &mut state)
                .unwrap()
                .expect_string()
                .unwrap(),
            "nil"
        );
    }
    
    #[test]
    fn test_for_loop_basic() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (define total 0)
            (for i 1 5
                (set total (+ total i)))
            total
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval for loop basic failed")
            .expect_int()
            .expect("expected int");

        assert_eq!(res, 10);
    }

    #[test]
    fn test_for_loop_custom_step() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Positive step: 0 + 2 + 4 + 6 + 8 = 20
        let code = r#"
            (define total 0)
            (for i 0 10 2
                (set total (+ total i)))
            total
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval for loop positive step failed")
            .expect_int()
            .expect("expected int");

        assert_eq!(res, 20);

        // Negative step: 10 + 7 + 4 + 1 = 22
        let code_neg = r#"
            (define total 0)
            (for i 10 0 -3
                (set total (+ total i)))
            total
        "#.to_string();

        let res_neg = eval_string(code_neg, &env, &mut state)
            .expect("eval for loop negative step failed")
            .expect_int()
            .expect("expected int");

        assert_eq!(res_neg, 22);
    }

    #[test]
    fn test_for_loop_zero_iterations() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (define run false)
            (for i 5 5
                (set run true))
            run
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval zero iteration for loop failed")
            .expect_bool()
            .expect("expected bool");

        assert_eq!(res, false);
    }

    #[test]
    fn test_for_loop_scoping() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (define i 999)
            (for i 0 3
                (+ i 1))
            i
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval for loop scope failed")
            .expect_int()
            .expect("expected int");

        assert_eq!(res, 999);
    }

    #[test]
    fn test_for_each_loop_basic() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (define acc 1)
            (for-each item '(2 3 4)
                (set acc (* acc item)))
            acc
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval for-each loop basic failed")
            .expect_int()
            .expect("expected int");

        assert_eq!(res, 24);
    }

    #[test]
    fn test_for_each_loop_empty_list() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (define counter 0)
            (for-each x '()
                (set counter (+ counter 1)))
            counter
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval empty for-each loop failed")
            .expect_int()
            .expect("expected int");

        assert_eq!(res, 0);
    }

    #[test]
    fn test_for_each_loop_multi_expression_body() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (define count 0)
            (define sum 0)
            (for-each x '(10 20 30)
                (set count (+ count 1))
                (set sum (+ sum x)))
            (+ count sum)
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval multi-expression for-each failed")
            .expect_int()
            .expect("expected int");

        assert_eq!(res, 63);
    }
}