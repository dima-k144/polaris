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
}