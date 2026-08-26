==> core_tests.rs <==
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
}
==> extra_tests.rs <==
#[cfg(test)]
mod tests {
    use crate::interpreter::{eval_string, State};
    use crate::stdlib::std_env;
    use crate::types::{Expr, ExprValue};
    use polars::prelude::*;

    #[test]
    fn test_wisp_polars_pipeline() {
        let code = r#"
-> (read-csv "data1.csv")
    filter 
        > $"age" 24
    select $"name" $"city"
"#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);
        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            assert_eq!(df.shape(), (4, 2));

            let names: Vec<&str> = df.column("name").unwrap().str().unwrap().iter().flatten().collect();
            let cities: Vec<&str> = df.column("city").unwrap().str().unwrap().iter().flatten().collect();
            assert_eq!(names, vec!["Alice", "Bob", "Diana", "Evan"]);
            assert_eq!(cities, vec!["London", "Paris", "Rome", "Madrid"]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_polars_pipeline2() {
        let code = r#"
define df
    read-csv "data1.csv"

-> df
    filter 
        > $"age" 24
    select $"name" $"city"
"#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);
        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            assert_eq!(df.shape(), (4, 2));

            let names: Vec<&str> = df.column("name").unwrap().str().unwrap().iter().flatten().collect();
            let cities: Vec<&str> = df.column("city").unwrap().str().unwrap().iter().flatten().collect();
            assert_eq!(names, vec!["Alice", "Bob", "Diana", "Evan"]);
            assert_eq!(cities, vec!["London", "Paris", "Rome", "Madrid"]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }


    #[test]
    fn test_wisp_colon_chaining_and_with_column() {
        let code = r#"
-> (read-csv "data1.csv")
    with-column $"age_months" 
        * $"age" 12
    sort $"age_months" :desc
"#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);
        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            assert_eq!(df.shape(), (5, 5));

            let age_months: Vec<i64> = df.column("age_months").unwrap().i64().unwrap().into_no_null_iter().collect();
            assert_eq!(age_months, vec![420, 360, 336, 300, 264]);

            let names: Vec<&str> = df.column("name").unwrap().str().unwrap().iter().flatten().collect();
            assert_eq!(names, vec!["Evan", "Bob", "Diana", "Alice", "Charlie"]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_polars_join() {
        let code = r#"
define d1 (read-csv "data1.csv")
define d2 (read-csv "data2.csv")
-> (join d1 d2 $"id" $"id")
    select $"name" $"department" $"salary"
"#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);
        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            assert_eq!(df.shape(), (3, 3));

            let names: Vec<&str> = df.column("name").unwrap().str().unwrap().iter().flatten().collect();
            let departments: Vec<&str> = df.column("department").unwrap().str().unwrap().iter().flatten().collect();
            let salaries: Vec<i64> = df.column("salary").unwrap().i64().unwrap().into_no_null_iter().collect();

            assert_eq!(names, vec!["Alice", "Bob", "Charlie"]);
            assert_eq!(departments, vec!["Engineering", "Marketing", "Engineering"]);
            assert_eq!(salaries, vec![75000, 65000, 60000]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_polars_group_by_and_aggregation() {
        let code = r#"
-> 
    read-csv "data2.csv"
    group-by $"department"
        alias (mean $"salary") "avg_salary"
        alias (count) "count"
"#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);
        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            // Sort by department to guarantee deterministic row order
            let df = lf.sort(["department"], SortMultipleOptions::default()).collect().unwrap();
            assert_eq!(df.shape(), (3, 3));

            let depts: Vec<&str> = df.column("department").unwrap().str().unwrap().iter().flatten().collect();
            let avg_salaries: Vec<f64> = df.column("avg_salary").unwrap().f64().unwrap().into_no_null_iter().collect();
            let counts: Vec<u32> = df.column("count").unwrap().u32().unwrap().into_no_null_iter().collect();

            assert_eq!(depts, vec!["Engineering", "Finance", "Marketing"]);
            assert_eq!(avg_salaries, vec![67500.0, 80000.0, 65000.0]);
            assert_eq!(counts, vec![2, 1, 1]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_polars_over_window() {
        let code = r#"
-> (read-csv "data2.csv")
    with-column $"dept_avg" 
        over (mean $"salary") $"department"
"#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);
        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            assert_eq!(df.shape(), (4, 4));

            let dept_avgs: Vec<f64> = df.column("dept_avg").unwrap().f64().unwrap().into_no_null_iter().collect();
            assert_eq!(dept_avgs, vec![67500.0, 65000.0, 67500.0, 80000.0]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_polars_fill_null_and_when() {
        let code = r#"
defun is_senior ()
    when (>= $"age" 25) "Senior" "Junior"

-> (read-csv "data3.csv")
    with-column $"city" (fill-null $"city" "Unknown")
    with-column $"is_senior" (is_senior)
    "#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);
        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            assert_eq!(df.shape(), (5, 5));

            let cities: Vec<&str> = df.column("city").unwrap().str().unwrap().iter().flatten().collect();
            assert_eq!(cities, vec!["London", "Paris", "Unknown", "Rome", "Unknown"]);

            let is_senior: Vec<Option<&str>> = df.column("is_senior").unwrap().str().unwrap().iter().collect();
            assert_eq!(
                is_senior,
                vec![Some("Senior"), Some("Junior"), Some("Junior"), Some("Senior"), Some("Junior")]
            );
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_filter_and_sort_descending() {
        let code = r#"
-> (read-csv "data1.csv")
    filter : > $"age" 23
    sort $"age" :desc
    select $"name" $"age"
"#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df.column("name").unwrap().str().unwrap().iter().flatten().collect();
            let ages: Vec<i64> = df.column("age").unwrap().i64().unwrap().iter().flatten().collect();

            assert_eq!(df.shape(), (4, 2));
            assert_eq!(names, vec!["Evan", "Bob", "Diana", "Alice"]);
            assert_eq!(ages, vec![35, 30, 28, 25]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_with_computed_column() {
        let code = r#"
-> (read-csv "data1.csv")
    filter : = $"city" "London"
    with-column $"age_next_year" : + $"age" 1
    select $"name" $"age_next_year"
"#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df.column("name").unwrap().str().unwrap().iter().flatten().collect();
            let next_ages: Vec<i64> = df.column("age_next_year").unwrap().i64().unwrap().iter().flatten().collect();

            assert_eq!(df.shape(), (1, 2));
            assert_eq!(names, vec!["Alice"]);
            assert_eq!(next_ages, vec![26]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_string_manipulation() {
        let code = r#"
-> (read-csv "data1.csv")
    filter : str-contains $"name" "^[A-C]"
    with-column $"city_mod" : str-replace $"city" "on$" "on-City"
    select $"name" $"city_mod"
    sort $"name"
    "#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df.column("name").unwrap().str().unwrap().iter().flatten().collect();
            let cities: Vec<&str> = df.column("city_mod").unwrap().str().unwrap().iter().flatten().collect();

            assert_eq!(df.shape(), (3, 2));
            assert_eq!(names, vec!["Alice", "Bob", "Charlie"]);
            assert_eq!(cities, vec!["London-City", "Paris", "Berlin"]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_aggregation_and_scalar() {
        let code = r#"
scalar
    -> (read-csv "data1.csv")
        agg : sum $"age"
"#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::Int(total_age) = result.unwrap().value {
            assert_eq!(total_age, 140);
        } else {
            panic!("Expected Int scalar result");
        }
    }

    #[test]
    fn test_wisp_conditional_when() {
        let code = r#"
-> (read-csv "data1.csv")
    with-column $"category"
        when (> $"age" 27) "Senior" "Junior"
    sort $"id"
    select $"name" $"category"
"#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let categories: Vec<&str> = df.column("category").unwrap().str().unwrap().iter().flatten().collect();

            assert_eq!(df.shape(), (5, 2));
            assert_eq!(categories, vec!["Junior", "Senior", "Junior", "Senior", "Senior"]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_complex_filter_logical_or() {
        let code = r#"
-> (read-csv "data1.csv")
    filter : or (> $"age" 28) (= $"city" "London")
    sort $"id"
    select $"name" $"city"
    "#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df.column("name").unwrap().str().unwrap().iter().flatten().collect();
            let cities: Vec<&str> = df.column("city").unwrap().str().unwrap().iter().flatten().collect();

            assert_eq!(df.shape(), (3, 2));
            assert_eq!(names, vec!["Alice", "Bob", "Evan"]);
            assert_eq!(cities, vec!["London", "Paris", "Madrid"]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_complex_filter_logical_or2() {
        let code = r#"
-> (read-csv "data1.csv")
    filter 
        or 
            > $"age" 28
            = $"city" "London"
    sort $"id"
    select $"name" $"city"
    "#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df.column("name").unwrap().str().unwrap().iter().flatten().collect();
            let cities: Vec<&str> = df.column("city").unwrap().str().unwrap().iter().flatten().collect();

            assert_eq!(df.shape(), (3, 2));
            assert_eq!(names, vec!["Alice", "Bob", "Evan"]);
            assert_eq!(cities, vec!["London", "Paris", "Madrid"]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_slice_pagination() {
        let code = r#"
-> (read-csv "data1.csv")
    sort $"age"
    slice 1 3
    select $"name" $"age"
    "#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df.column("name").unwrap().str().unwrap().iter().flatten().collect();
            let ages: Vec<i64> = df.column("age").unwrap().i64().unwrap().iter().flatten().collect();

            assert_eq!(df.shape(), (3, 2));
            assert_eq!(names, vec!["Alice", "Diana", "Bob"]);
            assert_eq!(ages, vec![25, 28, 30]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_regex_extract() {
        let code = r#"
-> (read-csv "data1.csv")
    with-column $"code" 
        str-extract $"city" "^([A-Za-z]{3})" 1
    sort $"id"
    select $"name" $"code"
    "#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df.column("name").unwrap().str().unwrap().iter().flatten().collect();
            let codes: Vec<&str> = df.column("code").unwrap().str().unwrap().iter().flatten().collect();

            assert_eq!(df.shape(), (5, 2));
            assert_eq!(names, vec!["Alice", "Bob", "Charlie", "Diana", "Evan"]);
            assert_eq!(codes, vec!["Lon", "Par", "Ber", "Rom", "Mad"]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_cumulative_sum() {
        let code = r#"
-> (read-csv "data1.csv")
    sort $"id"
    with-column $"running_age" : cum-sum $"age"
    select $"name" $"running_age"
    "#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df.column("name").unwrap().str().unwrap().iter().flatten().collect();
            let running_ages: Vec<i64> = df.column("running_age").unwrap().i64().unwrap().iter().flatten().collect();

            assert_eq!(df.shape(), (5, 2));
            assert_eq!(names, vec!["Alice", "Bob", "Charlie", "Diana", "Evan"]);
            assert_eq!(running_ages, vec![25, 55, 77, 105, 140]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_cumulative_sum2() {
        let code = r#"
-> (read-csv "data1.csv")
    sort $"id"
    with-column $"running_age" 
        cum-sum $"age"
    select $"name" $"running_age"
"#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df.column("name").unwrap().str().unwrap().iter().flatten().collect();
            let running_ages: Vec<i64> = df.column("running_age").unwrap().i64().unwrap().iter().flatten().collect();

            assert_eq!(df.shape(), (5, 2));
            assert_eq!(names, vec!["Alice", "Bob", "Charlie", "Diana", "Evan"]);
            assert_eq!(running_ages, vec![25, 55, 77, 105, 140]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_cast_and_arithmetic() {
        let code = r#"
-> (read-csv "data1.csv")
    filter : = $"id" 1
    with-column $"half_age" : / (cast $"age" :float) 2.0
    select $"name" $"half_age"
    "#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df.column("name").unwrap().str().unwrap().iter().flatten().collect();
            let half_ages: Vec<f64> = df.column("half_age").unwrap().f64().unwrap().iter().flatten().collect();

            assert_eq!(df.shape(), (1, 2));
            assert_eq!(names, vec!["Alice"]);
            assert_eq!(half_ages, vec![12.5]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_cast_and_arithmetic2() {
        let code = r#"
-> (read-csv "data1.csv")
    filter : = $"id" 1
    with-column $"half_age" 
        / (cast $"age" :float) 2.0
    select $"name" $"half_age"
    "#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);
    
        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df.column("name").unwrap().str().unwrap().iter().flatten().collect();
            let half_ages: Vec<f64> = df.column("half_age").unwrap().f64().unwrap().iter().flatten().collect();
        
            assert_eq!(df.shape(), (1, 2));
            assert_eq!(names, vec!["Alice"]);
            assert_eq!(half_ages, vec![12.5]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_cast_and_arithmetic3() {
        let code = r#"
-> (read-csv "data1.csv")
    filter : = $"id" 1
    with-column $"half_age" 
        /
            cast $"age" :float
            . 2.0
    select $"name" $"half_age"
    "#;
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let result = eval_string(code.to_string(), &env, &mut state);
    
        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df.column("name").unwrap().str().unwrap().iter().flatten().collect();
            let half_ages: Vec<f64> = df.column("half_age").unwrap().f64().unwrap().iter().flatten().collect();
        
            assert_eq!(df.shape(), (1, 2));
            assert_eq!(names, vec!["Alice"]);
            assert_eq!(half_ages, vec![12.5]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }
}

==> interpreter.rs <==
use std::{collections::HashMap, ops::Deref};
use std::cell::RefCell;
use std::rc::Rc;

use crate::stdlib;
use crate::types::{Env, Expr, ExprValue, Error};

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
                        params.push(param.expect_symbol()?)
                    }
                    let body = list.get(2..).unwrap().to_vec();
                    let func = ExprValue::UserFunction { body: body, params: params, name: None };
                    return Ok(Expr { value: func, line: None })
                },
                "defun" => {
                    let name = list[1].clone().expect_symbol()?;
                    let mut params = Vec::<String>::new();
                    for param in list[2].clone().expect_list()? {
                        params.push(param.expect_symbol()?)
                    }
                    let body = list.get(3..).unwrap().to_vec();
                    let func = Expr { value: ExprValue::UserFunction { body: body, params: params, name: Some(name.clone()) }, line: None };
                    define(&name.clone(), func.clone(), env);
                    return Ok(func)
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
                    let mut result = stdlib::nil();
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
                ExprValue::NativeFunction{ body, min_args, max_args } => {
                    if args.len() < min_args as usize {
                        return Err(Error::WrongNumberOfArguments { expected_min: min_args, expected_max: max_args, got: args.len() as u8 });
                    }
                    
                    return body(args);
                },
                ExprValue::UserFunction { body, params, name } => {
                    if args.len() < params.len() {
                        return Err(Error::WrongNumberOfArguments { 
                            expected_min: params.len() as u8, expected_max: params.len() as u8, got: args.len() as u8 
                        });
                    }

                    let child_env = Rc::new(RefCell::new(Env::with_parent(Rc::clone(env))));

                    for (param, arg) in params.iter().zip(args) {
                        define(param, arg, &child_env);
                    }

                    let mut result = stdlib::nil();
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

pub fn pretty_wisp(source: String) -> String {
    source.replace("\n)", ")\n")
}

pub fn eval_string(source: String, env: &Rc<RefCell<Env>>, state: &mut State) -> Result<Expr, Error> {
    let s_expr_source = crate::wisp::preprocess_wisp3(&source)?;
    //println!("wisp: {}", s_expr_source.clone());
    //println!("end wisp");
    let mut tokens = crate::lexer::scan(&s_expr_source)?;
    let exprs = crate::parser::parse(&mut tokens)?;
    
    let mut last_res = stdlib::nil();
    for expr in exprs {
        // Return immediately if an expression fails
        last_res = crate::interpreter::eval(expr, env, state)?;
    }
    Ok(last_res)
}

==> lexer.rs <==
use crate::types::Error;

pub fn scan(source: &str) -> Result<Vec<(String, usize)>, Error> {
    let mut tokens = Vec::<(String, usize)>::new();
    let mut item = String::new();
    let mut line = 0;
    let mut in_string = false;
    let mut in_comment = false;
    let mut escape = false;

    for c in source.chars() {
        if in_comment {
            if c == '\n' {
                line += 1;
                in_comment = false;
            }
            continue;
        }

        if in_string {
            if escape {
                item.push(c);
                escape = false;
            } else if c == '\\' {
                item.push(c);
                escape = true;
            } else if c == '"' {
                in_string = false;
                item.push('"');
                tokens.push((item, line));
                item = String::new();
            } else {
                item.push(c);
            }
            continue;
        }

        match c {
            '(' | ')' | '[' | ']' | '{' | '}' | '$' | '\'' => {
                if !item.is_empty() {
                    tokens.push((item, line));
                    item = String::new();
                }
                tokens.push((c.to_string(), line));
            }
            ' ' | '\t' | '\r' => {
                if !item.is_empty() {
                    tokens.push((item, line));
                    item = String::new();
                }
            }
            '\n' => {
                if !item.is_empty() {
                    tokens.push((item, line));
                    item = String::new();
                }
                line += 1;
            }
            '"' => {
                if !item.is_empty() {
                    tokens.push((item, line));
                    item = String::new();
                }
                in_string = true;
                item.push('"');
            }
            ';' => {
                if !item.is_empty() {
                    tokens.push((item, line));
                    item = String::new();
                }
                in_comment = true;
            }
            _ => item.push(c),
        }
    }

    if in_string {
        return Err(Error::UnclosedString);
    }
    if !item.is_empty() {
        tokens.push((item, line));
    }

    Ok(tokens)
}
==> main.rs <==
use std::io::{self, Write};
use std::fs;
use std::cell::RefCell;
use std::rc::Rc;
use crate::types::Error;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::interpreter::eval_string;

use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

mod lexer;
mod parser;
mod types;
mod interpreter;
mod stdlib;
mod core_tests;
mod polars_tests;
mod extra_tests;
mod preprocessor_test;
mod plotlarslib;
mod wisp;

fn is_complete_input(input: &String) -> bool {
    let open_parens_count = input.chars().filter(|&c| c == '(').count();
    let closed_parens_count = input.chars().filter(|&c| c == ')').count();
    let double_quotes_count = input.chars().filter(|&c| c == '"').count();
    open_parens_count == closed_parens_count && double_quotes_count % 2 == 0
}

pub fn format_error_message(error: &Error) -> String {
    match error {
        Error::UnclosedString => format!("Error: unclosed string"),
        Error::UnclosedListExpr => format!("Error: unclosed list expression"),
        Error::FailedToParseNumber { number } => format!("Error: failed to parse number '{number}'"),
        Error::WrongType { expected, got } => format!("Error: expected {0}, got {1}", expected, got.clone().type_to_str()),
        Error::LookupNotFound { symbol } => format!("Error: symbol '{symbol}' is not defined"),
        Error::WrongNumberOfArguments { 
            expected_min, expected_max, got 
        } => format!("Error: expected {expected_min}-{expected_max} number of argumens, got {got}"),
        Error::ScalarShapeError { shape } => format!("Error: failed to convert to lisp expr, shape is {} by {}", shape.0, shape.1),
        Error::ScalarUnsupportedError { name } => format!("Error: unsupported scalar conversion from type '{name}'"),
        Error::MissingRequiredKWArg { name } => format!("Error: missing required keyword argument '{name}'"),
        Error::Custom(message) => format!("Error: {message}"),
        Error::PolarsError(perror) => format!("Error: {}", perror)
    }
}

pub fn format_error_line_info(error: &Error, state: &interpreter::State) -> Option<String> {
    match state.file.clone() {
        Some(file) => {
            let contents = fs::read_to_string(file).ok()?;
            let lines: Vec<&str> = contents.lines().collect();
            if lines.is_empty() {
                return None;
            }

            let mut result = String::new();
            result.push_str(&format!(" on line {}\n", state.line));

            let start_line = state.line.saturating_sub(2);
            let end_line = (state.line + 2).min(lines.len().saturating_sub(1));

            for idx in start_line..=end_line {
                if idx == state.line {
                    result.push_str(&format!("{} | {} <- error here\n", idx, lines[idx]));
                } else {
                    result.push_str(&format!("{} | {}\n", idx, lines[idx]));
                }
            }

            Some(result)
        }
        None => None,
    }
}

fn handle_error(error: &Error, state: &interpreter::State) {
    let error_message = format_error_message(error);
    print!("{}", error_message);

    match format_error_line_info(error, state) {
        Some(line_info) => println!("{}", line_info),
        None => {},
    }
}

fn repl() {
    let env = crate::stdlib::std_env();
    
    // Initialize the line editor
    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(err) => {
            eprintln!("Failed to initialize line reader: {err}");
            return;
        }
    };

    // Optional: Load history from a persistent file
    let history_path = dirs::home_dir().map(|p| p.join(".mlisp_history"));
    if let Some(ref path) = history_path {
        let _ = rl.load_history(path);
    }

    loop {
        // Read input with prompt (Up/Down arrows work here)
        let readline = rl.readline("mlisp > ");
        match readline {
            Ok(line) => {
                let mut input = line;
                
                // Handle multiline incomplete parenthesis input
                while !is_complete_input(&input) {
                    match rl.readline("... ") {
                        Ok(more) => {
                            input.push('\n');
                            input.push_str(&more);
                        }
                        Err(_) => break,
                    }
                }

                if input.trim().is_empty() {
                    continue;
                }

                // Add to active session history
                let _ = rl.add_history_entry(input.as_str());

                // Evaluate MLisp expression
                let mut state = interpreter::State { line: 0, file: None };
                match eval_string(input.clone(), &env, &mut state) {
                    Ok(val) => match val.to_string() {
                        Ok(str) => println!("{str}"),
                        Err(err) => handle_error(&err, &state),
                    },
                    Err(err) => handle_error(&err, &state),
                };
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
            }
            Err(ReadlineError::Eof) => {
                println!("Exiting...");
                break;
            }
            Err(err) => {
                eprintln!("REPL error: {err}");
                break;
            }
        }
    }

    // Optional: Save session history to disk on exit
    if let Some(ref path) = history_path {
        let _ = rl.save_history(path);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let lsp_files: Vec<&String> = args.iter().filter(|arg| arg.ends_with(".lsp")).collect();

    if !lsp_files.is_empty() {
        for arg in lsp_files {
            let env = crate::stdlib::std_env();
            let contents = fs::read_to_string(arg).unwrap();
            let mut state = interpreter::State { line: 0, file: Some(arg.to_string()) };
            match eval_string(contents.clone(), &env, &mut state) {
                Ok(val) => {
                    match val.to_string() {
                        Ok(str) => println!("{str}"),
                        Err(err) => handle_error(&err, &state),
                    }
                },
                Err(err) => handle_error(&err, &state),
            };
        }
        return; // Exit without entering REPL
    }

    repl();
}

==> parser.rs <==
use crate::types::{Expr, ExprValue, Error};

pub fn is_dot_list(info: &Expr) -> bool {
    match info.value.clone() {
        ExprValue::List(list) => {
            if list.is_empty() { return false; }
            match &list[0].value {
                ExprValue::Symbol(symbol) => {
                    if symbol == "." {
                        true
                    } else {
                        false
                    }
                },
                _ => false
            }
        },
        _ => false
    }
}

pub fn parse1(tokens: &mut Vec<(String, usize)>) -> Result<Expr, Error> {
    if tokens.is_empty() {
        return Err(Error::Custom("Unexpected end of input".to_string()));
    }

    let token = tokens.remove(0);
    let mut chars = token.0.chars();
    let first_char = match chars.next() {
        Some(c) => c,
        None => return Err(Error::Custom("Empty token".to_string())),
    };
    let line = token.1;

    if first_char.is_ascii_digit() || (token.0.len() > 1 && first_char == '-' && token.0.chars().nth(1).unwrap().is_ascii_digit()) {
        if token.0.contains('.') {
            match token.0.parse::<f64>() {
                Ok(value) => Ok(Expr{value: ExprValue::Float(value), line: Some(line)}),
                Err(_) => Err(Error::FailedToParseNumber { number: token.0 }),
            }
        } else {
            match token.0.parse::<i64>() {
                Ok(value) => Ok(Expr{value: ExprValue::Int(value), line: Some(line)}),
                Err(_) => Err(Error::FailedToParseNumber { number: token.0 }),
            }
        }
    } else if token.0 == "(" || token.0 == "[" || token.0 == "{" {
        let closing = match token.0.as_str() {
            "(" => ")",
            "[" => "]",
            "{" => "}",
            _ => unreachable!(),
        };

        let mut items = Vec::<Expr>::new();
        while !tokens.is_empty() && tokens[0].0 != closing {
            let item = parse1(tokens)?;
            if is_dot_list(&item) {
                for x in item.expect_list()?.iter().skip(1) {
                    items.push(x.clone());
                }
            } else {
                items.push(item);
            }
        }

        if tokens.is_empty() {
            return Err(Error::UnclosedListExpr);
        }

        tokens.remove(0); // consume closing paren/bracket/brace
        Ok(Expr { value: ExprValue::List(items), line: Some(line) })
    } else if first_char == '"' {
        let inner = &token.0[1..token.0.len().saturating_sub(1)];
        Ok(Expr { value: ExprValue::String(inner.to_string()), line: Some(line) })
    } else if token.0 == "$" {
        let first = Expr { value: ExprValue::Symbol(String::from("col")), line: Some(line) };
        let second = parse1(tokens)?;
        Ok(Expr { value: ExprValue::List(vec![first, second]), line: Some(line) })
    } else if token.0 == "'" {
        let first = Expr { value: ExprValue::Symbol(String::from("quote")), line: Some(line) };
        let second = parse1(tokens)?;
        Ok(Expr { value: ExprValue::List(vec![first, second]), line: Some(line) })
    } else {
        Ok(Expr { value: ExprValue::Symbol(token.0), line: Some(line) })
    }
}

pub fn parse(tokens: &mut Vec<(String, usize)>) -> Result<Vec<Expr>, Error> {
    let mut exprs = Vec::<Expr>::new();
    while !tokens.is_empty() {
        exprs.push(parse1(tokens)?);
    }
    Ok(exprs)
}
==> plotlarslib.rs <==
use plotlars::{Plot, ScatterPlot, BarPlot, PieChart, Histogram, BoxPlot, TimeSeriesPlot, LinePlot};
use crate::types::{Expr, ExprValue, Error};
use crate::stdlib;
use polars::prelude::*;

fn kw(args: &[Expr], name: &str) -> Option<Expr> {
    for (i, arg) in args.iter().enumerate() {
        if let ExprValue::Symbol(symbol) = arg.value.clone() {
            if symbol == name {
                return args.get(i + 1).cloned();
            }
        }
    }
    None
}

fn require_kw(args: &[Expr], name: &str) -> Result<Expr, Error> {
    match kw(args, name) {
        Some(expr) => Ok(expr),
        None => Err(Error::MissingRequiredKWArg { name: name.to_string() }),
    }
}

fn has_flag(args: &[Expr], name: &str) -> bool {
    args.iter().any(|arg| matches!(arg.value.clone(), ExprValue::Symbol(s) if s == name))
}

pub fn scatter_plot(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args.first()
        .ok_or_else(|| Error::MissingRequiredKWArg { name: "dataframe".to_string() })?
        .clone()
        .expect_lazy_frame()?;

    let df = lf.collect().map_err(Error::PolarsError)?;

    // Required arguments
    let x = require_kw(&args, ":x")?.expect_string()?;
    let y = require_kw(&args, ":y")?.expect_string()?;

    // Optional arguments
    let title = kw(&args, ":title").map(|k| k.expect_string()).transpose()?;
    let x_title = kw(&args, ":x-title").map(|k| k.expect_string()).transpose()?;
    let y_title = kw(&args, ":y-title").map(|k| k.expect_string()).transpose()?;
    let group = kw(&args, ":group").map(|k| k.expect_string()).transpose()?;
    let opacity = kw(&args, ":opacity").map(|k| k.expect_float()).transpose()?;
    let size = kw(&args, ":size")
    .map(|k| {
        let val = k.expect_int()?;
        usize::try_from(val).map_err(|_| Error::Custom(format!("`:size` must be a positive integer, got {}", val)))
    })
    .transpose()?;

    let plot = ScatterPlot::builder()
        .data(&df)
        .x(x.as_str())
        .y(y.as_str())
        .maybe_plot_title(title.as_deref())
        .maybe_x_title(x_title.as_deref())
        .maybe_y_title(y_title.as_deref())
        .maybe_group(group.as_deref())
        .maybe_size(size)
        .maybe_opacity(opacity)
        .build();

    plot.plot();

    Ok(stdlib::nil())
}

pub fn bar_plot(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args.first()
        .ok_or_else(|| Error::MissingRequiredKWArg { name: "dataframe".to_string() })?
        .clone()
        .expect_lazy_frame()?;

    let df = lf.collect().map_err(Error::PolarsError)?;

    // Required arguments (accepts :labels or alias :x, and :values or alias :y)
    let labels = kw(&args, ":labels")
        .or_else(|| kw(&args, ":x"))
        .ok_or_else(|| Error::MissingRequiredKWArg { name: ":labels".to_string() })?
        .expect_string()?;

    let values = kw(&args, ":values")
        .or_else(|| kw(&args, ":y"))
        .ok_or_else(|| Error::MissingRequiredKWArg { name: ":values".to_string() })?
        .expect_string()?;

    // Optional arguments
    let title = kw(&args, ":title").map(|k| k.expect_string()).transpose()?;
    let x_title = kw(&args, ":x-title").map(|k| k.expect_string()).transpose()?;
    let y_title = kw(&args, ":y-title").map(|k| k.expect_string()).transpose()?;
    let group = kw(&args, ":group").map(|k| k.expect_string()).transpose()?;
    let error = kw(&args, ":error").map(|k| k.expect_string()).transpose()?;

    let plot = BarPlot::builder()
        .data(&df)
        .labels(labels.as_str())
        .values(values.as_str())
        .maybe_plot_title(title.as_deref())
        .maybe_x_title(x_title.as_deref())
        .maybe_y_title(y_title.as_deref())
        .maybe_group(group.as_deref())
        .maybe_error(error.as_deref())
        .build();

    plot.plot();

    Ok(stdlib::nil())
}

pub fn pie_plot(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args.first()
        .ok_or_else(|| Error::MissingRequiredKWArg { name: "dataframe".to_string() })?
        .clone()
        .expect_lazy_frame()?;

    let df = lf.collect().map_err(Error::PolarsError)?;

    // Required arguments
    let labels = require_kw(&args, ":labels")?.expect_string()?;

    // Optional arguments
    let title = kw(&args, ":title").map(|k| k.expect_string()).transpose()?;
    let hole = kw(&args, ":hole").map(|k| k.expect_float()).transpose()?;
    let pull = kw(&args, ":pull").map(|k| k.expect_float()).transpose()?;
    let rotation = kw(&args, ":rotation").map(|k| k.expect_float()).transpose()?;
    let facet = kw(&args, ":facet").map(|k| k.expect_string()).transpose()?;

    let plot = PieChart::builder()
        .data(&df)
        .labels(labels.as_str())
        .maybe_plot_title(title.as_deref())
        .maybe_hole(hole)
        .maybe_pull(pull)
        .maybe_rotation(rotation)
        .maybe_facet(facet.as_deref())
        .build();

    plot.plot();

    Ok(stdlib::nil())
}

pub fn histogram(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args.first()
        .ok_or_else(|| Error::MissingRequiredKWArg { name: "dataframe".to_string() })?
        .clone()
        .expect_lazy_frame()?;

    let df = lf.collect().map_err(Error::PolarsError)?;

    // Required arguments
    let x = require_kw(&args, ":x")?.expect_string()?;

    // Optional arguments
    let title = kw(&args, ":title").map(|k| k.expect_string()).transpose()?;
    let x_title = kw(&args, ":x-title").map(|k| k.expect_string()).transpose()?;
    let y_title = kw(&args, ":y-title").map(|k| k.expect_string()).transpose()?;
    let group = kw(&args, ":group").map(|k| k.expect_string()).transpose()?;
    let opacity = kw(&args, ":opacity").map(|k| k.expect_float()).transpose()?;
    let facet = kw(&args, ":facet").map(|k| k.expect_string()).transpose()?;

    let plot = Histogram::builder()
        .data(&df)
        .x(x.as_str())
        .maybe_plot_title(title.as_deref())
        .maybe_x_title(x_title.as_deref())
        .maybe_y_title(y_title.as_deref())
        .maybe_group(group.as_deref())
        .maybe_opacity(opacity)
        .maybe_facet(facet.as_deref())
        .build();

    plot.plot();

    Ok(stdlib::nil())
}

pub fn box_plot(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args.first()
        .ok_or_else(|| Error::MissingRequiredKWArg { name: "dataframe".to_string() })?
        .clone()
        .expect_lazy_frame()?;

    let df = lf.collect().map_err(Error::PolarsError)?;

    // Required arguments (accepts :labels / :x, and :values / :y)
    let labels = kw(&args, ":labels")
        .or_else(|| kw(&args, ":x"))
        .ok_or_else(|| Error::MissingRequiredKWArg { name: ":labels".to_string() })?
        .expect_string()?;

    let values = kw(&args, ":values")
        .or_else(|| kw(&args, ":y"))
        .ok_or_else(|| Error::MissingRequiredKWArg { name: ":values".to_string() })?
        .expect_string()?;

    // Optional arguments
    let title = kw(&args, ":title").map(|k| k.expect_string()).transpose()?;
    let x_title = kw(&args, ":x-title").map(|k| k.expect_string()).transpose()?;
    let y_title = kw(&args, ":y-title").map(|k| k.expect_string()).transpose()?;
    let group = kw(&args, ":group").map(|k| k.expect_string()).transpose()?;
    let opacity = kw(&args, ":opacity").map(|k| k.expect_float()).transpose()?;
    let facet = kw(&args, ":facet").map(|k| k.expect_string()).transpose()?;
    let jitter = kw(&args, ":jitter").map(|k| k.expect_float()).transpose()?;
    let point_offset = kw(&args, ":point-offset").map(|k| k.expect_float()).transpose()?;
    let box_points = if has_flag(&args, ":box-points") {
        Some(true)
    } else {
        None
    };

    let plot = BoxPlot::builder()
        .data(&df)
        .labels(labels.as_str())
        .values(values.as_str())
        .maybe_plot_title(title.as_deref())
        .maybe_x_title(x_title.as_deref())
        .maybe_y_title(y_title.as_deref())
        .maybe_group(group.as_deref())
        .maybe_opacity(opacity)
        .maybe_facet(facet.as_deref())
        .maybe_box_points(box_points)
        .maybe_point_offset(point_offset)
        .maybe_jitter(jitter)
        .build();

    plot.plot();

    Ok(stdlib::nil())
}

pub fn time_series_plot(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args.first()
        .ok_or_else(|| Error::MissingRequiredKWArg { name: "dataframe".to_string() })?
        .clone()
        .expect_lazy_frame()?;

    let mut df = lf.collect().map_err(Error::PolarsError)?;

    // Required arguments
    let x = require_kw(&args, ":x")?.expect_string()?;
    let y = require_kw(&args, ":y")?.expect_string()?;

    // Cast Date / Datetime to ISO-8601 String for Plotly rendering
    if let Ok(col) = df.column(&x) {
        if matches!(col.dtype(), DataType::Date | DataType::Datetime(_, _)) {
            let str_col = col.cast(&DataType::String).map_err(Error::PolarsError)?;
            df.with_column(str_col).map_err(Error::PolarsError)?;
        }
    }

    // Optional arguments
    let title = kw(&args, ":title").map(|k| k.expect_string()).transpose()?;
    let x_title = kw(&args, ":x-title").map(|k| k.expect_string()).transpose()?;
    let y_title = kw(&args, ":y-title").map(|k| k.expect_string()).transpose()?;
    let y2_title = kw(&args, ":y2-title").map(|k| k.expect_string()).transpose()?;
    let facet = kw(&args, ":facet").map(|k| k.expect_string()).transpose()?;
    let width = kw(&args, ":width").map(|k| k.expect_float()).transpose()?;
    let size = kw(&args, ":size")
        .map(|k| {
            let val = k.expect_int()?;
            usize::try_from(val).map_err(|_| Error::Custom(format!("`:size` must be a positive integer, got {}", val)))
        })
        .transpose()?;

    // Optional additional series (expects a list of strings or single string)
    let additional_series = match kw(&args, ":additional-series") {
        Some(some) => match some.value.clone() {
            ExprValue::List(list) => {
                let mut series_vec = Vec::new();
                for item in list {
                    series_vec.push(item.expect_string()?);
                }
                Some(series_vec)
            }
            ExprValue::String(s) => Some(vec![s]),
            other => return Err(Error::WrongType { expected: "list or string".into(), got: some.clone() }),
        },
        None => None,
    };

    let additional_series_slices: Option<Vec<&str>> = additional_series
        .as_ref()
        .map(|v| v.iter().map(|s| s.as_str()).collect());

    let plot = TimeSeriesPlot::builder()
        .data(&df)
        .x(x.as_str())
        .y(y.as_str())
        .maybe_additional_series(additional_series_slices)
        .maybe_plot_title(title.as_deref())
        .maybe_x_title(x_title.as_deref())
        .maybe_y_title(y_title.as_deref())
        .maybe_y2_title(y2_title.as_deref())
        .maybe_facet(facet.as_deref())
        .maybe_width(width)
        .maybe_size(size)
        .build();

    plot.plot();

    Ok(stdlib::nil())
}

pub fn line_plot(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args.first()
        .ok_or_else(|| Error::MissingRequiredKWArg { name: "dataframe".to_string() })?
        .clone()
        .expect_lazy_frame()?;

    let df = lf.collect().map_err(Error::PolarsError)?;

    // Required arguments
    let x = require_kw(&args, ":x")?.expect_string()?;
    let y = require_kw(&args, ":y")?.expect_string()?;

    // Optional arguments
    let title = kw(&args, ":title").map(|k| k.expect_string()).transpose()?;
    let x_title = kw(&args, ":x-title").map(|k| k.expect_string()).transpose()?;
    let y_title = kw(&args, ":y-title").map(|k| k.expect_string()).transpose()?;
    let y2_title = kw(&args, ":y2-title").map(|k| k.expect_string()).transpose()?;
    let facet = kw(&args, ":facet").map(|k| k.expect_string()).transpose()?;
    let width = kw(&args, ":width").map(|k| k.expect_float()).transpose()?;
    let size = kw(&args, ":size")
        .map(|k| {
            let val = k.expect_int()?;
            usize::try_from(val).map_err(|_| Error::Custom(format!("`:size` must be a positive integer, got {}", val)))
        })
        .transpose()?;

    // Optional additional lines (expects a list of strings or a single string)
    let additional_lines = match kw(&args, ":additional-lines") {
        Some(some) => match some.value.clone() {
            ExprValue::List(list) => {
                let mut lines_vec = Vec::new();
                for item in list {
                    lines_vec.push(item.expect_string()?);
                }
                Some(lines_vec)
            }
            ExprValue::String(s) => Some(vec![s]),
            other => return Err(Error::WrongType { expected: "list or string".into(), got: some }),
        },
        None => None,
    };

    let additional_lines_slices: Option<Vec<&str>> = additional_lines
        .as_ref()
        .map(|v| v.iter().map(|s| s.as_str()).collect());

    let plot = LinePlot::builder()
        .data(&df)
        .x(x.as_str())
        .y(y.as_str())
        .maybe_additional_lines(additional_lines_slices)
        .maybe_plot_title(title.as_deref())
        .maybe_x_title(x_title.as_deref())
        .maybe_y_title(y_title.as_deref())
        .maybe_y2_title(y2_title.as_deref())
        .maybe_facet(facet.as_deref())
        .maybe_width(width)
        .maybe_size(size)
        .build();

    plot.plot();

    Ok(stdlib::nil())
}

==> polars_tests.rs <==
#[cfg(test)]
mod tests {
    use crate::interpreter::{eval_string, State};
    use crate::types::Error;
    use crate::stdlib::std_env;
    use polars::prelude::*;
    use std::fs::File;

    fn generate_data1_parquet() -> Result<(), PolarsError> {
        let mut df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "department" => &["Engineering", "HR", "Engineering", "Marketing", "HR"],
            "salary" => &[90000i64, 60000, 95000, 70000, 65000]
        )?;

        let file = File::create("data1.parquet").unwrap();
        ParquetWriter::new(file).finish(&mut df)?;
        Ok(())
    }

    #[test]
    fn test_filter_eq() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (filter (read-csv "data1.csv") (= $"age" 30))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with '=' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[2i64],
            "name" => &["Bob"],
            "age" => &[30i64],
            "city" => &["Paris"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_neq() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (filter (read-csv "data1.csv") (!= $"age" 30))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with '!=' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 3, 4, 5],
            "name" => &["Alice", "Charlie", "Diana", "Evan"],
            "age" => &[25i64, 22, 28, 35],
            "city" => &["London", "Berlin", "Rome", "Madrid"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_gt() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (filter (read-csv "data1.csv") (> $"age" 25))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with '>' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[2i64, 4, 5],
            "name" => &["Bob", "Diana", "Evan"],
            "age" => &[30i64, 28, 35],
            "city" => &["Paris", "Rome", "Madrid"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_lt() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (filter (read-csv "data1.csv") (< $"age" 28))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with '<' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 3],
            "name" => &["Alice", "Charlie"],
            "age" => &[25i64, 22],
            "city" => &["London", "Berlin"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_gt_eq() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (filter (read-csv "data1.csv") (>= $"age" 28))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with '>=' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[2i64, 4, 5],
            "name" => &["Bob", "Diana", "Evan"],
            "age" => &[30i64, 28, 35],
            "city" => &["Paris", "Rome", "Madrid"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_lt_eq() {
        let env = std_env();
        let mut state = State { line: 0, file: None };
    
        let code = r#"
            (filter (read-csv "data1.csv") (<= $"age" 28))
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with '<=' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");
    
        let df = res.collect().expect("failed to collect DataFrame");
    
        let expected_df = df!(
            "id" => &[1i64, 3, 4],
            "name" => &["Alice", "Charlie", "Diana"],
            "age" => &[25i64, 22, 28],
            "city" => &["London", "Berlin", "Rome"],
        ).expect("failed to construct expected DataFrame");
    
        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_and() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (filter (read-csv "data1.csv") (and (> $"age" 22) (< $"age" 35)))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with 'and' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 4],
            "name" => &["Alice", "Bob", "Diana"],
            "age" => &[25i64, 30, 28],
            "city" => &["London", "Paris", "Rome"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_or() {
        let env = std_env();
        let mut state = State { line: 0, file: None };
    
        let code = r#"
            (filter (read-csv "data1.csv") (or (< $"age" 25) (>= $"age" 35)))
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with 'or' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");
    
        let df = res.collect().expect("failed to collect DataFrame");
    
        let expected_df = df!(
            "id" => &[3i64, 5],
            "name" => &["Charlie", "Evan"],
            "age" => &[22i64, 35],
            "city" => &["Berlin", "Madrid"],
        ).expect("failed to construct expected DataFrame");
    
        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_not() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (filter (read-csv "data1.csv") (not (> $"age" 25)))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with 'not' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 3],
            "name" => &["Alice", "Charlie"],
            "age" => &[25i64, 22],
            "city" => &["London", "Berlin"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_select_sum() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (select (read-csv "data1.csv") (sum $"age"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval select with 'sum' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "age" => &[140i64],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_select_mean() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (select (read-csv "data1.csv") (mean $"age"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval select with 'mean' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "age" => &[28.0f64],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_select_median() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (select (read-csv "data1.csv") (median $"age"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval select with 'median' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "age" => &[28.0f64],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_with_column_computed() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (with-column (read-csv "data1.csv") $"is_over_25" (> $"age" 25))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval with-column failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "age" => &[25i64, 30, 22, 28, 35],
            "city" => &["London", "Paris", "Berlin", "Rome", "Madrid"],
            "is_over_25" => &[false, true, false, true, true],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_sort_asc() {
        let env = std_env();
        let mut state = State { line: 0, file: None };
    
        let code = r#"
            (sort (read-csv "data1.csv") $"age")
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval sort ascending failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");
    
        let df = res.collect().expect("failed to collect DataFrame");
    
        let expected_df = df!(
            "id" => &[3i64, 1, 4, 2, 5],
            "name" => &["Charlie", "Alice", "Diana", "Bob", "Evan"],
            "age" => &[22i64, 25, 28, 30, 35],
            "city" => &["Berlin", "London", "Rome", "Paris", "Madrid"],
        ).expect("failed to construct expected DataFrame");
    
        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_group_by() {
        use crate::interpreter::{eval_string, State};
        use crate::stdlib::std_env;
        use polars::prelude::*;
    
        let env = std_env();
        let mut state = State { line: 0, file: None };
    
        // Group by city and compute mean age, then sort by city to ensure deterministic order
        let code = r#"
            (sort (group-by (read-csv "data1.csv") $"city" (mean $"age")) $"city")
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval group-by failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");
    
        let df = res.collect().expect("failed to collect DataFrame");
    
        let expected_df = df!(
            "city" => &["Berlin", "London", "Madrid", "Paris", "Rome"],
            "age" => &[22.0f64, 25.0, 35.0, 30.0, 28.0],
        ).expect("failed to construct expected DataFrame");
    
        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_agg_multiple() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (agg (read-csv "data1.csv") 
                 (alias (sum $"age") "total_age") 
                 (alias (mean $"age") "mean_age"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval agg failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "total_age" => &[140i64],
            "mean_age" => &[28.0f64],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_join_inner() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (sort (join (read-csv "data1.csv") (read-csv "data2.csv") $"id" $"id" :inner) $"id")
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval inner join failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3],
            "name" => &["Alice", "Bob", "Charlie"],
            "age" => &[25i64, 30, 22],
            "city" => &["London", "Paris", "Berlin"],
            "department" => &["Engineering", "Marketing", "Engineering"],
            "salary" => &[75000i64, 65000, 60000],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_arithmetic_plus() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let res = eval_string("(+ 1 2 3 4)".to_string(), &env, &mut state)
            .expect("eval failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 10);
    }

    #[test]
    fn test_arithmetic_minus() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let res = eval_string("(- 20 5 3 2)".to_string(), &env, &mut state)
            .expect("eval failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 10);
    }

    #[test]
    fn test_arithmetic_mult() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let res = eval_string("(* 2 3 4)".to_string(), &env, &mut state)
            .expect("eval failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 24);
    }

    #[test]
    fn test_arithmetic_divide() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let res = eval_string("(/ 100 2 5)".to_string(), &env, &mut state)
            .expect("eval failed")
            .expect_float()
            .expect("expected number");

        assert_eq!(res, 10.0);
    }

    #[test]
    fn test_arithmetic_nested() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Evaluates: ((10 + 5) * 2) - (20 / 4) = (15 * 2) - 5 = 30 - 5 = 25
        let code = "(- (* (+ 10 5) 2) (/ 20 4))".to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval failed")
            .expect_float()
            .expect("expected number");

        assert_eq!(res, 25.0);
    }

    #[test]
    fn test_polars_plus() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (with-column (select (read-csv "data1.csv") $"id" $"age") $"age_plus_5" (+ $"age" 5))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval plus failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "age" => &[25i64, 30, 22, 28, 35],
            "age_plus_5" => &[30i64, 35, 27, 33, 40],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_polars_minus() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (with-column (select (read-csv "data1.csv") $"id" $"age") $"age_minus_2" (- $"age" 2))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval minus failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "age" => &[25i64, 30, 22, 28, 35],
            "age_minus_2" => &[23i64, 28, 20, 26, 33],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_polars_mult() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (with-column (select (read-csv "data1.csv") $"id" $"age") $"age_double" (* $"age" 2))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval multiply failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "age" => &[25i64, 30, 22, 28, 35],
            "age_double" => &[50i64, 60, 44, 56, 70],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_polars_divide() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (with-column (select (read-csv "data1.csv") $"id" $"age") $"age_half" (/ $"age" 2))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval divide failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "age" => &[25i64, 30, 22, 28, 35],
            "age_half" => &[12.5f64, 15.0, 11.0, 14.0, 17.5],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_is_null() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (filter (read-csv "data3.csv") (null? $"age"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with 'null?' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[2i64, 5],
            "name" => &[Some("Bob"), Some("Evan")],
            "age" => &[None::<i64>, None::<i64>],
            "city" => &[Some("Paris"), None],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_is_not_null() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (filter (read-csv "data3.csv") (not-null? $"city"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with 'not-null?' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 4],
            "name" => &[Some("Alice"), Some("Bob"), None],
            "age" => &[Some(25i64), None, Some(28)],
            "city" => &[Some("London"), Some("Paris"), Some("Rome")],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_fill_null() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (with-column (read-csv "data3.csv") $"age" (fill-null $"age" 0))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval with-column fill-null failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &[Some("Alice"), Some("Bob"), Some("Charlie"), None, Some("Evan")],
            "age" => &[25i64, 0, 22, 28, 0],
            "city" => &[Some("London"), Some("Paris"), None, Some("Rome"), None],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_drop_nulls_all() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (drop-nulls (read-csv "data3.csv"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval drop-nulls failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64],
            "name" => &[Some("Alice")],
            "age" => &[Some(25i64)],
            "city" => &[Some("London")],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_drop_nulls_selected_columns() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (drop-nulls (read-csv "data3.csv") $"name" $"age")
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval drop-nulls with columns failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 3],
            "name" => &[Some("Alice"), Some("Charlie")],
            "age" => &[Some(25i64), Some(22)],
            "city" => &[Some("London"), None],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_when_with_otherwise() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (with-column (read-csv "data1.csv") $"category" (when (>= $"age" 30) "senior" "junior"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval when with otherwise failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "age" => &[25i64, 30, 22, 28, 35],
            "city" => &["London", "Paris", "Berlin", "Rome", "Madrid"],
            "category" => &["junior", "senior", "junior", "junior", "senior"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_when_without_otherwise() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (with-column (read-csv "data1.csv") $"status" (when (>= $"age" 30) "senior"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval 2-arg when failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "age" => &[25i64, 30, 22, 28, 35],
            "city" => &["London", "Paris", "Berlin", "Rome", "Madrid"],
            "status" => &[Option::<&str>::None, Some("senior"), None, None, Some("senior")],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_aggregations_min_max_count_nunique_std() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (agg (read-csv "data1.csv")
                 (alias (min $"age") "min_age")
                 (alias (max $"age") "max_age")
                 (alias (count $"age") "count_age")
                 (alias (n-unique $"city") "unique_cities"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval aggregations failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "min_age" => &[22i64],
            "max_age" => &[35i64],
            "count_age" => &[5u32],
            "unique_cities" => &[5u32],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_write_csv_roundtrip() {
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let temp_output = "test_output.csv";

        // Filter and write to CSV
        let write_code = format!(
            r#"(write-csv (filter (read-csv "data1.csv") (= $"age" 30)) "{}")"#,
            temp_output
        );

        eval_string(write_code, &env, &mut state)
            .expect("eval write-csv failed");

        // Read back and assert content matches
        let read_code = format!(r#"(read-csv "{}")"#, temp_output);
        let res = eval_string(read_code, &env, &mut state)
            .expect("eval read-csv on written file failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[2i64],
            "name" => &["Bob"],
            "age" => &[30i64],
            "city" => &["Paris"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);

        // Clean up temporary test file
        let _ = std::fs::remove_file(temp_output);
    }

    #[test]
    fn test_str_contains() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Use a single backslash inside r#"..."#
        let code = r#"
            (filter (read-csv "data4.csv") (str-contains $"email" "@example\.com$"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with 'str-contains' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 4],
            "name" => &["Alice Johnson", "Diana Prince"],
            "email" => &["alice.j@example.com", "diana.p@example.com"],
            "sku" => &["PROD-101-US", "STAGING-404-US"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }
    
    #[test]
    fn test_str_replace() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Mask the numbers in the SKU column with "XXX"
        let code = r#"
            (with-column (read-csv "data4.csv") $"masked_sku" (str-replace $"sku" "[0-9]+" "XXX"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval with-column 'str-replace' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice Johnson", "Bob Smith", "Charlie Brown", "Diana Prince", "Evan Wright"],
            "email" => &[
                "alice.j@example.com",
                "bob_smith@work.org",
                "charlie@test.net",
                "diana.p@example.com",
                "evan_w@domain.io",
            ],
            "sku" => &[
                "PROD-101-US",
                "DEV-202-EU",
                "PROD-303-APAC",
                "STAGING-404-US",
                "DEV-505-LATAM",
            ],
            "masked_sku" => &[
                "PROD-XXX-US",
                "DEV-XXX-EU",
                "PROD-XXX-APAC",
                "STAGING-XXX-US",
                "DEV-XXX-LATAM",
            ],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_str_extract() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Extract the numeric portion (capture group 1) from SKU
        let code = r#"
            (with-column (read-csv "data4.csv") $"sku_code" (str-extract $"sku" ".*-([0-9]+)-.*" 1))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval with-column 'str-extract' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice Johnson", "Bob Smith", "Charlie Brown", "Diana Prince", "Evan Wright"],
            "email" => &[
                "alice.j@example.com",
                "bob_smith@work.org",
                "charlie@test.net",
                "diana.p@example.com",
                "evan_w@domain.io",
            ],
            "sku" => &[
                "PROD-101-US",
                "DEV-202-EU",
                "PROD-303-APAC",
                "STAGING-404-US",
                "DEV-505-LATAM",
            ],
            "sku_code" => &[
                Some("101"),
                Some("202"),
                Some("303"),
                Some("404"),
                Some("505"),
            ],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_window_over_mean() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Compute average age per city over the dataset without grouping
        let code = r#"
            (with-column (read-csv "data1.csv") $"city_mean_age" (over (mean $"age") $"city"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval over mean failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "age" => &[25i64, 30, 22, 28, 35],
            "city" => &["London", "Paris", "Berlin", "Rome", "Madrid"],
            "city_mean_age" => &[25.0f64, 30.0, 22.0, 28.0, 35.0],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_window_shift_and_cumsum() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (-> (read-csv "data1.csv")
                (with-column $"prev_age" (shift $"age" 1))
                (with-column $"running_age" (cum-sum $"age")))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval shift/cum-sum failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "age" => &[25i64, 30, 22, 28, 35],
            "city" => &["London", "Paris", "Berlin", "Rome", "Madrid"],
            "prev_age" => &[None::<i64>, Some(25), Some(30), Some(22), Some(28)],
            "running_age" => &[25i64, 55, 77, 105, 140],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_to_date_and_components() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Parse date and extract year, month, and day components
        let code = r#"
            (-> (read-csv "data5.csv")
                (with-column $"date" (to-date $"date_str" "%Y-%m-%d"))
                (with-column $"year" (dt-year $"date"))
                (with-column $"month" (dt-month $"date"))
                (with-column $"day" (dt-day $"date"))
                (select $"id" $"year" $"month" $"day"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval to-date and dt components failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3],
            "year" => &[2024i32, 2025, 2026],
            "month" => &[1i8, 6, 11],
            "day" => &[15i8, 20, 5],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_to_datetime_and_hour() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Parse datetime and extract hour component
        let code = r#"
            (-> (read-csv "data5.csv")
                (with-column $"dt" (to-datetime $"datetime_str" "%Y-%m-%d %H:%M:%S"))
                (with-column $"hour" (dt-hour $"dt"))
                (select $"id" $"hour"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval to-datetime and dt-hour failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3],
            "hour" => &[8i8, 14, 22],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_dt_format() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Parse date and reformat to custom display string
        let code = r#"
            (-> (read-csv "data5.csv")
                (with-column $"date" (to-date $"date_str" "%Y-%m-%d"))
                (with-column $"formatted_date" (dt-format $"date" "%d/%m/%Y"))
                (select $"id" $"formatted_date"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval dt-format failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3],
            "formatted_date" => &["15/01/2024", "20/06/2025", "05/11/2026"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_by_date_component() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Filter rows where year is greater than or equal to 2025
        let code = r#"
            (-> (read-csv "data5.csv")
                (with-column $"date" (to-date $"date_str" "%Y-%m-%d"))
                (filter (>= (dt-year $"date") 2025))
                (select $"id" $"date_str"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter by date component failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[2i64, 3],
            "date_str" => &["2025-06-20", "2026-11-05"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_write_and_read_parquet_roundtrip() {
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let temp_parquet = "test_output.parquet";
    
        // 1. Filter CSV data and write to Parquet
        let write_code = format!(
            r#"(write-parquet (filter (read-csv "data1.csv") (= $"city" "Paris")) "{}")"#,
            temp_parquet
        );
    
        eval_string(write_code, &env, &mut state)
            .expect("eval write-parquet failed");
    
        // 2. Read back from Parquet and assert contents
        let read_code = format!(r#"(read-parquet "{}")"#, temp_parquet);
        let res = eval_string(read_code, &env, &mut state)
            .expect("eval read-parquet failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");
    
        let df = res.collect().expect("failed to collect DataFrame");
    
        let expected_df = df!(
            "id" => &[2i64],
            "name" => &["Bob"],
            "age" => &[30i64],
            "city" => &["Paris"],
        ).expect("failed to construct expected DataFrame");
    
        assert_eq!(df, expected_df);
    
        // Clean up temporary test file
        let _ = std::fs::remove_file(temp_parquet);
    }
    
    #[test]
    fn test_parquet_query_pipeline() {
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let parquet_path = "data1.parquet";
    
        // Create fixture data1.parquet using polars directly before querying
        let mut df_fixture = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "department" => &["Engineering", "HR", "Engineering", "Marketing", "HR"],
            "salary" => &[90000i64, 60000, 95000, 70000, 65000],
        ).expect("failed to create fixture DataFrame");
    
        let file = std::fs::File::create(parquet_path).expect("failed to create parquet file");
        ParquetWriter::new(file)
            .finish(&mut df_fixture)
            .expect("failed to write parquet fixture");
    
        // Run query over data1.parquet via MLisp
        let code = r#"
            (-> (read-parquet "data1.parquet")
                (filter (= $"department" "Engineering"))
                (select (alias (mean $"salary") "avg_eng_salary")))
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval read-parquet pipeline failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");
    
        let df = res.collect().expect("failed to collect DataFrame");
    
        let expected_df = df!(
            "avg_eng_salary" => &[92500.0f64],
        ).expect("failed to construct expected DataFrame");
    
        assert_eq!(df, expected_df);
    
        // Clean up fixture file
        let _ = std::fs::remove_file(parquet_path);
    }

    #[test]
    fn test_sink_csv_roundtrip() {
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let temp_output = "test_sink_output.csv";

        // Stream filtered lazy plan directly to CSV
        let sink_code = format!(
            r#"(sink (filter (read-csv "data1.csv") (= $"city" "Paris")) "{}")"#,
            temp_output
        );

        eval_string(sink_code, &env, &mut state)
            .expect("eval sink csv failed");

        // Read back and verify
        let read_code = format!(r#"(read-csv "{}")"#, temp_output);
        let res = eval_string(read_code, &env, &mut state)
            .expect("eval read-csv failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[2i64],
            "name" => &["Bob"],
            "age" => &[30i64],
            "city" => &["Paris"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);

        let _ = std::fs::remove_file(temp_output);
    }

    #[test]
    fn test_sink_parquet_roundtrip() {
        let env = std_env();
        let mut state = State { line: 0, file: None };
        let temp_output = "test_sink_output.parquet";

        // Stream filtered lazy plan directly to Parquet
        let sink_code = format!(
            r#"(sink (filter (read-csv "data1.csv") (> $"age" 28)) "{}")"#,
            temp_output
        );

        eval_string(sink_code, &env, &mut state)
            .expect("eval sink parquet failed");

        // Read back from Parquet and verify
        let read_code = format!(r#"(read-parquet "{}")"#, temp_output);
        let res = eval_string(read_code, &env, &mut state)
            .expect("eval read-parquet failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[2i64, 5],
            "name" => &["Bob", "Evan"],
            "age" => &[30i64, 35],
            "city" => &["Paris", "Madrid"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);

        let _ = std::fs::remove_file(temp_output);
    }

    #[test]
    fn test_pivot_basic() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (sort (pivot (read-csv "data1.csv") $"city" $"name" $"age") $"name")
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval pivot failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "Berlin" => &[None, None, Some(22i64), None, None],
            "London" => &[Some(25i64), None, None, None, None],
            "Madrid" => &[None, None, None, None, Some(35i64)],
            "Paris" => &[None, Some(30i64), None, None, None],
            "Rome" => &[None, None, None, Some(28i64), None],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_pivot_with_explicit_agg_sum() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (sort (pivot (read-csv "data1.csv") $"city" $"name" $"age" :sum) $"name")
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval pivot with :sum failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        // :sum produces 0i64 for non-matching combinations instead of null
        let expected_df = df!(
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "Berlin" => &[0i64, 0, 22, 0, 0],
            "London" => &[25i64, 0, 0, 0, 0],
            "Madrid" => &[0i64, 0, 0, 0, 35],
            "Paris" => &[0i64, 30, 0, 0, 0],
            "Rome" => &[0i64, 0, 0, 28, 0],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_pivot_string_col_arguments() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Column names can be strings, but the subsequent sort must use a column expression $"name"
        let code = r#"
            (sort (pivot (read-csv "data1.csv") "city" "name" "age" :first) $"name")
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval pivot with string arguments failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "Berlin" => &[None, None, Some(22i64), None, None],
            "London" => &[Some(25i64), None, None, None, None],
            "Madrid" => &[None, None, None, None, Some(35i64)],
            "Paris" => &[None, Some(30i64), None, None, None],
            "Rome" => &[None, None, None, Some(28i64), None],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_pivot_pipeline_threading() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (-> (read-csv "data1.csv")
                (pivot $"city" $"id" $"age" :max)
                (sort $"id"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval pipeline pivot failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "Berlin" => &[None, None, Some(22i64), None, None],
            "London" => &[Some(25i64), None, None, None, None],
            "Madrid" => &[None, None, None, None, Some(35i64)],
            "Paris" => &[None, Some(30i64), None, None, None],
            "Rome" => &[None, None, None, Some(28i64), None],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_melt_basic() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (melt (read-csv "data1.csv") (list $"id") (list $"name" $"city"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval melt basic failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5, 1, 2, 3, 4, 5],
            "variable" => &[
                "name", "name", "name", "name", "name",
                "city", "city", "city", "city", "city",
            ],
            "value" => &[
                "Alice", "Bob", "Charlie", "Diana", "Evan",
                "London", "Paris", "Berlin", "Rome", "Madrid",
            ],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_melt_custom_variable_and_value_names() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (melt (read-csv "data1.csv") (list $"id" $"name") (list $"age") "metric" "val")
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval melt with custom column names failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "metric" => &["age", "age", "age", "age", "age"],
            "val" => &[25i64, 30, 22, 28, 35],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_melt_string_column_arguments() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (melt (read-csv "data1.csv") (list "id") (list "name" "city"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval melt with string arguments failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5, 1, 2, 3, 4, 5],
            "variable" => &[
                "name", "name", "name", "name", "name",
                "city", "city", "city", "city", "city",
            ],
            "value" => &[
                "Alice", "Bob", "Charlie", "Diana", "Evan",
                "London", "Paris", "Berlin", "Rome", "Madrid",
            ],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_unpivot_alias_and_pipeline() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (-> (read-csv "data1.csv")
                (unpivot (list $"id") (list $"name" $"city") "attr" "val")
                (filter (= $"attr" "city"))
                (sort $"id"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval unpivot pipeline failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "attr" => &["city", "city", "city", "city", "city"],
            "val" => &["London", "Paris", "Berlin", "Rome", "Madrid"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_concat_vertical_args() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Concatenate two filtered LazyFrames vertically via direct arguments
        let code = r#"
            (concat 
                (filter (read-csv "data1.csv") (< $"age" 25))
                (filter (read-csv "data1.csv") (> $"age" 30)))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval concat vertical args failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[3i64, 5],
            "name" => &["Charlie", "Evan"],
            "age" => &[22i64, 35],
            "city" => &["Berlin", "Madrid"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_concat_vertical_list() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Concatenate a list of LazyFrames vertically
        let code = r#"
            (concat (list 
                (filter (read-csv "data1.csv") (= $"id" 1))
                (filter (read-csv "data1.csv") (= $"id" 2))))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval concat list failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2],
            "name" => &["Alice", "Bob"],
            "age" => &[25i64, 30],
            "city" => &["London", "Paris"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_concat_horizontal() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Concatenate columns horizontally using the :horizontal strategy
        let code = r#"
            (concat 
                (select (read-csv "data1.csv") $"id" $"name")
                (select (read-csv "data1.csv") $"age" $"city")
                :horizontal)
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval concat horizontal failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "age" => &[25i64, 30, 22, 28, 35],
            "city" => &["London", "Paris", "Berlin", "Rome", "Madrid"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_concat_diagonal() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Diagonal concatenation of frames with non-matching schemas
        let code = r#"
            (concat 
                (select (read-csv "data1.csv") $"id" $"name")
                (select (read-csv "data1.csv") $"id" $"age")
                :diagonal)
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval concat diagonal failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5, 1, 2, 3, 4, 5],
            "name" => &[
                Some("Alice"), Some("Bob"), Some("Charlie"), Some("Diana"), Some("Evan"),
                None, None, None, None, None
            ],
            "age" => &[
                None, None, None, None, None,
                Some(25i64), Some(30), Some(22), Some(28), Some(35)
            ],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_scalar_int() {
        let env = std_env();
        let mut state = State { line: 0, file: None };
    
        // Aggregate to a 1x1 LazyFrame containing the sum of ages (140)
        let code = r#"
            (scalar (agg (read-csv "data1.csv") (sum $"age")))
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval scalar int failed")
            .expect_int()
            .expect("expected int result");
    
        assert_eq!(res, 140);
    }
    
    #[test]
    fn test_scalar_float() {
        let env = std_env();
        let mut state = State { line: 0, file: None };
    
        // Aggregate to a 1x1 LazyFrame containing the mean age (28.0)
        let code = r#"
            (scalar (agg (read-csv "data1.csv") (mean $"age")))
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval scalar float failed")
            .expect_float()
            .expect("expected float result");
    
        assert_eq!(res, 28.0);
    }
    
    #[test]
    fn test_scalar_string() {
        let env = std_env();
        let mut state = State { line: 0, file: None };
    
        // Filter to a single row and select 1 column to get a 1x1 String
        let code = r#"
            (scalar (select (filter (read-csv "data1.csv") (= $"id" 1)) $"name"))
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval scalar string failed")
            .expect_string()
            .expect("expected string result");
    
        assert_eq!(res, "Alice");
    }
    
    #[test]
    fn test_scalar_boolean() {
        let env = std_env();
        let mut state = State { line: 0, file: None };
    
        // Materialize a 1x1 boolean expression
        let code = r#"
            (scalar (select (filter (read-csv "data1.csv") (= $"id" 1)) (= $"name" "Alice")))
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval scalar boolean failed")
            .expect_symbol()
            .expect("expected symbol result");
    
        assert_eq!(res, "true");
    }
    
    #[test]
    fn test_scalar_shape_error() {
        let env = std_env();
        let mut state = State { line: 0, file: None };
    
        // Attempting scalar conversion on a non-1x1 LazyFrame should fail with ScalarShapeError
        let code = r#"
            (scalar (read-csv "data1.csv"))
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state);
        assert!(matches!(res, Err(Error::ScalarShapeError { .. })));
    }

    #[test]
    fn test_rolling_mean() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (with-column (select (read-csv "data1.csv") $"id" $"age") $"roll_mean" (rolling-mean $"age" 2))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval rolling-mean failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "age" => &[25i64, 30, 22, 28, 35],
            "roll_mean" => &[25.0f64, 27.5, 26.0, 25.0, 31.5],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_rolling_sum() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (with-column (select (read-csv "data1.csv") $"id" $"age") $"roll_sum" (rolling-sum $"age" 2))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval rolling-sum failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "age" => &[25i64, 30, 22, 28, 35],
            "roll_sum" => &[25i64, 55, 52, 50, 63],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_rolling_min_and_max() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (-> (select (read-csv "data1.csv") $"id" $"age")
                (with-column $"roll_min" (rolling-min $"age" 2))
                (with-column $"roll_max" (rolling-max $"age" 2)))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval rolling-min/max failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "age" => &[25i64, 30, 22, 28, 35],
            "roll_min" => &[25i64, 25, 22, 22, 28],
            "roll_max" => &[25i64, 30, 30, 28, 35],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_rolling_median() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (with-column (select (read-csv "data1.csv") $"id" $"age") $"roll_median" (rolling-median $"age" 3))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval rolling-median failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "age" => &[25i64, 30, 22, 28, 35],
            "roll_median" => &[25.0f64, 27.5, 25.0, 28.0, 28.0],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_str_split() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Split sku into list of parts separated by "-"
        let code = r#"
            (with-column (read-csv "data4.csv") $"sku_parts" (str-split $"sku" "-"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval with-column 'str-split' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let sku_parts: Series = Series::new(
            PlSmallStr::from_static("sku_parts"),
            &[
                Series::new(PlSmallStr::from_static(""), &["PROD", "101", "US"]),
                Series::new(PlSmallStr::from_static(""), &["DEV", "202", "EU"]),
                Series::new(PlSmallStr::from_static(""), &["PROD", "303", "APAC"]),
                Series::new(PlSmallStr::from_static(""), &["STAGING", "404", "US"]),
                Series::new(PlSmallStr::from_static(""), &["DEV", "505", "LATAM"]),
            ],
        );

        let mut expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice Johnson", "Bob Smith", "Charlie Brown", "Diana Prince", "Evan Wright"],
            "email" => &[
                "alice.j@example.com",
                "bob_smith@work.org",
                "charlie@test.net",
                "diana.p@example.com",
                "evan_w@domain.io",
            ],
            "sku" => &[
                "PROD-101-US",
                "DEV-202-EU",
                "PROD-303-APAC",
                "STAGING-404-US",
                "DEV-505-LATAM",
            ],
        ).expect("failed to construct expected DataFrame");

        expected_df.with_column(sku_parts.into()).unwrap();
        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_str_slice() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Extract first 4 characters of the SKU column
        let code = r#"
            (with-column (read-csv "data4.csv") $"prefix" (str-slice $"sku" 0 4))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval with-column 'str-slice' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice Johnson", "Bob Smith", "Charlie Brown", "Diana Prince", "Evan Wright"],
            "email" => &[
                "alice.j@example.com",
                "bob_smith@work.org",
                "charlie@test.net",
                "diana.p@example.com",
                "evan_w@domain.io",
            ],
            "sku" => &[
                "PROD-101-US",
                "DEV-202-EU",
                "PROD-303-APAC",
                "STAGING-404-US",
                "DEV-505-LATAM",
            ],
            "prefix" => &[
                Some("PROD"),
                Some("DEV-"),
                Some("PROD"),
                Some("STAG"),
                Some("DEV-"),
            ],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_str_strip() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Strip character set ("PRODLATAMUS-") from both ends of sku
        let code = r#"
            (with-column (read-csv "data4.csv") $"stripped" (str-strip $"sku" "PRODLATAMUS-"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval with-column 'str-strip' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice Johnson", "Bob Smith", "Charlie Brown", "Diana Prince", "Evan Wright"],
            "email" => &[
                "alice.j@example.com",
                "bob_smith@work.org",
                "charlie@test.net",
                "diana.p@example.com",
                "evan_w@domain.io",
            ],
            "sku" => &[
                "PROD-101-US",
                "DEV-202-EU",
                "PROD-303-APAC",
                "STAGING-404-US",
                "DEV-505-LATAM",
            ],
            "stripped" => &[
                Some("101"),
                Some("EV-202-E"),
                Some("303-APAC"),
                Some("GING-404"),
                Some("EV-505"),
            ],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_str_strip_start_and_end() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (-> (read-csv "data4.csv")
                (with-column $"no_lead" (str-strip-start $"sku" "PRODEV-"))
                (with-column $"no_trail" (str-strip-end $"sku" "-USEUAPACLATAM")))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval strip start/end failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice Johnson", "Bob Smith", "Charlie Brown", "Diana Prince", "Evan Wright"],
            "email" => &[
                "alice.j@example.com",
                "bob_smith@work.org",
                "charlie@test.net",
                "diana.p@example.com",
                "evan_w@domain.io",
            ],
            "sku" => &[
                "PROD-101-US",
                "DEV-202-EU",
                "PROD-303-APAC",
                "STAGING-404-US",
                "DEV-505-LATAM",
            ],
            "no_lead" => &[
                Some("101-US"),
                Some("202-EU"),
                Some("303-APAC"),
                Some("STAGING-404-US"),
                Some("505-LATAM"),
            ],
            "no_trail" => &[
                Some("PROD-101"),
                Some("DEV-202"),
                Some("PROD-303"),
                Some("STAGING-404"),
                Some("DEV-505"),
            ],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_str_extract_groups() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Extract named capture groups from SKU: env, code, and region
        let code = r#"
            (with-column (read-csv "data4.csv") $"sku_info" 
                         (str-extract-groups $"sku" "^(?<env>[A-Z]+)-(?<code>[0-9]+)-(?<region>[A-Z]+)$"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval with-column 'str-extract-groups' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let env_s = Column::new(PlSmallStr::from_static("env"), &["PROD", "DEV", "PROD", "STAGING", "DEV"]);
        let code_s = Column::new(PlSmallStr::from_static("code"), &["101", "202", "303", "404", "505"]);
        let region_s = Column::new(PlSmallStr::from_static("region"), &["US", "EU", "APAC", "US", "LATAM"]);

        let sku_info = StructChunked::from_columns(
            PlSmallStr::from_static("sku_info"),
            5,
            &[env_s, code_s, region_s],
        )
        .expect("failed to build struct column")
        .into_series();

        let mut expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice Johnson", "Bob Smith", "Charlie Brown", "Diana Prince", "Evan Wright"],
            "email" => &[
                "alice.j@example.com",
                "bob_smith@work.org",
                "charlie@test.net",
                "diana.p@example.com",
                "evan_w@domain.io",
            ],
            "sku" => &[
                "PROD-101-US",
                "DEV-202-EU",
                "PROD-303-APAC",
                "STAGING-404-US",
                "DEV-505-LATAM",
            ],
        ).expect("failed to construct expected DataFrame");

        expected_df.with_column(sku_info.into()).unwrap();
        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_sort_multi_column_mixed_orders() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Pass list expressions for columns and ordering symbols
        let code = r#"
            (sort (read-csv "data1.csv") (list $"city" $"age") '(:asc :desc))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval sort multi-column mixed failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[3i64, 1, 5, 2, 4],
            "name" => &["Charlie", "Alice", "Evan", "Bob", "Diana"],
            "age" => &[22i64, 25, 35, 30, 28],
            "city" => &["Berlin", "London", "Madrid", "Paris", "Rome"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_sort_multi_column_desc() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Pass list expression for columns and :desc symbol
        let code = r#"
            (sort (read-csv "data1.csv") (list $"age" $"id") :desc)
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval sort multi-column desc failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[5i64, 2, 4, 1, 3],
            "name" => &["Evan", "Bob", "Diana", "Alice", "Charlie"],
            "age" => &[35i64, 30, 28, 25, 22],
            "city" => &["Madrid", "Paris", "Rome", "London", "Berlin"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_slice_expr() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Explicit slice on column expression: start at offset 1, take 3 items
        let code = r#"
            (select (read-csv "data1.csv") (slice-expr $"name" 1 3))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval slice-expr failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "name" => &["Bob", "Charlie", "Diana"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_get_expr() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Index specific elements (first: 0, last: -1) using get
        let code = r#"
            (select (read-csv "data1.csv")
                    (alias (get $"name" 0) "first_name")
                    (alias (get $"name" -1) "last_name"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval get expr failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "first_name" => &["Alice"],
            "last_name" => &["Evan"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }
}
==> preprocessor_test.rs <==
#[cfg(test)]
mod tests {
    use crate::wisp::preprocess_wisp3;

    #[test]
    fn test1() {
        let code = 
r#"
a
"#;
        let expected = 
r#"
(a)
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test2() {
        let code = 
r#"
a b c
"#;
        let expected = 
r#"
(a b c)
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test3() {
        let code = 
r#"
a
    b
"#;
        let expected = 
r#"
(a
    (b))
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test4() {
        let code = 
r#"
a 1 2 3
    b 1 2 3
"#;
        let expected = 
r#"
(a 1 2 3
    (b 1 2 3))
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test5() {
        let code = 
r#"
a
    b
        c
"#;
        let expected = 
r#"
(a
    (b
        (c)))
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test6() {
        let code = 
r#"
a
    b
    c
"#;
        let expected = 
r#"
(a
    (b)
    (c))
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test7() {
        let code = 
r#"
a
    b
    c
    d
"#;
        let expected = 
r#"
(a
    (b)
    (c)
    (d))
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test8() {
        let code = 
r#"
a
    b
        c
    d
"#;
        let expected = 
r#"
(a
    (b
        (c))
    (d))
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test9() {
        let code = 
r#"
a
    b
c
"#;
        let expected = 
r#"
(a
    (b))
(c)
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test10() {
        let code = 
r#"
a
    b
        c
    d
e
"#;
        let expected = 
r#"
(a
    (b
        (c))
    (d))
(e)
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }


    #[test]
    fn test_colon1() {
        let code = 
r#"
a : b
"#;
        let expected = 
r#"
(a (b))
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

     #[test]
    fn test_colon2() {
        let code = 
r#"
a 1 2 : b 3 4
"#;
        let expected = 
r#"
(a 1 2 (b 3 4))
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_colon3() {
        let code = 
r#"
a : b : c
"#;
        let expected = 
r#"
(a (b (c)))
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_dot1() {
        let code = 
r#"
a
. b c
"#;
        let expected = 
r#"
(a)
(. b c)
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_dot2() {
        let code = 
r#"
1 2
. 3 4
"#;
        let expected = 
r#"
(1 2)
(. 3 4)
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_traditional_sexp1() {
         let code = 
r#"
(a)
"#;
        let expected = 
r#"
(a)
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_traditional_sexp2() {
         let code = 
r#"
(a
    b)
"#;
        let expected = 
r#"
(a
    b)
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }
}

==> stdlib.rs <==
use std::{collections::HashMap, f32::consts::E};
use std::fs::File;
use polars::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use crate::types::{Env, Error, Expr, ExprValue::{self, PolarsExpr}};
use crate::plotlarslib;
use crate::interpreter::{eval_string, State};
use polars::prelude::RollingOptionsFixedWindow;

pub fn nil() -> Expr { return Expr{ value: ExprValue::String("nil".to_string()), line: None } }

pub fn new_int(value: i64) -> Expr {
    Expr { value: ExprValue::Int(value), line: None }
}

pub fn new_float(value: f64) -> Expr {
    Expr { value: ExprValue::Float(value), line: None }
}

pub fn new_string(value: String) -> Expr {
    Expr { value: ExprValue::String(value), line: None }
}

pub fn new_list(value: Vec<Expr>) -> Expr {
    Expr { value: ExprValue::List(value), line: None }
}

pub fn new_polars_expr(value: polars::prelude::Expr) -> Expr {
    Expr { value: ExprValue::PolarsExpr(value), line: None }
}

pub fn new_lazy_frame(value: LazyFrame) -> Expr {
    Expr { value: ExprValue::LazyFrame(value), line: None }
}

pub fn new_symbol(value: String) -> Expr {
    Expr { value: ExprValue::Symbol(value), line: None }
}

fn print(args: Vec<Expr>) -> Result<Expr, Error> {
    for arg in args {
        println!("{}", arg.to_string()?);
    }
    Ok(nil())
}

fn list(args: Vec<Expr>) -> Result<Expr, Error> {
    Ok(new_list(args))
}

fn plus(args: Vec<Expr>) -> Result<Expr, Error> {
    let has_polars_expr_arg = args.iter().any(|arg| matches!(arg.value, ExprValue::PolarsExpr(_)));
    let has_float_arg = args.iter().any(|arg| matches!(arg.value, ExprValue::Float(_)));

    if has_polars_expr_arg {
        let mut expr = args[0].clone().expect_polars_expr()?;
        for arg in args.iter().skip(1) {
            expr = expr + arg.clone().expect_polars_expr()?;
        }
        Ok(new_polars_expr(expr))
    } else if has_float_arg {
        let mut sum = 0.0;
        for arg in args {
            sum += arg.expect_number()?.to_float();
        }
        Ok(new_float(sum))
    } else {
        let mut sum = 0;
        for arg in args {
            sum += arg.expect_number()?.to_int();
        }
        Ok(new_int(sum))
    }
}

fn minus(args: Vec<Expr>) -> Result<Expr, Error> {
    let has_polars_expr_arg = args.iter().any(|arg| matches!(arg.value, ExprValue::PolarsExpr(_)));
    let has_float_arg = args.iter().any(|arg| matches!(arg.value, ExprValue::Float(_)));

    if has_polars_expr_arg {
        let mut expr = args[0].clone().expect_polars_expr()?;
        for arg in args.iter().skip(1) {
            expr = expr - arg.clone().expect_polars_expr()?;
        }
        Ok(new_polars_expr(expr))
    } else if has_float_arg {
        let mut value = args[0].clone().expect_number()?.to_float();
        for arg in args.iter().skip(1) {
            value -= arg.clone().expect_number()?.to_float();
        }
        Ok(new_float(value))
    } else {
        let mut value = args[0].clone().expect_number()?.to_int();
        for arg in args.iter().skip(1) {
            value -= arg.clone().expect_number()?.to_int();
        }
        Ok(new_int(value))
    }
}

fn mult(args: Vec<Expr>) -> Result<Expr, Error> {
    let has_polars_expr_arg = args.iter().any(|arg| matches!(arg.value, ExprValue::PolarsExpr(_)));
    let has_float_arg = args.iter().any(|arg| matches!(arg.value, ExprValue::Float(_)));

    if has_polars_expr_arg {
        let mut expr = args[0].clone().expect_polars_expr()?;
        for arg in args.iter().skip(1) {
            expr = expr * arg.clone().expect_polars_expr()?;
        }
        Ok(new_polars_expr(expr))
    } else if has_float_arg {
        let mut value = args[0].clone().expect_number()?.to_float();
        for arg in args.iter().skip(1) {
            value *= arg.clone().expect_number()?.to_float();
        }
        Ok(new_float(value))
    } else {
        let mut value = args[0].clone().expect_number()?.to_int();
        for arg in args.iter().skip(1) {
            value *= arg.clone().expect_number()?.to_int();
        }
        Ok(new_int(value))
    }
}

fn divide(args: Vec<Expr>) -> Result<Expr, Error> {
    let has_polars_expr_arg = args.iter().any(|arg| matches!(arg.value, ExprValue::PolarsExpr(_)));
    // let has_float_arg = args.iter().any(|arg| matches!(arg, Expr::Float(_)));
    // Always compute real division in f64

    if has_polars_expr_arg {
        // Cast the initial expression to Float64 so division produces f64
        let mut expr = args[0].clone().expect_polars_expr()?;
        for arg in args.iter().skip(1) {
            let next_expr = arg.clone().expect_polars_expr()?;
            expr = expr / next_expr;
        }
        Ok(new_polars_expr(expr))
    } else {
        let mut value = args[0].clone().expect_number()?.to_float();
        for arg in args.iter().skip(1) {
            value /= arg.clone().expect_number()?.to_float();
        }
        Ok(new_float(value))
    }
}

fn read_csv(args: Vec<Expr>) -> Result<Expr, Error> {
    let path = args[0].clone().expect_string()?;
    match LazyCsvReader::new(path.as_str().into())
        .with_has_header(true)
        .finish() {
            Ok(lf) => Ok(new_lazy_frame(lf)),
            Err(err) => Err(Error::PolarsError(err)),
        }
}

fn write_csv(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let path = args[1].clone().expect_string()?;

    let mut df = lf.collect().map_err(Error::PolarsError)?;
    let mut file = File::create(&path).map_err(|err| {
        Error::PolarsError(PolarsError::ComputeError(
            format!("Failed to create file '{}': {}", path, err).into(),
        ))
    })?;

    CsvWriter::new(&mut file)
        .include_header(true)
        .finish(&mut df)
        .map_err(Error::PolarsError)?;

    Ok(nil()) 
}

// (sink df "path/to/output.parquet") or (sink df "path/to/output.csv")
fn sink(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let path = args[1].clone().expect_string()?;

    let format = if path.ends_with(".parquet") {
        FileWriteFormat::Parquet(Arc::new(ParquetWriteOptions::default()))
    } else if path.ends_with(".csv") {
        FileWriteFormat::Csv(CsvWriterOptions {
            include_header: true,
            ..Default::default()
        })
    } else {
        return Err(Error::PolarsError(PolarsError::ComputeError(
            format!("Unsupported sink file format for '{}'", path).into(),
        )));
    };

    let destination = SinkDestination::File {
        target: SinkTarget::Path(path.as_str().into()),
    };
    let sink_args = UnifiedSinkArgs::default();

    // lf.sink(...) returns a LazyFrame representing the sink task.
    // .collect() triggers the actual streaming execution to disk.
    lf.sink(destination, format, sink_args)
        .map_err(Error::PolarsError)?
        .collect()
        .map_err(|err| {
            Error::PolarsError(PolarsError::ComputeError(
                format!("sink failed for '{}': {}", path, err).into(),
            ))
        })?;

    Ok(nil())
}

// (read-parquet "path/to/data.parquet") -> LazyFrame
fn read_parquet(args: Vec<Expr>) -> Result<Expr, Error> {
    let path = args[0].clone().expect_string()?;
    match LazyFrame::scan_parquet(path.as_str().into(), ScanArgsParquet::default()) {
        Ok(lf) => Ok(new_lazy_frame(lf)),
        Err(err) => Err(Error::PolarsError(err)),
    }
}

// (write-parquet df "path/to/output.parquet") -> nil
fn write_parquet(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let path = args[1].clone().expect_string()?;

    let mut df = lf.collect().map_err(Error::PolarsError)?;
    let file = File::create(&path).map_err(|err| {
        Error::PolarsError(PolarsError::ComputeError(
            format!("Failed to create file '{}': {}", path, err).into(),
        ))
    })?;

    ParquetWriter::new(file)
        .finish(&mut df)
        .map_err(Error::PolarsError)?;

    Ok(nil())
}

fn head(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    Ok(new_lazy_frame(lf.limit(10)))
}

fn tail(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    Ok(new_lazy_frame(lf.slice(-10, 10)))
}

fn slice(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let offset = args[1].clone().expect_int()?;
    let len = args[2].clone().expect_int()?;
    Ok(new_lazy_frame(lf.slice(offset, len as u32)))
}

fn schema(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut lf = args[0].clone().expect_lazy_frame()?; 
    let schema = lf.collect_schema().unwrap();
    let mut result = Vec::<Expr>::new();
    let mut fields = Vec::<Expr>::new();
    for field in schema.iter_fields() {
        
        fields.push(new_list(vec![new_string(field.name.into_string()), new_string(field.dtype.to_string())]));
    }
    result.push(new_list(fields));
    Ok(new_list(result))
}

fn shape(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut lf = args[0].clone().expect_lazy_frame()?;  
    let shape = lf.collect().unwrap().shape();
    Ok(new_list(vec![new_int(shape.0 as i64), new_int(shape.1 as i64)]))
}

fn col_(args: Vec<Expr>) -> Result<Expr, Error> {
    Ok(new_polars_expr(col(args[0].clone().expect_string()?)))
}

fn lit_(args: Vec<Expr>) -> Result<Expr, Error> {
    match args[0].value.clone() {
        ExprValue::Int(num) => Ok(new_polars_expr(lit(num))),
        ExprValue::Float(num) => Ok(new_polars_expr(lit(num))),
        ExprValue::String(string) => Ok(new_polars_expr(lit(string))),
        _ => Err(Error::WrongType { expected: String::from("number"), got: args[0].clone() })
    }
}

fn eq(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.eq(lit)))
}

fn neq(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.neq(lit)))
}

fn gt(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.gt(lit)))
}

fn lt(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.lt(lit)))
}

fn gt_eq(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.gt_eq(lit)))
}

fn lt_eq(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.lt_eq(lit)))
}

fn not(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.not()))
}

fn and(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr1 = args[0].clone().expect_polars_expr()?;
    let expr2 = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr1.and(expr2)))
}

fn or(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr1 = args[0].clone().expect_polars_expr()?;
    let expr2 = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr1.or(expr2)))
}

fn sum(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.sum()))
}

fn mean(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.mean()))
}

fn median(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.median()))
}

fn count(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.is_empty() {
        // (count) without arguments returns the total number of rows
        Ok(new_polars_expr(polars::prelude::len()))
    } else {
        // (count $"col") returns non-null count for that column
        let expr = args[0].clone().expect_polars_expr()?;
        Ok(new_polars_expr(expr.count()))
    }
}

fn min(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.min()))
}

fn max(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.max()))
}

fn n_unique(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.n_unique()))
}

fn std_dev(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    // Default to sample standard deviation (ddof = 1); allow overriding via second argument
    let ddof = if args.len() > 1 {
        args[1].clone().expect_int()? as u8
    } else {
        1
    };
    Ok(new_polars_expr(expr.std(ddof)))
}

fn select(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let mut exprs = Vec::<polars::prelude::Expr>::new();
    for arg in args.iter().skip(1) {
        exprs.push(arg.clone().expect_polars_expr()?);
    }

    // Eagerly validate all expressions against the current schema
    lf.clone()
        .select(exprs.clone())
        .collect_schema()
        .map_err(Error::PolarsError)?;

    Ok(new_lazy_frame(lf.select(exprs)))
}

fn filter(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let expr = args[1].clone().expect_polars_expr()?;

    // Eagerly validate that all columns in `expr` exist in `lf`
    lf.clone()
        .select(vec![expr.clone()])
        .collect_schema()
        .map_err(Error::PolarsError)?;

    Ok(new_lazy_frame(lf.filter(expr)))
}

fn with_column(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let alias_expr = args[1].clone().expect_polars_expr()?;
    let value = args[2].clone().expect_polars_expr()?;
    let alias = match alias_expr {
        polars::prelude::Expr::Column(name) => name.to_string(),
        _ => {
            panic!("Shouldnt happen");
        }
    };

    // Eagerly validate `value`
    lf.clone()
        .select(vec![value.clone()])
        .collect_schema()
        .map_err(Error::PolarsError)?;

    Ok(new_lazy_frame(lf.with_column(value.alias(&alias))))
}

// --- 1. Multi-column Sorting with Mixed Orders ---
// Examples in Lisp:
//   (sort df $"a")
//   (sort df $"a" :desc)
//   (sort df ($"a" $"b") (:desc :asc))
//   (sort df ($"a" $"b") :desc) ; sets descending for all
fn sort(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;

    // 1. Extract by expressions
    let by_exprs: Vec<polars::prelude::Expr> = match &args[1].value {
        ExprValue::List(items) => {
            let mut exprs = Vec::with_capacity(items.len());
            for item in items {
                exprs.push(item.clone().expect_polars_expr()?);
            }
            exprs
        }
        _ => vec![args[1].clone().expect_polars_expr()?],
    };

    // 2. Extract descending flags
    let descending: Vec<bool> = if args.len() > 2 {
        match &args[2].value {
            ExprValue::List(items) => {
                let mut desc = Vec::with_capacity(items.len());
                for item in items {
                    let sym = item.clone().expect_symbol()?;
                    desc.push(matches!(sym.as_str(), ":desc" | ":descending" | "true"));
                }
                desc
            }
            _ => {
                let sym = args[2].clone().expect_symbol()?;
                let is_desc = matches!(sym.as_str(), ":desc" | ":descending" | "true");
                vec![is_desc; by_exprs.len()]
            }
        }
    } else {
        vec![false; by_exprs.len()]
    };

    if !descending.is_empty() && descending.len() != by_exprs.len() {
        return Err(Error::Custom(format!(
            "Number of sort orders ({}) does not match number of columns ({})",
            descending.len(),
            by_exprs.len()
        )));
    }

    let options = SortMultipleOptions::new().with_order_descending_multi(descending);
    Ok(new_lazy_frame(lf.sort_by_exprs(by_exprs, options)))
}

// (slice-expr $"col" offset length)
fn expr_slice(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let offset = args[1].clone().expect_int()?;
    let length = args[2].clone().expect_int()?;

    Ok(new_polars_expr(expr.slice(lit(offset), lit(length as u32))))
}

// (get $"col" 0) -> first element
// (get $"col" -1) -> last element
fn expr_get(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let index = args[1].clone().expect_int()?;

    // Set null_on_oob to true (returns null if out of bounds) or false (raises an error)
    Ok(new_polars_expr(expr.get(lit(index), true)))
}

fn group_by(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;

    // Support either a single grouping column or a list of grouping columns
    let group_keys = match &args[1].value {
        ExprValue::List(cols) => {
            let mut exprs = Vec::new();
            for col_expr in cols {
                exprs.push(col_expr.clone().expect_polars_expr()?);
            }
            exprs
        }
        _ => vec![args[1].clone().expect_polars_expr()?],
    };

    // Collect all trailing expressions as aggregations
    let mut agg_exprs = Vec::<polars::prelude::Expr>::new();
    for arg in args.iter().skip(2) {
        agg_exprs.push(arg.clone().expect_polars_expr()?);
    }

    Ok(new_lazy_frame(lf.group_by(group_keys).agg(agg_exprs)))
}

fn alias(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let name = args[1].clone().expect_string()?;
    Ok(new_polars_expr(expr.alias(&name)))
}

fn agg(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let mut exprs = Vec::<polars::prelude::Expr>::new();
    
    // Collect all expression arguments passed to agg
    for arg in args.iter().skip(1) {
        exprs.push(arg.clone().expect_polars_expr()?);
    }
    
    // Polars aggregates entire lazy frames via select
    Ok(new_lazy_frame(lf.select(exprs)))
}

fn join(args: Vec<Expr>) -> Result<Expr, Error> {
    let left_lf = args[0].clone().expect_lazy_frame()?;
    let right_lf = args[1].clone().expect_lazy_frame()?;
    
    // Parse left key(s)
    let left_on = match &args[2].value {
        ExprValue::List(cols) => {
            let mut exprs = Vec::new();
            for c in cols {
                exprs.push(c.clone().expect_polars_expr()?);
            }
            exprs
        }
        _ => vec![args[2].clone().expect_polars_expr()?],
    };

    // Parse right key(s)
    let right_on = match &args[3].value {
        ExprValue::List(cols) => {
            let mut exprs = Vec::new();
            for c in cols {
                exprs.push(c.clone().expect_polars_expr()?);
            }
            exprs
        }
        _ => vec![args[3].clone().expect_polars_expr()?],
    };

    // Determine join type (defaults to Inner)
    let how = if args.len() > 4 {
        match args[4].clone().expect_symbol()?.as_str() {
            ":left" => JoinType::Left,
            ":outer" | ":full" => JoinType::Full,
            ":cross" => JoinType::Cross,
            ":semi" => JoinType::Semi,
            ":anti" => JoinType::Anti,
            _ => JoinType::Inner,
        }
    } else {
        JoinType::Inner
    };

    let joined = left_lf.join(
        right_lf,
        left_on,
        right_on,
        JoinArgs::new(how),
    );

    Ok(new_lazy_frame(joined))
}

fn is_null_(args: Vec<Expr>) -> Result<Expr, Error> {
    let col = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(is_null(col)))
}

fn is_not_null_(args: Vec<Expr>) -> Result<Expr, Error> {
    let col = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(is_not_null(col)))
}

fn fill_null_(args: Vec<Expr>) -> Result<Expr, Error> {
    let col = args[0].clone().expect_polars_expr()?;
    let value = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(col.fill_null(value)))
}

fn drop_nulls(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    if args.len() == 1 {
        Ok(new_lazy_frame(lf.drop_nulls(None)))
    } else {
        let mut col_names = Vec::<String>::new();

        let mut extract_name = |expr: &Expr| -> Result<(), Error> {
            match expr.value.clone() {
                ExprValue::String(s) | ExprValue::Symbol(s) => {
                    col_names.push(s.clone());
                    Ok(())
                }
                ExprValue::PolarsExpr(polars::prelude::Expr::Column(name)) => {
                    col_names.push(name.to_string());
                    Ok(())
                }
                other => Err(Error::WrongType {
                    expected: String::from("column name or $col"),
                    got: expr.clone(),
                }),
            }
        };

        for arg in args.iter().skip(1) {
            match arg.value.clone() {
                ExprValue::List(list) => {
                    for item in list {
                        extract_name(&item)?;
                    }
                }
                _ => extract_name(arg)?,
            }
        }

        Ok(new_lazy_frame(lf.drop_nulls(Some(cols(col_names).into()))))
    }
}

fn when(args: Vec<Expr>) -> Result<Expr, Error> {
    let condition = args[0].clone().expect_polars_expr()?;
    let on_true = args[1].clone().expect_polars_expr()?;
    match args.len() {
        2 => Ok(new_polars_expr(polars::prelude::when(condition).then(on_true).otherwise(polars::prelude::lit(polars::prelude::NULL)))),
        _ => {
            let on_false = args[2].clone().expect_polars_expr()?;
            Ok(new_polars_expr(polars::prelude::when(condition).then(on_true).otherwise(on_false)))
        }
    }
}

// (str-contains $"name" "^A.*")
fn str_contains(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.len() < 2 {
        return Err(Error::WrongType {
            expected: "2 arguments (column, pattern)".into(),
            got: args[0].clone(),
        });
    }
    let col_expr = args[0].clone().expect_polars_expr()?;
    let pat_expr = args[1].clone().expect_polars_expr()?;
    
    // Polars: expr.str().contains(pattern, strict)
    Ok(new_polars_expr(col_expr.str().contains(pat_expr, true)))
}

// (str-replace $"city" "^Lon" "New-Lon")
fn str_replace(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let pat_expr = args[1].clone().expect_polars_expr()?;
    let rep_expr = args[2].clone().expect_polars_expr()?;

    // false = interpret pattern as regex, not literal
    Ok(new_polars_expr(col_expr.str().replace(pat_expr, rep_expr, false)))
}

// (str-extract $"code" "([A-Z]+)-(\\d+)" 1)
fn str_extract(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let pat_expr = args[1].clone().expect_polars_expr()?;
    let group_idx = args[2].clone().expect_int()? as usize;

    Ok(new_polars_expr(col_expr.str().extract(pat_expr, group_idx)))
}

// (over <expr> <partition_col1> ...) or (over <expr> (<col1> <col2>))
fn over(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;

    let partition_by = match &args[1].value {
        ExprValue::List(cols) => {
            let mut exprs = Vec::new();
            for col_expr in cols {
                exprs.push(col_expr.clone().expect_polars_expr()?);
            }
            exprs
        }
        _ => {
            let mut exprs = Vec::new();
            for arg in args.iter().skip(1) {
                exprs.push(arg.clone().expect_polars_expr()?);
            }
            exprs
        }
    };

    match expr.over(partition_by) {
        Ok(result) => Ok(new_polars_expr(result)),
        Err(err) => Err(Error::PolarsError(err)),
    } 
}

// (shift $"col" 1) or (shift $"col" -1)
fn shift(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let offset = args[1].clone().expect_int()? as i64;
    Ok(new_polars_expr(expr.shift(lit(offset))))
}

// (cum-sum $"col")
fn cum_sum(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let reverse = if args.len() > 1 {
        matches!(args[1].clone().expect_symbol()?.as_str(), ":desc" | ":reverse")
    } else {
        false
    };
    
    Ok(new_polars_expr(expr.cum_sum(reverse)))
}

// (cum-min $"col")
fn cum_min(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let reverse = if args.len() > 1 {
        matches!(args[1].clone().expect_symbol()?.as_str(), ":desc" | ":reverse")
    } else {
        false
    };
    Ok(new_polars_expr(expr.cum_min(reverse)))
}

// (cum-max $"col")
fn cum_max(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let reverse = if args.len() > 1 {
        matches!(args[1].clone().expect_symbol()?.as_str(), ":desc" | ":reverse")
    } else {
        false
    };
    Ok(new_polars_expr(expr.cum_max(reverse)))
}

// (cum-count $"col")
fn cum_count(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let reverse = if args.len() > 1 {
        matches!(args[1].clone().expect_symbol()?.as_str(), ":desc" | ":reverse")
    } else {
        false
    };
    Ok(new_polars_expr(expr.cum_count(reverse)))
}

// (rank $"col") or (rank $"col" :dense)
fn rank(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let method = if args.len() > 1 {
        match args[1].clone().expect_symbol()?.as_str() {
            ":dense" => RankMethod::Dense,
            ":min" => RankMethod::Min,
            ":max" => RankMethod::Max,
            ":average" => RankMethod::Average,
            ":ordinal" | ":first" => RankMethod::Ordinal,
            _ => RankMethod::Average,
        }
    } else {
        RankMethod::Average
    };

    let options = RankOptions {
        method,
        descending: false,
    };

    Ok(new_polars_expr(expr.rank(options, None)))
}

// (to-date $"col" "%Y-%m-%d") or (to-date $"col") -> ISO-8601 default
fn to_date(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let format = if args.len() > 1 {
        Some(args[1].clone().expect_string()?)
    } else {
        None
    };

    let options = StrptimeOptions {
        format: format.map(|s| PlSmallStr::from(s.as_str())),
        strict: true,
        exact: true,
        cache: true,
    };

    Ok(new_polars_expr(col_expr.str().to_date(options)))
} 

// (to-datetime $"col" "%Y-%m-%d %H:%M:%S")
fn to_datetime(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let format = if args.len() > 1 {
        Some(args[1].clone().expect_string()?)
    } else {
        None
    };

    let options = StrptimeOptions {
        format: format.map(|s| PlSmallStr::from(s.as_str())),
        strict: true,
        exact: true,
        cache: true,
    };

    // Defaults to Microseconds timeunit, no timezone
    Ok(new_polars_expr(col_expr.str().to_datetime(
        Some(TimeUnit::Microseconds),
        None,
        options,
        polars::prelude::lit("raise"),
    )))
}

// (dt-year $"date_col")
fn dt_year(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(col_expr.dt().year()))
}

// (dt-month $"date_col")
fn dt_month(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(col_expr.dt().month()))
}

// (dt-day $"date_col")
fn dt_day(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(col_expr.dt().day()))
}

// (dt-hour $"datetime_col")
fn dt_hour(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(col_expr.dt().hour()))
}

// (dt-format $"date_col" "%B %d, %Y")
fn dt_format(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let format = args[1].clone().expect_string()?;
    Ok(new_polars_expr(col_expr.dt().to_string(&format)))
}

// Helper to extract a column name string from String, Symbol, or $col Expr
fn extract_col_name(expr: &Expr) -> Result<String, Error> {
    match expr.value.clone() {
        ExprValue::String(s) | ExprValue::Symbol(s) => Ok(s.clone()),
        ExprValue::PolarsExpr(polars::prelude::Expr::Column(name)) => Ok(name.to_string()),
        other => Err(Error::WrongType {
            expected: String::from("column name or $col"),
            got: expr.clone(),
        }),
    }
}

// Helper to extract a Vec<String> from either a single item or Expr::List
fn extract_col_names(arg: &Expr) -> Result<Vec<String>, Error> {
    match arg.value.clone() {
        ExprValue::List(list) => list.iter().map(extract_col_name).collect(),
        _ => Ok(vec![extract_col_name(arg)?]),
    }
}

fn pivot_(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let on_cols = extract_col_names(&args[1])?;
    let index_cols = extract_col_names(&args[2])?;
    let values_cols = extract_col_names(&args[3])?;

    let agg_sym = if args.len() > 4 {
        args[4].clone().expect_symbol()?
    } else {
        ":first".to_string()
    };

    let agg_expr = match agg_sym.as_str() {
        ":sum" => polars::prelude::element().sum(),
        ":min" => polars::prelude::element().min(),
        ":max" => polars::prelude::element().max(),
        ":mean" | ":avg" => polars::prelude::element().mean(),
        ":median" => polars::prelude::element().median(),
        ":count" | ":len" => polars::prelude::len(),
        ":last" => polars::prelude::element().last(),
        ":first" | _ => polars::prelude::element().first(),
    };

    // 1. Select distinct on_cols and SORT them for deterministic column ordering
    let sort_options = SortMultipleOptions::new().with_order_descending(false);
    let pivot_values = Arc::new(
        lf.clone()
            .select([cols(on_cols.clone()).as_expr()])
            .unique(None, UniqueKeepStrategy::Any)
            .sort_by_exprs(vec![cols(on_cols.clone()).as_expr()], sort_options)
            .collect()
            .map_err(Error::PolarsError)?,
    );

    // 2. Pivot LazyFrame
    let pivoted = lf.pivot(
        cols(on_cols),
        pivot_values,
        cols(index_cols),
        cols(values_cols),
        agg_expr,
        false,
        PlSmallStr::from_static("_"),
        Default::default(),
    );

    Ok(new_lazy_frame(pivoted))
}

fn melt_(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let id_vars = extract_col_names(&args[1])?;
    let value_vars = extract_col_names(&args[2])?;

    let var_name = if args.len() > 3 {
        Some(PlSmallStr::from(args[3].clone().expect_string()?.as_str()))
    } else {
        None
    };

    let val_name = if args.len() > 4 {
        Some(PlSmallStr::from(args[4].clone().expect_string()?.as_str()))
    } else {
        None
    };

    let unpivot_args = UnpivotArgsDSL {
        index: cols(id_vars).into(),
        on: cols(value_vars).into(),
        variable_name: var_name,
        value_name: val_name,
    };

    Ok(new_lazy_frame(lf.unpivot(unpivot_args)))
}

fn concat_(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut frames = Vec::<LazyFrame>::new();
    let mut how_str: Option<String> = None;

    let items = match &args[0].value {
        ExprValue::List(list) => list.as_slice(),
        _ => &args[..],
    };

    for item in items {
        match item.value.clone() {
            ExprValue::LazyFrame(lf) => frames.push(lf.clone()),
            ExprValue::Symbol(s) if s.starts_with(':') => how_str = Some(s.clone()),
            _ => {
                if let Ok(lf) = item.clone().expect_lazy_frame() {
                    frames.push(lf);
                }
            }
        }
    }

    if args.len() > 1 && how_str.is_none() {
        if let Ok(sym) = args.last().unwrap().clone().expect_symbol() {
            if sym.starts_with(':') {
                how_str = Some(sym);
            }
        }
    }

    let concatted = match how_str.as_deref() {
        Some(":horizontal") | Some(":horiz") => {
            concat_lf_horizontal(frames, Default::default()).map_err(Error::PolarsError)?
        }
        Some(":diagonal") | Some(":diag") => {
            let mut union_args = UnionArgs::default();
            union_args.diagonal = true;
            polars::prelude::concat(frames, union_args).map_err(Error::PolarsError)?
        }
        _ => {
            // Default: Vertical concatenation
            polars::prelude::concat(frames, UnionArgs::default()).map_err(Error::PolarsError)?
        }
    };

    Ok(new_lazy_frame(concatted))
}

fn cast_(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let target_type = match args[1].clone().expect_symbol()?.as_str() {
        ":str" | ":string" => DataType::String,
        ":int" | ":i64" => DataType::Int64,
        ":float" | ":f64" => DataType::Float64,
        ":date" => DataType::Date,
        ":datetime" => DataType::Datetime(TimeUnit::Microseconds, None),
        ":bool" => DataType::Boolean,
        other => return Err(Error::Custom(format!("Unknown cast target: {}", other))),
    };
    Ok(new_polars_expr(col_expr.cast(target_type)))
}

// Helper to construct window options: (expr, window_size, [min_periods], [center])
fn parse_rolling_options(args: &[Expr]) -> Result<(polars::prelude::Expr, RollingOptionsFixedWindow), Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let window_size = args[1].clone().expect_int()? as usize;

    let min_periods = if args.len() > 2 {
        args[2].clone().expect_int()? as usize
    } else {
        1
    };

    let center = if args.len() > 3 {
        match args[3].clone().expect_symbol()?.as_str() {
            ":center" | "true" => true,
            _ => false,
        }
    } else {
        false
    };

    let options = RollingOptionsFixedWindow {
        window_size,
        min_periods,
        weights: None,
        center,
        fn_params: None,
    };

    Ok((expr, options))
}

// (rolling-mean $"col" 5) or (rolling-mean $"col" 5 1 :center)
fn rolling_mean(args: Vec<Expr>) -> Result<Expr, Error> {
    let (expr, opts) = parse_rolling_options(&args)?;
    Ok(new_polars_expr(expr.rolling_mean(opts)))
}

// (rolling-sum $"col" 5)
fn rolling_sum(args: Vec<Expr>) -> Result<Expr, Error> {
    let (expr, opts) = parse_rolling_options(&args)?;
    Ok(new_polars_expr(expr.rolling_sum(opts)))
}

// (rolling-min $"col" 5)
fn rolling_min(args: Vec<Expr>) -> Result<Expr, Error> {
    let (expr, opts) = parse_rolling_options(&args)?;
    Ok(new_polars_expr(expr.rolling_min(opts)))
}

// (rolling-max $"col" 5)
fn rolling_max(args: Vec<Expr>) -> Result<Expr, Error> {
    let (expr, opts) = parse_rolling_options(&args)?;
    Ok(new_polars_expr(expr.rolling_max(opts)))
}

// (rolling-std $"col" 5)
fn rolling_std(args: Vec<Expr>) -> Result<Expr, Error> {
    let (expr, opts) = parse_rolling_options(&args)?;
    Ok(new_polars_expr(expr.rolling_std(opts)))
}

// (rolling-median $"col" 5)
fn rolling_median(args: Vec<Expr>) -> Result<Expr, Error> {
    let (expr, opts) = parse_rolling_options(&args)?;
    Ok(new_polars_expr(expr.rolling_median(opts)))
}

// (str-split $"col" ",")
fn str_split(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let pat_expr = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(col_expr.str().split(pat_expr)))
}

// (str-strip $"col" [chars])
fn str_strip(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let expr = if args.len() > 1 {
        args[1].clone().expect_polars_expr()?
    } else {
        polars::prelude::lit(polars::prelude::NULL)
    };
    Ok(new_polars_expr(col_expr.str().strip_chars(expr)))
}

// (str-strip-start $"col" [chars])
fn str_strip_start(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let expr = if args.len() > 1 {
        args[1].clone().expect_polars_expr()?
    } else {
        polars::prelude::lit(polars::prelude::NULL)
    };
    Ok(new_polars_expr(col_expr.str().strip_chars_start(expr)))
}

// (str-strip-end $"col" [chars])
fn str_strip_end(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let expr = if args.len() > 1 {
        args[1].clone().expect_polars_expr()?
    } else {
        polars::prelude::lit(polars::prelude::NULL)
    };
    Ok(new_polars_expr(col_expr.str().strip_chars_end(expr)))
}

// (str-slice $"col" offset length)
fn str_slice(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let offset = args[1].clone().expect_int()?;
    let length = args[2].clone().expect_int()?;
    Ok(new_polars_expr(col_expr.str().slice(lit(offset), lit(length))))
}

// (str-extract-groups $"col" "([A-Z]+)-(\\d+)")
fn str_extract_groups(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let pat = args[1].clone().expect_string()?;
    
    let res = col_expr.str().extract_groups(&pat).map_err(Error::PolarsError)?;
    Ok(new_polars_expr(res))
}

// (scalar expr) — materialize a 1x1 result into a Lisp scalar
fn scalar(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let df = lf.collect().map_err(Error::PolarsError)?;
    if df.shape() != (1, 1) { return Err(Error::ScalarShapeError { shape: df.shape() }) }
    let row = df.get(0).unwrap();
    match row[0].clone() {
        AnyValue::Null => Ok(nil()),
        AnyValue::Boolean(b) => if b { Ok(new_symbol("true".to_string())) } else { Ok(new_symbol("false".to_string())) },
        AnyValue::String(s) => Ok(new_string(s.to_string())),
        AnyValue::UInt8(i) => Ok(new_int(i.into())),
        AnyValue::UInt16(i) => Ok(new_int(i.into())),
        AnyValue::UInt32(i) => Ok(new_int(i.into())),
        AnyValue::UInt64(i) => Ok(new_int(i as i64)),
        AnyValue::UInt128(i) => Ok(new_int(i as i64)),
        AnyValue::Int8(i) => Ok(new_int(i as i64)),
        AnyValue::Int16(i) => Ok(new_int(i as i64)),
        AnyValue::Int32(i) => Ok(new_int(i as i64)),
        AnyValue::Int64(i) => Ok(new_int(i as i64)),
        AnyValue::Int128(i) => Ok(new_int(i as i64)),
        AnyValue::Float16(f) => Ok(new_float(f.into())),
        AnyValue::Float32(f) => Ok(new_float(f.into())),
        AnyValue::Float64(f) => Ok(new_float(f.into())),
        AnyValue::Date(_) => Err(Error::ScalarUnsupportedError { name: "Date".to_string() }),
        AnyValue::Datetime(_, time_unit, time_zone) => Err(Error::ScalarUnsupportedError { name: "DateTime".to_string() }),
        AnyValue::DatetimeOwned(_, time_unit, time_zone) => Err(Error::ScalarUnsupportedError { name: "DatetimeOwned".to_string() }),
        AnyValue::Duration(_, time_unit) => Err(Error::ScalarUnsupportedError { name: "Duration".to_string() }),
        AnyValue::Time(_) => Err(Error::ScalarUnsupportedError { name: "Time".to_string() }),
        AnyValue::Categorical(_, categorical_mapping) => Err(Error::ScalarUnsupportedError { name: "Categorical".to_string() }),
        AnyValue::CategoricalOwned(_, categorical_mapping) => Err(Error::ScalarUnsupportedError { name: "CategoricalOwned".to_string() }),
        AnyValue::Enum(_, categorical_mapping) => Err(Error::ScalarUnsupportedError { name: "Enum".to_string() }),
        AnyValue::EnumOwned(_, categorical_mapping) => Err(Error::ScalarUnsupportedError { name: "EnumOwned".to_string() }),
        AnyValue::List(series) => Err(Error::ScalarUnsupportedError { name: "List".to_string() }),
        AnyValue::Struct(_, struct_array, fields) => Err(Error::ScalarUnsupportedError { name: "Struct".to_string() }),
        AnyValue::StructOwned(_) => Err(Error::ScalarUnsupportedError { name: "StructOwned".to_string() }),
        AnyValue::StringOwned(pl_small_str) => Ok(new_string(pl_small_str.to_string())),
        AnyValue::Binary(items) => Err(Error::ScalarUnsupportedError { name: "Binary".to_string() }),
        AnyValue::BinaryOwned(items) => Err(Error::ScalarUnsupportedError { name: "BinaryOwned".to_string() }),
        _ => Err(Error::ScalarUnsupportedError { name: "Unknown".to_string() })
    }
}

fn collect_(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    lf.collect_with_engine(polars::prelude::Engine::Streaming).unwrap();
    Ok(nil())
}

pub fn insert_native_function(defines: &mut HashMap<String, Expr>, name: &str, body: fn(Vec<Expr>) -> Result<Expr, Error>,
    min_args: u8, max_args: u8) {
        defines.insert(name.to_string(), Expr {value: ExprValue::NativeFunction { body, min_args, max_args }, line: None });
}

pub fn std_env() -> Rc<RefCell<Env>> {
    let mut defines = HashMap::new();

    // Constant symbol
    defines.insert("nil".to_string(), nil());

    // Native Functions: (defines, name, function_pointer, min_args, max_args)
    insert_native_function(&mut defines, "print", print, 0, u8::MAX);
    insert_native_function(&mut defines, "list", list, 0, u8::MAX);

    // Arithmetic
    insert_native_function(&mut defines, "+", plus, 1, u8::MAX);
    insert_native_function(&mut defines, "-", minus, 1, u8::MAX);
    insert_native_function(&mut defines, "*", mult, 1, u8::MAX);
    insert_native_function(&mut defines, "/", divide, 1, u8::MAX);

    // IO
    insert_native_function(&mut defines, "read-csv", read_csv, 1, 1);
    insert_native_function(&mut defines, "write-csv", write_csv, 2, 2);
    insert_native_function(&mut defines, "sink", sink, 2, 2);
    insert_native_function(&mut defines, "read-parquet", read_parquet, 1, 1);
    insert_native_function(&mut defines, "write-parquet", write_parquet, 2, 2);

    // Collect
    insert_native_function(&mut defines, "collect", collect_, 1, 1);

    // Inspection
    insert_native_function(&mut defines, "head", head, 1, 1);
    insert_native_function(&mut defines, "tail", tail, 1, 1);
    insert_native_function(&mut defines, "slice", slice, 3, 3);
    insert_native_function(&mut defines, "schema", schema, 1, 1);
    insert_native_function(&mut defines, "shape", shape, 1, 1);

    // Expressions & Literals
    insert_native_function(&mut defines, "col", col_, 1, 1);
    insert_native_function(&mut defines, "lit", lit_, 1, 1);

    // Comparison & Logical
    insert_native_function(&mut defines, "=", eq, 2, 2);
    insert_native_function(&mut defines, "!=", neq, 2, 2);
    insert_native_function(&mut defines, ">", gt, 2, 2);
    insert_native_function(&mut defines, "<", lt, 2, 2);
    insert_native_function(&mut defines, ">=", gt_eq, 2, 2);
    insert_native_function(&mut defines, "<=", lt_eq, 2, 2);
    insert_native_function(&mut defines, "not", not, 1, 1);
    insert_native_function(&mut defines, "and", and, 2, 2);
    insert_native_function(&mut defines, "or", or, 2, 2);

    // Aggregations
    insert_native_function(&mut defines, "sum", sum, 1, 1);
    insert_native_function(&mut defines, "mean", mean, 1, 1);
    insert_native_function(&mut defines, "median", median, 1, 1);
    insert_native_function(&mut defines, "count", count, 0, 1);
    insert_native_function(&mut defines, "min", min, 1, 1);
    insert_native_function(&mut defines, "max", max, 1, 1);
    insert_native_function(&mut defines, "n-unique", n_unique, 1, 1);
    insert_native_function(&mut defines, "std", std_dev, 1, 2);

    // Register Series slicing and indexing
    insert_native_function(&mut defines, "slice-expr", expr_slice, 3, 3);
    insert_native_function(&mut defines, "get", expr_get, 2, 2);

    // LazyFrame Operations
    insert_native_function(&mut defines, "select", select, 1, u8::MAX);
    insert_native_function(&mut defines, "filter", filter, 2, 2);
    insert_native_function(&mut defines, "with-column", with_column, 3, 3);
    insert_native_function(&mut defines, "sort", sort, 2, 3);
    insert_native_function(&mut defines, "group-by", group_by, 2, u8::MAX);
    insert_native_function(&mut defines, "alias", alias, 2, 2);
    insert_native_function(&mut defines, "agg", agg, 1, u8::MAX);
    insert_native_function(&mut defines, "join", join, 4, 5);

    // Null Handling & Conditionals
    insert_native_function(&mut defines, "null?", is_null_, 1, 1);
    insert_native_function(&mut defines, "not-null?", is_not_null_, 1, 1);
    insert_native_function(&mut defines, "fill-null", fill_null_, 2, 2);
    insert_native_function(&mut defines, "drop-nulls", drop_nulls, 1, u8::MAX);
    insert_native_function(&mut defines, "when", when, 2, 3);

    // String
    insert_native_function(&mut defines, "str-contains", str_contains, 2, 2);
    insert_native_function(&mut defines, "str-replace", str_replace, 3, 3);
    insert_native_function(&mut defines, "str-extract", str_extract, 3, 3);
    insert_native_function(&mut defines, "str-split", str_split, 2, 2);
    insert_native_function(&mut defines, "str-strip", str_strip, 1, 2);
    insert_native_function(&mut defines, "str-strip-start", str_strip_start, 1, 2);
    insert_native_function(&mut defines, "str-strip-end", str_strip_end, 1, 2);
    insert_native_function(&mut defines, "str-slice", str_slice, 3, 3);
    insert_native_function(&mut defines, "str-extract-groups", str_extract_groups, 2, 2);

    // Window & Cumulative Functions
    insert_native_function(&mut defines, "over", over, 2, u8::MAX);
    insert_native_function(&mut defines, "shift", shift, 2, 2);
    insert_native_function(&mut defines, "cum-sum", cum_sum, 1, 2);
    insert_native_function(&mut defines, "cum-min", cum_min, 1, 2);
    insert_native_function(&mut defines, "cum-max", cum_max, 1, 2);
    insert_native_function(&mut defines, "cum-count", cum_count, 1, 2);
    insert_native_function(&mut defines, "rank", rank, 1, 2);

    // Date and time
    insert_native_function(&mut defines, "to-date", to_date, 1, 2);
    insert_native_function(&mut defines, "to-datetime", to_datetime, 1, 2);
    insert_native_function(&mut defines, "dt-year", dt_year, 1, 1);
    insert_native_function(&mut defines, "dt-month", dt_month, 1, 1);
    insert_native_function(&mut defines, "dt-day", dt_day, 1, 1);
    insert_native_function(&mut defines, "dt-hour", dt_hour, 1, 1);
    insert_native_function(&mut defines, "dt-format", dt_format, 2, 2);

    // Reshaping & Set Operations
    insert_native_function(&mut defines, "pivot", pivot_, 4, 5);
    insert_native_function(&mut defines, "melt", melt_, 3, 5);
    insert_native_function(&mut defines, "unpivot", melt_, 3, 5);
    insert_native_function(&mut defines, "concat", concat_, 1, u8::MAX);

    // Rolling / Window aggregations
    insert_native_function(&mut defines, "rolling-mean", rolling_mean, 2, 4);
    insert_native_function(&mut defines, "rolling-sum", rolling_sum, 2, 4);
    insert_native_function(&mut defines, "rolling-min", rolling_min, 2, 4);
    insert_native_function(&mut defines, "rolling-max", rolling_max, 2, 4);
    insert_native_function(&mut defines, "rolling-median", rolling_median, 2, 4);
    insert_native_function(&mut defines, "rolling-std", rolling_std, 2, 5);

    // Casting
    insert_native_function(&mut defines, "cast", cast_, 2, 2);

    // Plotlars
    insert_native_function(&mut defines, "scatter-plot", plotlarslib::scatter_plot, 1, u8::MAX);
    insert_native_function(&mut defines, "bar-plot", plotlarslib::bar_plot, 1, u8::MAX);
    insert_native_function(&mut defines, "pie-plot", plotlarslib::pie_plot, 1, u8::MAX);
    insert_native_function(&mut defines, "histogram", plotlarslib::histogram, 1, u8::MAX);
    insert_native_function(&mut defines, "box-plot", plotlarslib::box_plot, 1, u8::MAX);
    insert_native_function(&mut defines, "time-series-plot", plotlarslib::time_series_plot, 1, u8::MAX);
    insert_native_function(&mut defines, "line-plot", plotlarslib::line_plot, 1, u8::MAX);

    // Polars to lisp
    insert_native_function(&mut defines, "scalar", scalar, 1, 1);

    Rc::new(RefCell::new(Env {
        defines,
        outer: None,
    }))
}

==> types.rs <==
use std::collections::HashMap;
use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;
use polars::prelude::*;

#[derive(Clone)]
pub struct Expr {
    pub value: ExprValue,
    pub line: Option<usize>
}

#[derive(Clone)]
pub enum ExprValue {
    Symbol(String),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Expr>),
    NativeFunction{ body: fn(Vec<Expr>) -> Result<Expr, Error>, min_args: u8, max_args: u8},
    UserFunction{ body: Vec<Expr>, params: Vec<String>, name: Option<String> },
    LazyFrame(LazyFrame),
    PolarsExpr(polars::prelude::Expr),
}

pub enum Number {
    Int(i64),
    Float(f64)
}

impl Expr {
     pub fn expect_symbol(self) -> Result<String, Error> {
        let Expr { value, line } = self;
        match value {
            ExprValue::Symbol(s) => Ok(s),
            other => Err(Error::WrongType {
                expected: String::from("symbol"),
                got: Expr { value: other, line },
            }),
        }
    }

    pub fn expect_number(self) -> Result<Number, Error> {
        let Expr { value, line } = self;
        match value {
            ExprValue::Int(n) => Ok(Number::Int(n)),
            ExprValue::Float(n) => Ok(Number::Float(n)),
            other => Err(Error::WrongType {
                expected: String::from("number"),
                got: Expr { value: other, line },
            }),
        }
    }

    pub fn expect_int(self) -> Result<i64, Error> {
        let Expr { value, line } = self;
        match value {
            ExprValue::Int(i) => Ok(i),
            other => Err(Error::WrongType {
                expected: String::from("int"),
                got: Expr { value: other, line },
            }),
        }
    }

    pub fn expect_float(self) -> Result<f64, Error> {
        let Expr { value, line } = self;
        match value {
            ExprValue::Float(f) => Ok(f),
            other => Err(Error::WrongType {
                expected: String::from("float"),
                got: Expr { value: other, line },
            }),
        }
    }

    pub fn expect_string(self) -> Result<String, Error> {
        let Expr { value, line } = self;
        match value {
            ExprValue::String(s) => Ok(s),
            other => Err(Error::WrongType {
                expected: String::from("string"),
                got: Expr { value: other, line },
            }),
        }
    }

    pub fn expect_list(self) -> Result<Vec<Expr>, Error> {
        let Expr { value, line } = self;
        match value {
            ExprValue::List(l) => Ok(l),
            other => Err(Error::WrongType {
                expected: String::from("list"),
                got: Expr { value: other, line },
            }),
        }
    }

    pub fn expect_lazy_frame(self) -> Result<LazyFrame, Error> {
        let Expr { value, line } = self;
        match value {
            ExprValue::LazyFrame(lf) => Ok(lf),
            other => Err(Error::WrongType {
                expected: String::from("lazy frame"),
                got: Expr { value: other, line },
            }),
        }
    }

    pub fn expect_polars_expr(self) -> Result<polars::prelude::Expr, Error> {
        let Expr { value, line } = self;
        match value {
            ExprValue::PolarsExpr(pe) => Ok(pe),
            ExprValue::Int(i) => Ok(polars::prelude::lit(i)),
            ExprValue::Float(f) => Ok(polars::prelude::lit(f)),
            ExprValue::String(str) => Ok(polars::prelude::lit(str)),
            other => Err(Error::WrongType {
                expected: String::from("polars expr"),
                got: Expr { value: other, line },
            }),
        }
    }

    pub fn type_to_str(self) -> &'static str {
        match self.value {
            ExprValue::Symbol(_) => "symbol",
            ExprValue::Int(_) => "int",
            ExprValue::Float(_) => "float",
            ExprValue::String(_) => "string",
            ExprValue::List(_) => "list",
            ExprValue::NativeFunction { body, min_args, max_args } => "native function",
            ExprValue::UserFunction { body, params, name } => "user function",
            ExprValue::LazyFrame(_) => "lazy frame",
            ExprValue::PolarsExpr(_) => "polars expr",
        }
    }

    pub fn to_string(self) -> Result<String, Error> {
        match self.value {
            ExprValue::LazyFrame(lf) => {
                let df = lf.clone().collect().map_err(Error::PolarsError)?;
                Ok(format!("{}", df))
            }
            // For others, fall back to standard Display or format manually
            other => Ok(format!("{}", other)),
        }
    }
}

impl Number {
    pub fn to_int(self) -> i64 {
        match self {
            Number::Int(i) => i,
            Number::Float(f) => f as i64,
        }
    }

    pub fn to_float(self) -> f64 {
        match self {
            Number::Int(i) => i as f64,
            Number::Float(f) => f,
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl fmt::Display for ExprValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprValue::Symbol(s) => write!(f, "{s}"),
            ExprValue::Int(n) => write!(f, "{n}"),
            ExprValue::Float(n) => write!(f, "{n}"),
            ExprValue::String(s) => write!(f, "{s}"),
            ExprValue::List(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item.value)?;
                }
                write!(f, ")")
            },
            ExprValue::NativeFunction{ body, min_args, max_args } => write!(f, "<NativeFunction>"),
            ExprValue::UserFunction { body, params, name } => {
                match name {
                    Some(name) => write!(f, "<{name}>"),
                    None => write!(f, "<lambda>"),
                }
            }
            ExprValue::LazyFrame(lf) => write!(f, "<LazyFrame>"),
            ExprValue::PolarsExpr(pe) => write!(f, "{}", pe),
        }
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Expr")
            .field("value", &self.value)
            .field("line", &self.line)
            .finish()
    }
}

impl fmt::Debug for ExprValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprValue::Symbol(s) => f.debug_tuple("Symbol").field(s).finish(),
            ExprValue::Int(n) => f.debug_tuple("Number").field(n).finish(),
            ExprValue::Float(n) => f.debug_tuple("Number").field(n).finish(),
            ExprValue::String(s) => f.debug_tuple("String").field(s).finish(),
            ExprValue::List(items) => f.debug_tuple("List").field(items).finish(),
            ExprValue::NativeFunction{ body, min_args, max_args } => { f.write_str("NativeFunction") },
            ExprValue::UserFunction { body, params, name } => { f.write_str("UserFunction") },
            ExprValue::LazyFrame(_) => { f.write_str("LazyFrame") },
            ExprValue::PolarsExpr(_) => { f.write_str("PolarsExpr") },
        }
    }
}

pub struct Env {
    pub defines: HashMap<String, Expr>,
    pub outer: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            defines: HashMap::new(),
            outer: None,
        }
    }

    pub fn with_parent(outer: Rc<RefCell<Env>>) -> Self {
        Self {
            defines: HashMap::new(),
            outer: Some(outer),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Error {
    UnclosedString,
    UnclosedListExpr,
    FailedToParseNumber {
        number: String
    },
    WrongType {
        expected: String,
        got: Expr
    },
    LookupNotFound {
        symbol: String
    },
    WrongNumberOfArguments {
        expected_min: u8,
        expected_max: u8,
        got: u8
    },
    ScalarShapeError{
        shape: (usize, usize)
    },
    ScalarUnsupportedError{
        name: String
    },
    MissingRequiredKWArg{
        name: String
    },
    Custom(String),
    PolarsError(polars::prelude::PolarsError)
}

==> wisp.rs <==
use crate::types::Error;

pub fn preprocess_wisp_____(input: &str) -> Result<String, Error> {
    let mut result = String::new();
    let mut indent_stack: Vec<usize> = Vec::new();
    let mut paren_depth: usize = 0;
    let mut bracket_depth: usize = 0;
    let mut brace_depth: usize = 0;
    let mut in_string = false;
    let mut consecutive_empty_lines = 0;

    for raw_line in input.lines() {
        let line_without_comment = strip_comments(raw_line);
        let trimmed = line_without_comment.trim();

        if trimmed.is_empty() {
            consecutive_empty_lines += 1;
            if consecutive_empty_lines >= 2 {
                while indent_stack.pop().is_some() {
                    result.push_str(") ");
                }
            }
            result.push('\n');
            continue;
        }
        consecutive_empty_lines = 0;

        // If inside an open paren/bracket/brace or multiline string, pass through directly
        if in_string || paren_depth > 0 || bracket_depth > 0 || brace_depth > 0 {
            update_delimiters(
                trimmed,
                &mut in_string,
                &mut paren_depth,
                &mut bracket_depth,
                &mut brace_depth,
            );
            result.push_str(trimmed);
            result.push('\n');
            continue;
        }

        // Standard parenthesized S-expression at top-level: bypass indent stack
        if trimmed.starts_with('(') || trimmed.starts_with('[') || trimmed.starts_with('{') {
            while indent_stack.pop().is_some() {
                result.push_str(") ");
            }
            update_delimiters(
                trimmed,
                &mut in_string,
                &mut paren_depth,
                &mut bracket_depth,
                &mut brace_depth,
            );
            result.push_str(trimmed);
            result.push('\n');
            continue;
        }

        let indent = raw_line
            .chars()
            .take_while(|c| *c == ' ' || *c == '\t')
            .map(|c| if c == '\t' { 4 } else { 1 })
            .sum::<usize>();

        let is_dot_continuation = trimmed == "." || trimmed.starts_with(". ");
        let is_colon_only = trimmed == ":";

        while let Some(&top) = indent_stack.last() {
            if indent < top {
                indent_stack.pop();
                result.push_str(") ");
            } else {
                break;
            }
        }

        let content_to_process = if is_dot_continuation {
            if trimmed == "." { "" } else { &trimmed[2..] }
        } else if is_colon_only {
            ""
        } else {
            trimmed
        };

        if is_dot_continuation {
            // A "." line continues the *enclosing* form. The previous sibling
            // at this indent (e.g. `cast …`) auto-opened a "(" — close it (and
            // anything else at this level) so the content lands in the parent.
            while let Some(&top) = indent_stack.last() {
                if top >= indent {
                    indent_stack.pop();
                    result.push_str(") ");
                } else {
                    break;
                }
            }
            let expanded = expand_inline_colons(content_to_process);
            result.push_str(&expanded);
        } else if is_colon_only {
            indent_stack.push(indent);
            result.push_str("(");
        } else {
            if indent_stack.last().map_or(true, |&top| indent > top) {
                indent_stack.push(indent);
                result.push_str("(");
            } else {
                result.push_str(") (");
            }
            let expanded = expand_inline_colons(content_to_process);
            result.push_str(&expanded);
        }

        update_delimiters(
            content_to_process,
            &mut in_string,
            &mut paren_depth,
            &mut bracket_depth,
            &mut brace_depth,
        );
        result.push('\n');
    }

    while indent_stack.pop().is_some() {
        result.push_str(") ");
    }

    Ok(result)
}

fn strip_comments(s: &str) -> &str {
    let mut in_str = false;
    let mut escape = false;
    for (i, c) in s.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if c == '\\' {
            escape = true;
        } else if c == '"' {
            in_str = !in_str;
        } else if c == ';' && !in_str {
            return &s[..i];
        }
    }
    s
}

fn update_delimiters(
    s: &str,
    in_string: &mut bool,
    paren_depth: &mut usize,
    bracket_depth: &mut usize,
    brace_depth: &mut usize,
) {
    let mut escape = false;
    for c in s.chars() {
        if escape {
            escape = false;
            continue;
        }
        if c == '\\' {
            escape = true;
            continue;
        }
        if c == '"' {
            *in_string = !*in_string;
        } else if !*in_string {
            match c {
                '(' => *paren_depth += 1,
                ')' => *paren_depth = paren_depth.saturating_sub(1),
                '[' => *bracket_depth += 1,
                ']' => *bracket_depth = bracket_depth.saturating_sub(1),
                '{' => *brace_depth += 1,
                '}' => *brace_depth = brace_depth.saturating_sub(1),
                _ => {}
            }
        }
    }
}

fn expand_inline_colons(line: &str) -> String {
    let mut trimmed = line.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut prefix_parens = 0;
    if trimmed.starts_with(": ") {
        prefix_parens += 1;
        trimmed = &trimmed[2..];
    } else if trimmed == ":" {
        return "()".to_string();
    }

    let mut ends_with_empty_colon = false;
    if trimmed.ends_with(" :") {
        ends_with_empty_colon = true;
        trimmed = &trimmed[..trimmed.len() - 2];
    }

    let parts: Vec<&str> = trimmed.split(" : ").collect();
    let inline_colons = parts.len().saturating_sub(1);

    let mut out = "(".repeat(prefix_parens);
    out.push_str(parts[0]);

    for part in &parts[1..] {
        out.push_str(" (");
        out.push_str(part);
    }

    if ends_with_empty_colon {
        out.push_str(" ()");
    }

    out.push_str(&")".repeat(inline_colons + prefix_parens));
    out.replace("\\:", ":")
}

fn count_indent_level(s: &String) -> usize {
    s.chars().take_while(|&c| c == ' ').count()
}

fn parent(i: usize, lines: &Vec::<String>) -> Option<usize> {
    for j in i..0 {
        if count_indent_level(&lines[j]) == count_indent_level(&lines[i]) {
            return Some(j)
        }
    }
    None
}

pub fn preprocess_wisp2_____(input: &str) -> Result<String, Error> {
    let mut lines: Vec::<String> = input.lines().map(String::from).collect();
    lines.insert(0, "".to_string());
    lines.push("".to_string());

    let mut unclosed_parens = 0;
    for i in 1..lines.len() {
        let indent_level = count_indent_level(&lines[i]);
        println!("{}", indent_level);

        if lines[i].trim().starts_with('(') || lines[i - 1].trim().starts_with('(') {
            continue;
        }

        if lines[i].trim().is_empty() {
            for _ in 0..unclosed_parens {
                lines[i-1].push(')');
            }
            unclosed_parens = 0;
        }

        if !lines[i].trim().is_empty() {
            lines[i].insert(indent_level, '(');
            lines[i].push(')');
        }

        if count_indent_level(&lines[i-1]) < indent_level {
            lines[i-1].pop();
            unclosed_parens += 1;
        }

        if count_indent_level(&lines[i-1]) > indent_level {
            lines[i-1].push(')');
        }
    }

    Ok(lines.join("\n"))
}

fn indent(line: &str) -> usize {
    if line.trim().is_empty() { 0 }
    else { line.len() - line.trim_start_matches(' ').len() }
}

pub fn preprocess_wisp3(input: &str) -> Result<String, Error> {
    let mut lines: Vec::<String> = input.lines().map(String::from).collect();
    lines.push("".to_string());
    let mut i = 0;

    while i != lines.len() {
        // empty line
        if lines[i].trim().is_empty() { i += 1; continue; }

        // traditional s exp
        if lines[i].trim().starts_with('(') {
            while !lines[i].trim().is_empty() {
                i += 1;
            }
            continue;
        }

        // add (
        let cur_indent = indent(&lines[i]);
        lines[i].insert(cur_indent, '(');

        // handle :
        if lines[i].contains(" : ") {
            for _ in 0..lines[i].matches(" : ").count() {
                lines[i].push(')');
            }
            lines[i] = lines[i].replace(" : ", " (");
        }

        if indent(&lines[i]) == indent(&lines[i + 1]) {
            lines[i].push(')');
        } else if indent(&lines[i]) > indent(&lines[i + 1]) {
            let diff = (indent(&lines[i]) / 4) - (indent(&lines[i + 1]) / 4);
            lines[i].push(')'); 
            for _ in 0..diff { lines[i].push(')'); }
        }

        i += 1;
    }

    Ok(lines.join("\n"))
}
