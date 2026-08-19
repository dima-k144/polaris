#[cfg(test)]
mod tests {
    use crate::interpreter::{eval_string, State};
    use crate::stdlib::std_env;
    use polars::prelude::*;

    #[test]
    fn test_filter_eq() {
        let env = std_env();
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };
    
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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };
    
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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };
    
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
        let mut state = State { line: 0 };
    
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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

        let res = eval_string("(+ 1 2 3 4)".to_string(), &env, &mut state)
            .expect("eval failed")
            .expect_number()
            .expect("expected number");

        assert_eq!(res, 10.0);
    }

    #[test]
    fn test_arithmetic_minus() {
        let env = std_env();
        let mut state = State { line: 0 };

        let res = eval_string("(- 20 5 3 2)".to_string(), &env, &mut state)
            .expect("eval failed")
            .expect_number()
            .expect("expected number");

        assert_eq!(res, 10.0);
    }

    #[test]
    fn test_arithmetic_mult() {
        let env = std_env();
        let mut state = State { line: 0 };

        let res = eval_string("(* 2 3 4)".to_string(), &env, &mut state)
            .expect("eval failed")
            .expect_number()
            .expect("expected number");

        assert_eq!(res, 24.0);
    }

    #[test]
    fn test_arithmetic_divide() {
        let env = std_env();
        let mut state = State { line: 0 };

        let res = eval_string("(/ 100 2 5)".to_string(), &env, &mut state)
            .expect("eval failed")
            .expect_number()
            .expect("expected number");

        assert_eq!(res, 10.0);
    }

    #[test]
    fn test_arithmetic_nested() {
        let env = std_env();
        let mut state = State { line: 0 };

        // Evaluates: ((10 + 5) * 2) - (20 / 4) = (15 * 2) - 5 = 30 - 5 = 25
        let code = "(- (* (+ 10 5) 2) (/ 20 4))".to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval failed")
            .expect_number()
            .expect("expected number");

        assert_eq!(res, 25.0);
    }

    #[test]
    fn test_polars_plus() {
        let env = std_env();
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };
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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };

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
        let mut state = State { line: 0 };
    
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
}