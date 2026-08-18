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
            (filter (read-csv "data1.csv") (= (col "age") (lit 30)))
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
            (filter (read-csv "data1.csv") (!= (col "age") (lit 30)))
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
            (filter (read-csv "data1.csv") (> (col "age") (lit 25)))
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
            (filter (read-csv "data1.csv") (< (col "age") (lit 28)))
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
            (filter (read-csv "data1.csv") (>= (col "age") (lit 28)))
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
            (filter (read-csv "data1.csv") (<= (col "age") (lit 28)))
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
            (filter (read-csv "data1.csv") (and (> (col "age") (lit 22)) (< (col "age") (lit 35))))
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
            (filter (read-csv "data1.csv") (or (< (col "age") (lit 25)) (>= (col "age") (lit 35))))
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
            (filter (read-csv "data1.csv") (not (> (col "age") (lit 25))))
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
}