#[cfg(test)]
mod tests {
    use crate::wisp::preprocess_wisp3;
    use crate::types::Error;

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

        #[test]
    fn test_comments_and_blank_lines_are_ignored() {
        let code = r#"
; comment before the expression
a ; trailing comment

  b ; nested trailing comment
"#;

        let expected = r#"
(a
  (b))
"#;

        assert_eq!(preprocess_wisp3(code).unwrap(), expected);
    }

    #[test]
    fn test_semicolons_inside_strings_are_preserved() {
        let code = r#"
print "hello;world" ; this is a comment
"#;

        let expected = r#"
(print "hello;world")
"#;

        assert_eq!(preprocess_wisp3(code).unwrap(), expected);
    }

    #[test]
    fn test_colons_inside_strings_are_not_transformed() {
        let code = r#"
message "key: value" : format "ok"
"#;

        let expected = r#"
(message "key: value" (format "ok"))
"#;

        assert_eq!(preprocess_wisp3(code).unwrap(), expected);
    }

    #[test]
    fn test_colons_inside_parenthesized_expressions_are_not_transformed() {
        let code = r#"
outer (inner :keyword value) : next value
"#;

        let expected = r#"
(outer (inner :keyword value) (next value))
"#;

        assert_eq!(preprocess_wisp3(code).unwrap(), expected);
    }

    #[test]
    fn test_multiple_inline_colons_are_nested() {
        let code = r#"
a : b : c : d
"#;

        let expected = r#"
(a (b (c (d))))
"#;

        assert_eq!(preprocess_wisp3(code).unwrap(), expected);
    }

    #[test]
    fn test_escaped_colon_is_literal() {
        let code = r#"
a \: b
"#;

        let expected = r#"
(a : b)
"#;

        assert_eq!(preprocess_wisp3(code).unwrap(), expected);
    }

    #[test]
    fn test_dot_continuation_is_grouped() {
        let code = r#"
a
  . (+ 1 2)
"#;

        let expected = r#"
(a
  (. (+ 1 2)))
"#;

        assert_eq!(preprocess_wisp3(code).unwrap(), expected);
    }

    #[test]
    fn test_dot_continuation_with_multiple_arguments() {
        let code = r#"
pipeline
  . filter (> $age 18)
"#;

        let expected = r#"
(pipeline
  (. filter (> $age 18)))
"#;

        assert_eq!(preprocess_wisp3(code).unwrap(), expected);
    }

    #[test]
    fn test_empty_dot_continuation_is_an_error() {
        let code = r#"
.
"#;

        let result = preprocess_wisp3(code);

        assert!(matches!(result, Err(Error::Custom(message)) if message.contains(
            "continuation requires an expression"
        )));
    }

    #[test]
    fn test_multiline_traditional_expression_is_preserved() {
        let code = r#"
(
  define value
    (+ 1 2)
)
"#;

        let expected = r#"
(
  define value
    (+ 1 2)
)
"#;

        assert_eq!(preprocess_wisp3(code).unwrap(), expected);
    }

    #[test]
    fn test_multiline_bracket_expression_is_preserved() {
        let code = r#"
[
  a
  b
]
"#;

        let expected = r#"
[
  a
  b
]
"#;

        assert_eq!(preprocess_wisp3(code).unwrap(), expected);
    }

    #[test]
    fn test_multiline_brace_infix_expression_is_preserved() {
        let code = r#"
{
  1 + 2
}
"#;

        let expected = r#"
{
  1 + 2
}
"#;

        assert_eq!(preprocess_wisp3(code).unwrap(), expected);
    }

    #[test]
    fn test_traditional_expression_with_string_delimiters() {
        let code = r#"
(print "this string contains (parentheses)")
"#;

        let expected = r#"
(print "this string contains (parentheses)")
"#;

        assert_eq!(preprocess_wisp3(code).unwrap(), expected);
    }

    #[test]
    fn test_wisp_followed_by_traditional_expression() {
        let code = r#"
define value
  (+ 1 2)
(print value)
"#;

        let expected = r#"
(define value
  (+ 1 2))
(print value)
"#;

        assert_eq!(preprocess_wisp3(code).unwrap(), expected);
    }

    #[test]
    fn test_traditional_expression_followed_by_wisp() {
        let code = r#"
(print "before")
define value
  42
"#;

        let expected = r#"
(print "before")
(define value
  (42))
"#;

        assert_eq!(preprocess_wisp3(code).unwrap(), expected);
    }

    #[test]
    fn test_nested_indentation_after_dedent() {
        let code = r#"
root
  child
    grandchild
  sibling
next
"#;

        let expected = r#"
(root
  (child
    (grandchild))
  (sibling))
(next)
"#;

        assert_eq!(preprocess_wisp3(code).unwrap(), expected);
    }

    #[test]
    fn test_single_line_input_without_newline() {
        let code = "value";
        let expected = "(value)";

        assert_eq!(preprocess_wisp3(code).unwrap(), expected);
    }

    #[test]
    fn test_input_without_leading_newline() {
        let code = "a\n  b\n";
        let expected = "(a\n  (b))\n";

        assert_eq!(preprocess_wisp3(code).unwrap(), expected);
    }

    #[test]
    fn test_unclosed_traditional_expression_is_an_error() {
        let code = r#"
(print
  "missing closing paren"
"#;

        assert!(matches!(
            preprocess_wisp3(code),
            Err(Error::UnclosedListExpr)
        ));
    }

    #[test]
    fn test_unexpected_closing_bracket_is_an_error() {
        let code = r#"
[
  a
)
"#;

        assert!(preprocess_wisp3(code).is_err());
    }

    #[test]
    fn test_colon_keyword_syntax_is_not_grouped() {
        let code = r#"
sort $age :desc
"#;

        let expected = r#"
(sort $age :desc)
"#;

        assert_eq!(preprocess_wisp3(code).unwrap(), expected);
    }

    #[test]
    fn test_colon_grouping_requires_whitespace() {
        let code = r#"
a:b
a :b
a: b
"#;

        let expected = r#"
(a:b)
(a :b)
(a: b)
"#;

        assert_eq!(preprocess_wisp3(code).unwrap(), expected);
    }
}
