#[cfg(test)]
mod tests {
    use crate::interpreter::{eval_string, State};
    use crate::stdlib::std_env;
    use polars::prelude::*;

    #[test]
    fn test_filter_eq() {
        let mut env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (filter (read-csv "data1.csv") (= $"age" 30))
        "#.to_string();

        let res = eval_string(code, &mut env, &mut state)
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
        let mut env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (filter (read-csv "data1.csv") (!= $"age" 30))
        "#.to_string();

        let res = eval_string(code, &mut env, &mut state)
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
        let mut env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (filter (read-csv "data1.csv") (> $"age" 25))
        "#.to_string();

        let res = eval_string(code, &mut env, &mut state)
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
        let mut env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (filter (read-csv "data1.csv") (< $"age" 28))
        "#.to_string();

        let res = eval_string(code, &mut env, &mut state)
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
        let mut env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (filter (read-csv "data1.csv") (>= $"age" 28))
        "#.to_string();

        let res = eval_string(code, &mut env, &mut state)
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
        let mut env = std_env();
        let mut state = State { line: 0 };
    
        let code = r#"
            (filter (read-csv "data1.csv") (<= $"age" 28))
        "#.to_string();
    
        let res = eval_string(code, &mut env, &mut state)
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
        let mut env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (filter (read-csv "data1.csv") (and (> $"age" 22) (< $"age" 35)))
        "#.to_string();

        let res = eval_string(code, &mut env, &mut state)
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
        let mut env = std_env();
        let mut state = State { line: 0 };
    
        let code = r#"
            (filter (read-csv "data1.csv") (or (< $"age" 25) (>= $"age" 35)))
        "#.to_string();
    
        let res = eval_string(code, &mut env, &mut state)
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
        let mut env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (filter (read-csv "data1.csv") (not (> $"age" 25)))
        "#.to_string();

        let res = eval_string(code, &mut env, &mut state)
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
        let mut env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (select (read-csv "data1.csv") (sum $"age"))
        "#.to_string();

        let res = eval_string(code, &mut env, &mut state)
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
        let mut env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (select (read-csv "data1.csv") (mean $"age"))
        "#.to_string();

        let res = eval_string(code, &mut env, &mut state)
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
        let mut env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (select (read-csv "data1.csv") (median $"age"))
        "#.to_string();

        let res = eval_string(code, &mut env, &mut state)
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
        let mut env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (with-column (read-csv "data1.csv") $"is_over_25" (> $"age" 25))
        "#.to_string();

        let res = eval_string(code, &mut env, &mut state)
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
        let mut env = std_env();
        let mut state = State { line: 0 };
    
        let code = r#"
            (sort (read-csv "data1.csv") $"age")
        "#.to_string();
    
        let res = eval_string(code, &mut env, &mut state)
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
    
        let mut env = std_env();
        let mut state = State { line: 0 };
    
        // Group by city and compute mean age, then sort by city to ensure deterministic order
        let code = r#"
            (sort (group-by (read-csv "data1.csv") $"city" (mean $"age")) $"city")
        "#.to_string();
    
        let res = eval_string(code, &mut env, &mut state)
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
        let mut env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (agg (read-csv "data1.csv") 
                 (alias (sum $"age") "total_age") 
                 (alias (mean $"age") "mean_age"))
        "#.to_string();

        let res = eval_string(code, &mut env, &mut state)
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
        let mut env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (sort (join (read-csv "data1.csv") (read-csv "data2.csv") $"id" $"id" :inner) $"id")
        "#.to_string();

        let res = eval_string(code, &mut env, &mut state)
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
        let mut env = std_env();
        let mut state = State { line: 0 };

        let res = eval_string("(+ 1 2 3 4)".to_string(), &mut env, &mut state)
            .expect("eval failed")
            .expect_number()
            .expect("expected number");

        assert_eq!(res, 10.0);
    }

    #[test]
    fn test_arithmetic_minus() {
        let mut env = std_env();
        let mut state = State { line: 0 };

        let res = eval_string("(- 20 5 3 2)".to_string(), &mut env, &mut state)
            .expect("eval failed")
            .expect_number()
            .expect("expected number");

        assert_eq!(res, 10.0);
    }

    #[test]
    fn test_arithmetic_mult() {
        let mut env = std_env();
        let mut state = State { line: 0 };

        let res = eval_string("(* 2 3 4)".to_string(), &mut env, &mut state)
            .expect("eval failed")
            .expect_number()
            .expect("expected number");

        assert_eq!(res, 24.0);
    }

    #[test]
    fn test_arithmetic_divide() {
        let mut env = std_env();
        let mut state = State { line: 0 };

        let res = eval_string("(/ 100 2 5)".to_string(), &mut env, &mut state)
            .expect("eval failed")
            .expect_number()
            .expect("expected number");

        assert_eq!(res, 10.0);
    }

    #[test]
    fn test_arithmetic_nested() {
        let mut env = std_env();
        let mut state = State { line: 0 };

        // Evaluates: ((10 + 5) * 2) - (20 / 4) = (15 * 2) - 5 = 30 - 5 = 25
        let code = "(- (* (+ 10 5) 2) (/ 20 4))".to_string();

        let res = eval_string(code, &mut env, &mut state)
            .expect("eval failed")
            .expect_number()
            .expect("expected number");

        assert_eq!(res, 25.0);
    }

    #[test]
    fn test_polars_plus() {
        let mut env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (with-column (select (read-csv "data1.csv") $"id" $"age") $"age_plus_5" (+ $"age" 5))
        "#.to_string();

        let res = eval_string(code, &mut env, &mut state)
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
        let mut env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (with-column (select (read-csv "data1.csv") $"id" $"age") $"age_minus_2" (- $"age" 2))
        "#.to_string();

        let res = eval_string(code, &mut env, &mut state)
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
        let mut env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (with-column (select (read-csv "data1.csv") $"id" $"age") $"age_double" (* $"age" 2))
        "#.to_string();

        let res = eval_string(code, &mut env, &mut state)
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
        let mut env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (with-column (select (read-csv "data1.csv") $"id" $"age") $"age_half" (/ $"age" 2))
        "#.to_string();

        let res = eval_string(code, &mut env, &mut state)
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
}