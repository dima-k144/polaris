#[cfg(test)]
mod tests {
    use crate::interpreter::{State, eval_string};
    use crate::stdlib::std_env;
    use crate::types::{Expr, ExprValue};
    use polars::prelude::*;

    #[test]
    fn test_wisp_polars_pipeline() {
        let code = r#"
-> (read-csv "tests/data1.csv")
    filter 
        > $age 24
    select $name $city
"#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);
        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            assert_eq!(df.shape(), (4, 2));

            let names: Vec<&str> = df
                .column("name")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            let cities: Vec<&str> = df
                .column("city")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
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
    read-csv "tests/data1.csv"

-> df
    filter 
        > $age 24
    select $name $city
"#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);
        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            assert_eq!(df.shape(), (4, 2));

            let names: Vec<&str> = df
                .column("name")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            let cities: Vec<&str> = df
                .column("city")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            assert_eq!(names, vec!["Alice", "Bob", "Diana", "Evan"]);
            assert_eq!(cities, vec!["London", "Paris", "Rome", "Madrid"]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_colon_chaining_and_with_column() {
        let code = r#"
-> (read-csv "tests/data1.csv")
    with-column $age_months 
        * $age 12
    sort $age_months :desc
"#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);
        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            assert_eq!(df.shape(), (5, 5));

            let age_months: Vec<i64> = df
                .column("age_months")
                .unwrap()
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect();
            assert_eq!(age_months, vec![420, 360, 336, 300, 264]);

            let names: Vec<&str> = df
                .column("name")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            assert_eq!(names, vec!["Evan", "Bob", "Diana", "Alice", "Charlie"]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_polars_join() {
        let code = r#"
define d1 (read-csv "tests/data1.csv")
define d2 (read-csv "tests/data2.csv")
-> (join d1 d2 $id $id)
    select $name $department $salary
"#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);
        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            assert_eq!(df.shape(), (3, 3));

            let names: Vec<&str> = df
                .column("name")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            let departments: Vec<&str> = df
                .column("department")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            let salaries: Vec<i64> = df
                .column("salary")
                .unwrap()
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect();

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
    read-csv "tests/data2.csv"
    group-by $department
        alias (mean $salary) "avg_salary"
        alias (count) "count"
"#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);
        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            // Sort by department to guarantee deterministic row order
            let df = lf
                .sort(["department"], SortMultipleOptions::default())
                .collect()
                .unwrap();
            assert_eq!(df.shape(), (3, 3));

            let depts: Vec<&str> = df
                .column("department")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            let avg_salaries: Vec<f64> = df
                .column("avg_salary")
                .unwrap()
                .f64()
                .unwrap()
                .into_no_null_iter()
                .collect();
            let counts: Vec<u32> = df
                .column("count")
                .unwrap()
                .u32()
                .unwrap()
                .into_no_null_iter()
                .collect();

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
-> (read-csv "tests/data2.csv")
    with-column $dept_avg 
        over (mean $salary) $department
"#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);
        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            assert_eq!(df.shape(), (4, 4));

            let dept_avgs: Vec<f64> = df
                .column("dept_avg")
                .unwrap()
                .f64()
                .unwrap()
                .into_no_null_iter()
                .collect();
            assert_eq!(dept_avgs, vec![67500.0, 65000.0, 67500.0, 80000.0]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_polars_fill_null_and_when() {
        let code = r#"
defun is_senior ()
    when (>= $age 25) "Senior" "Junior"

-> (read-csv "tests/data3.csv")
    with-column $city (fill-null $city "Unknown")
    with-column $is_senior (is_senior)
    "#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);
        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            assert_eq!(df.shape(), (5, 5));

            let cities: Vec<&str> = df
                .column("city")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            assert_eq!(
                cities,
                vec!["London", "Paris", "Unknown", "Rome", "Unknown"]
            );

            let is_senior: Vec<Option<&str>> = df
                .column("is_senior")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .collect();
            assert_eq!(
                is_senior,
                vec![
                    Some("Senior"),
                    Some("Junior"),
                    Some("Junior"),
                    Some("Senior"),
                    Some("Junior")
                ]
            );
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_filter_and_sort_descending() {
        let code = r#"
-> (read-csv "tests/data1.csv")
    filter : > $age 23
    sort $age :desc
    select $name $age
"#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df
                .column("name")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            let ages: Vec<i64> = df
                .column("age")
                .unwrap()
                .i64()
                .unwrap()
                .iter()
                .flatten()
                .collect();

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
-> (read-csv "tests/data1.csv")
    filter : = $city "London"
    with-column $age_next_year : + $age 1
    select $name $age_next_year
"#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df
                .column("name")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            let next_ages: Vec<i64> = df
                .column("age_next_year")
                .unwrap()
                .i64()
                .unwrap()
                .iter()
                .flatten()
                .collect();

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
-> (read-csv "tests/data1.csv")
    filter : str-contains $name "^[A-C]"
    with-column $city_mod : str-replace $city "on$" "on-City"
    select $name $city_mod
    sort $name
    "#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df
                .column("name")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            let cities: Vec<&str> = df
                .column("city_mod")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();

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
    -> (read-csv "tests/data1.csv")
        agg : sum $age
"#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
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
-> (read-csv "tests/data1.csv")
    with-column $category
        when (> $age 27) "Senior" "Junior"
    sort $id
    select $name $category
"#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let categories: Vec<&str> = df
                .column("category")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();

            assert_eq!(df.shape(), (5, 2));
            assert_eq!(
                categories,
                vec!["Junior", "Senior", "Junior", "Senior", "Senior"]
            );
        } else {
            panic!("Expected LazyFrame result");
        }
    }

    #[test]
    fn test_wisp_complex_filter_logical_or() {
        let code = r#"
-> (read-csv "tests/data1.csv")
    filter : or (> $age 28) (= $city "London")
    sort $id
    select $name $city
    "#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df
                .column("name")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            let cities: Vec<&str> = df
                .column("city")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();

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
-> (read-csv "tests/data1.csv")
    filter 
        or 
            > $age 28
            = $city "London"
    sort $id
    select $name $city
    "#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df
                .column("name")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            let cities: Vec<&str> = df
                .column("city")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();

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
-> (read-csv "tests/data1.csv")
    sort $age
    slice 1 3
    select $name $age
    "#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df
                .column("name")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            let ages: Vec<i64> = df
                .column("age")
                .unwrap()
                .i64()
                .unwrap()
                .iter()
                .flatten()
                .collect();

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
-> (read-csv "tests/data1.csv")
    with-column $code 
        str-extract $city "^([A-Za-z]{3})" 1
    sort $id
    select $name $code
    "#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df
                .column("name")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            let codes: Vec<&str> = df
                .column("code")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();

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
-> (read-csv "tests/data1.csv")
    sort $id
    with-column $running_age : cum-sum $age
    select $name $running_age
    "#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df
                .column("name")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            let running_ages: Vec<i64> = df
                .column("running_age")
                .unwrap()
                .i64()
                .unwrap()
                .iter()
                .flatten()
                .collect();

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
-> (read-csv "tests/data1.csv")
    sort $id
    with-column $running_age 
        cum-sum $age
    select $name $running_age
"#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df
                .column("name")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            let running_ages: Vec<i64> = df
                .column("running_age")
                .unwrap()
                .i64()
                .unwrap()
                .iter()
                .flatten()
                .collect();

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
-> (read-csv "tests/data1.csv")
    filter : = $id 1
    with-column $half_age : / (cast $age :float) 2.0
    select $name $half_age
    "#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df
                .column("name")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            let half_ages: Vec<f64> = df
                .column("half_age")
                .unwrap()
                .f64()
                .unwrap()
                .iter()
                .flatten()
                .collect();

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
-> (read-csv "tests/data1.csv")
    filter : = $id 1
    with-column $half_age 
        / (cast $age :float) 2.0
    select $name $half_age
    "#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df
                .column("name")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            let half_ages: Vec<f64> = df
                .column("half_age")
                .unwrap()
                .f64()
                .unwrap()
                .iter()
                .flatten()
                .collect();

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
-> (read-csv "tests/data1.csv")
    filter : = $id 1
    with-column $half_age 
        /
            cast $age :float
            . 2.0
    select $name $half_age
    "#;
        let env = std_env();
        let mut state = State {
            line: 0,
            file: None,
        };
        let result = eval_string(code.to_string(), &env, &mut state);

        assert!(result.is_ok());
        if let ExprValue::LazyFrame(lf) = result.unwrap().value {
            let df = lf.collect().unwrap();
            let names: Vec<&str> = df
                .column("name")
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .flatten()
                .collect();
            let half_ages: Vec<f64> = df
                .column("half_age")
                .unwrap()
                .f64()
                .unwrap()
                .iter()
                .flatten()
                .collect();

            assert_eq!(df.shape(), (1, 2));
            assert_eq!(names, vec!["Alice"]);
            assert_eq!(half_ages, vec![12.5]);
        } else {
            panic!("Expected LazyFrame result");
        }
    }
}
