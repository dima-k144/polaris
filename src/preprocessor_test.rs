#[cfg(test)]
mod tests {
    use crate::wisp::preprocess_wisp3;

    #[test]
    fn test1() {
        let code = r#"
a
"#;
        let expected = r#"
(a)
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test2() {
        let code = r#"
a b c
"#;
        let expected = r#"
(a b c)
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test3() {
        let code = r#"
a
    b
"#;
        let expected = r#"
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
        let code = r#"
a 1 2 3
    b 1 2 3
"#;
        let expected = r#"
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
        let code = r#"
a
    b
        c
"#;
        let expected = r#"
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
        let code = r#"
a
    b
    c
"#;
        let expected = r#"
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
        let code = r#"
a
    b
    c
    d
"#;
        let expected = r#"
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
        let code = r#"
a
    b
        c
    d
"#;
        let expected = r#"
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
        let code = r#"
a
    b
c
"#;
        let expected = r#"
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
        let code = r#"
a
    b
        c
    d
e
"#;
        let expected = r#"
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
        let code = r#"
a : b
"#;
        let expected = r#"
(a (b))
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_colon2() {
        let code = r#"
a 1 2 : b 3 4
"#;
        let expected = r#"
(a 1 2 (b 3 4))
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_colon3() {
        let code = r#"
a : b : c
"#;
        let expected = r#"
(a (b (c)))
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_dot1() {
        let code = r#"
a
. b c
"#;
        let expected = r#"
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
        let code = r#"
1 2
. 3 4
"#;
        let expected = r#"
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
        let code = r#"
(a)
"#;
        let expected = r#"
(a)
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_traditional_sexp2() {
        let code = r#"
(a
    b)
"#;
        let expected = r#"
(a
    b)
"#;

        let result = preprocess_wisp3(code);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected);
    }
}
