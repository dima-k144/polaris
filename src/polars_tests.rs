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
            (filter (read-csv "data1.csv") (= $age 30))
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
            (filter (read-csv "data1.csv") (!= $age 30))
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
            (filter (read-csv "data1.csv") (> $age 25))
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
            (filter (read-csv "data1.csv") (< $age 28))
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
            (filter (read-csv "data1.csv") (>= $age 28))
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
            (filter (read-csv "data1.csv") (<= $age 28))
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
            (filter (read-csv "data1.csv") (and (> $age 22) (< $age 35)))
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
            (filter (read-csv "data1.csv") (or (< $age 25) (>= $age 35)))
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
            (filter (read-csv "data1.csv") (not (> $age 25)))
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
            (select (read-csv "data1.csv") (sum $age))
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
            (select (read-csv "data1.csv") (mean $age))
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
            (select (read-csv "data1.csv") (median $age))
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
            (with-column (read-csv "data1.csv") $is_over_25 (> $age 25))
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
            (sort (read-csv "data1.csv") $age)
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
            (sort (group-by (read-csv "data1.csv") $city (mean $age)) $city)
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
                 (alias (sum $age) "total_age") 
                 (alias (mean $age) "mean_age"))
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
            (sort (join (read-csv "data1.csv") (read-csv "data2.csv") $id $id :inner) $id)
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
            (with-column (select (read-csv "data1.csv") $id $age) $age_plus_5 (+ $age 5))
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
            (with-column (select (read-csv "data1.csv") $id $age) $age_minus_2 (- $age 2))
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
            (with-column (select (read-csv "data1.csv") $id $age) $age_double (* $age 2))
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
            (with-column (select (read-csv "data1.csv") $id $age) $age_half (/ $age 2))
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
            (filter (read-csv "data3.csv") (null? $age))
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
            (filter (read-csv "data3.csv") (not-null? $city))
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
            (with-column (read-csv "data3.csv") $age (fill-null $age 0))
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
            (drop-nulls (read-csv "data3.csv") $name $age)
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
            (with-column (read-csv "data1.csv") $category (when (>= $age 30) "senior" "junior"))
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
            (with-column (read-csv "data1.csv") $status (when (>= $age 30) "senior"))
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
                 (alias (min $age) "min_age")
                 (alias (max $age) "max_age")
                 (alias (count $age) "count_age")
                 (alias (n-unique $city) "unique_cities"))
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
            r#"(write-csv (filter (read-csv "data1.csv") (= $age 30)) "{}")"#,
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
            (filter (read-csv "data4.csv") (str-contains $email "@example\.com$"))
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
            (with-column (read-csv "data4.csv") $masked_sku (str-replace $sku "[0-9]+" "XXX"))
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
            (with-column (read-csv "data4.csv") $sku_code (str-extract $sku ".*-([0-9]+)-.*" 1))
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
            (with-column (read-csv "data1.csv") $city_mean_age (over (mean $age) $city))
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
                (with-column $prev_age (shift $age 1))
                (with-column $running_age (cum-sum $age)))
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
                (with-column $date (to-date $date_str "%Y-%m-%d"))
                (with-column $year (dt-year $date))
                (with-column $month (dt-month $date))
                (with-column $day (dt-day $date))
                (select $id $year $month $day))
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
                (with-column $dt (to-datetime $datetime_str "%Y-%m-%d %H:%M:%S"))
                (with-column $hour (dt-hour $dt))
                (select $id $hour))
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
                (with-column $date (to-date $date_str "%Y-%m-%d"))
                (with-column $formatted_date (dt-format $date "%d/%m/%Y"))
                (select $id $formatted_date))
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
                (with-column $date (to-date $date_str "%Y-%m-%d"))
                (filter (>= (dt-year $date) 2025))
                (select $id $date_str))
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
            r#"(write-parquet (filter (read-csv "data1.csv") (= $city "Paris")) "{}")"#,
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
                (filter (= $department "Engineering"))
                (select (alias (mean $salary) "avg_eng_salary")))
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
            r#"(sink (filter (read-csv "data1.csv") (= $city "Paris")) "{}")"#,
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
            r#"(sink (filter (read-csv "data1.csv") (> $age 28)) "{}")"#,
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
            (sort (pivot (read-csv "data1.csv") $city $name $age) $name)
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
            (sort (pivot (read-csv "data1.csv") $city $name $age :sum) $name)
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

        // Column names can be strings, but the subsequent sort must use a column expression $name
        let code = r#"
            (sort (pivot (read-csv "data1.csv") "city" "name" "age" :first) $name)
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
                (pivot $city $id $age :max)
                (sort $id))
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
            (melt (read-csv "data1.csv") (list $id) (list $name $city))
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
            (melt (read-csv "data1.csv") (list $id $name) (list $age) "metric" "val")
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
                (unpivot (list $id) (list $name $city) "attr" "val")
                (filter (= $attr "city"))
                (sort $id))
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
                (filter (read-csv "data1.csv") (< $age 25))
                (filter (read-csv "data1.csv") (> $age 30)))
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
                (filter (read-csv "data1.csv") (= $id 1))
                (filter (read-csv "data1.csv") (= $id 2))))
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
                (select (read-csv "data1.csv") $id $name)
                (select (read-csv "data1.csv") $age $city)
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
                (select (read-csv "data1.csv") $id $name)
                (select (read-csv "data1.csv") $id $age)
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
            (scalar (agg (read-csv "data1.csv") (sum $age)))
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
            (scalar (agg (read-csv "data1.csv") (mean $age)))
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
            (scalar (select (filter (read-csv "data1.csv") (= $id 1)) $name))
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
            (scalar (select (filter (read-csv "data1.csv") (= $id 1)) (= $name "Alice")))
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval scalar boolean failed")
            .expect_bool()
            .expect("expected bool result");
    
        assert_eq!(res, true);
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
            (with-column (select (read-csv "data1.csv") $id $age) $roll_mean (rolling-mean $age 2))
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
            (with-column (select (read-csv "data1.csv") $id $age) $roll_sum (rolling-sum $age 2))
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
            (-> (select (read-csv "data1.csv") $id $age)
                (with-column $roll_min (rolling-min $age 2))
                (with-column $roll_max (rolling-max $age 2)))
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
            (with-column (select (read-csv "data1.csv") $id $age) $roll_median (rolling-median $age 3))
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
            (with-column (read-csv "data4.csv") $sku_parts (str-split $sku "-"))
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
            (with-column (read-csv "data4.csv") $prefix (str-slice $sku 0 4))
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
            (with-column (read-csv "data4.csv") $stripped (str-strip $sku "PRODLATAMUS-"))
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
                (with-column $no_lead (str-strip-start $sku "PRODEV-"))
                (with-column $no_trail (str-strip-end $sku "-USEUAPACLATAM")))
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
            (with-column (read-csv "data4.csv") $sku_info 
                         (str-extract-groups $sku "^(?<env>[A-Z]+)-(?<code>[0-9]+)-(?<region>[A-Z]+)$"))
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
            (sort (read-csv "data1.csv") (list $city $age) '(:asc :desc))
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
            (sort (read-csv "data1.csv") (list $age $id) :desc)
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
            (select (read-csv "data1.csv") (slice-expr $name 1 3))
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
                    (alias (get $name 0) "first_name")
                    (alias (get $name -1) "last_name"))
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

    #[test]
    fn test_select_variance() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Sample variance with default ddof=1 (24.5) and population variance ddof=0 (19.6)
        let code = r#"
            (select (read-csv "data1.csv") 
                    (alias (var $age) "sample_var")
                    (alias (var $age 0) "pop_var"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval select with 'var' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "sample_var" => &[24.5f64],
            "pop_var" => &[19.6f64],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_select_quantile() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // 50th percentile (median) and 75th percentile with :linear interpolation
        let code = r#"
            (select (read-csv "data1.csv") 
                    (alias (quantile $age 0.5) "q50")
                    (alias (quantile $age 0.75 :linear) "q75"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval select with 'quantile' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "q50" => &[28.0f64],
            "q75" => &[30.0f64],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_select_cov() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Covariance between id and age
        let code = r#"
            (select (read-csv "data1.csv") 
                    (alias (cov $id $age) "cov_id_age"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval select with 'cov' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "cov_id_age" => &[4.5f64],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_select_corr() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Pearson correlation between id and age
        let code = r#"
            (select (read-csv "data1.csv") 
                    (alias (corr $id $age) "corr_id_age"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval select with 'corr' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let corr_val = df.column("corr_id_age")
            .expect("missing corr_id_age column")
            .f64()
            .expect("expected f64 series")
            .get(0)
            .expect("missing scalar result");

        // Verify correlation value approx ~ 0.5749889
        assert!((corr_val - 0.5749889084999459).abs() < 1e-6);
    }

    #[test]
    fn test_describe_dataframe() {
        let env = std_env();
        let mut state = State { line: 0, file: None };
    
        let code = r#"
            (describe (read-csv "data1.csv"))
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval describe failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");
    
        let df = res.collect().expect("failed to collect DataFrame");
    
        // Verify wide output format columns
        assert_eq!(df.height(), 1);
        assert!(df.column("id_count").is_ok());
        assert!(df.column("age_mean").is_ok());
        assert!(df.column("age_std").is_ok());
        assert!(df.column("name_null_count").is_ok());
    }

    #[test]
    fn test_filter_and_multi() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Test `and` with 3 conditions: age > 22, age < 35, and city != "Rome"
        let code = r#"
            (filter (read-csv "data1.csv") (and (> $age 22) (< $age 35) (!= $city "Rome")))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with multi-arg 'and' failed")
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
    fn test_filter_or_multi() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Test `or` with 3 conditions: city is Berlin, city is Madrid, or name is Bob
        let code = r#"
            (filter (read-csv "data1.csv") (or (= $city "Berlin") (= $city "Madrid") (= $name "Bob")))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with multi-arg 'or' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[2i64, 3, 5],
            "name" => &["Bob", "Charlie", "Evan"],
            "age" => &[30i64, 22, 35],
            "city" => &["Paris", "Berlin", "Madrid"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_not_multi() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Test `not` with 2 conditions: negates (age > 24 AND city != "London")
        // Rows matching the inner AND: Bob (30, Paris), Diana (28, Rome), Evan (35, Madrid)
        // Negated result returns Alice (London) and Charlie (age 22)
        let code = r#"
            (filter (read-csv "data1.csv") (not (> $age 24) (!= $city "London")))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with multi-arg 'not' failed")
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
    fn test_scalar_logical_multi() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Test core evaluator boolean scalar behavior
        let and_res = eval_string("(and true true false true)".to_string(), &env, &mut state)
            .expect("eval scalar and failed")
            .expect_bool()
            .expect("expected bool");
        assert_eq!(and_res, false);

        let or_res = eval_string("(or false false true false)".to_string(), &env, &mut state)
            .expect("eval scalar or failed")
            .expect_bool()
            .expect("expected bool");
        assert_eq!(or_res, true);

        let not_res = eval_string("(not true true false)".to_string(), &env, &mut state)
            .expect("eval scalar not failed")
            .expect_bool()
            .expect("expected bool");
        assert_eq!(not_res, true);
    }

    #[test]
    fn test_filter_multiple_predicates_two() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Test passing 2 separate predicate arguments directly to filter: (> $age 22) and (< $age 35)
        let code = r#"
            (filter (read-csv "data1.csv") (> $age 22) (< $age 35))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with multiple predicates failed")
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
    fn test_filter_multiple_predicates_three() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Test passing 3 predicates across multiple columns: (> $age 22), (< $age 35), and (!= $city "Rome")
        let code = r#"
            (filter (read-csv "data1.csv") (> $age 22) (< $age 35) (!= $city "Rome"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with 3 predicates failed")
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
    fn test_filter_multiple_predicates_pipeline_threading() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Test multiple predicates inside the threading macro (->)
        let code = r#"
            (-> (read-csv "data1.csv")
                (filter (>= $age 25) (!= $name "Evan"))
                (select $name $city))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter multi-predicate pipeline failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "name" => &["Alice", "Bob", "Diana"],
            "city" => &["London", "Paris", "Rome"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_multiple_predicates_empty_result() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Predicates that contradict/match no rows
        let code = r#"
            (filter (read-csv "data1.csv") (>= $age 30) (= $city "London"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter empty result failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &Vec::<i64>::new(),
            "name" => &Vec::<&str>::new(),
            "age" => &Vec::<i64>::new(),
            "city" => &Vec::<&str>::new(),
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_with_multiple_columns_basic() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (with-column (read-csv "data1.csv")
                         $age_in_months (* $age 12)
                         $is_over_25 (> $age 25))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval with-column multiple assignments failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "age" => &[25i64, 30, 22, 28, 35],
            "city" => &["London", "Paris", "Berlin", "Rome", "Madrid"],
            "age_in_months" => &[300i64, 360, 264, 336, 420],
            "is_over_25" => &[false, true, false, true, true],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_with_multiple_columns_sequential_dependency() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Test that later column assignments can reference previously created columns in the same call
        let code = r#"
            (with-column (read-csv "data1.csv")
                         $age_plus_10 (+ $age 10)
                         $double_new_age (* $age_plus_10 2)
                         $is_senior (>= $double_new_age 80))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval sequential with-column failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "age" => &[25i64, 30, 22, 28, 35],
            "city" => &["London", "Paris", "Berlin", "Rome", "Madrid"],
            "age_plus_10" => &[35i64, 40, 32, 38, 45],
            "double_new_age" => &[70i64, 80, 64, 76, 90],
            "is_senior" => &[false, true, false, false, true],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_with_multiple_columns_in_pipeline() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        let code = r#"
            (-> (read-csv "data1.csv")
                (with-column $status (when (>= $age 30) "Senior" "Junior")
                             $is_paris (= $city "Paris"))
                (filter $is_paris)
                (select $name $status))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval pipeline with multiple column assignments failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "name" => &["Bob"],
            "status" => &["Senior"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_with_multiple_columns_odd_args_error() {
        let env = std_env();
        let mut state = State { line: 0, file: None };

        // Passing an incomplete pair (df $col1 expr1 $col2) should return WrongNumberOfArguments
        let code = r#"
            (with-column (read-csv "data1.csv") $col1 (+ $age 1) $col2)
        "#.to_string();

        let res = eval_string(code, &env, &mut state);
        assert!(matches!(res, Err(Error::WrongNumberOfArguments { .. })));
    }
}