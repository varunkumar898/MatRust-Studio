use proptest::prelude::*;
use rustlab_parse::parse;

proptest! {
    #[test]
    fn prop_never_panic_on_arbitrary_string(s in "\\PC*") {
        // Parsing arbitrary Unicode strings must never panic or abort:
        // it must return either Ok(Program) or Err(ParseError).
        let _ = parse(&s);
    }

    #[test]
    fn prop_valid_arithmetic_expressions_parse(
        a in -10000..10000,
        b in -10000..10000,
        c in 1..10000
    ) {
        let expr = format!("{a} + {b} * {c} - ({a} / {c})");
        let result = parse(&expr);
        prop_assert!(result.is_ok(), "Expected valid expression '{}' to parse successfully: {:?}", expr, result.err());
    }

    #[test]
    fn prop_matrix_literal_expressions_parse(
        rows in 1usize..5,
        cols in 1usize..5
    ) {
        let mut row_strs = Vec::new();
        for r in 0..rows {
            let mut col_strs = Vec::new();
            for c in 0..cols {
                col_strs.push(format!("{}", r * cols + c + 1));
            }
            row_strs.push(col_strs.join(", "));
        }
        let mat_src = format!("[ {} ]", row_strs.join("; "));
        let result = parse(&mat_src);
        prop_assert!(result.is_ok(), "Expected matrix '{}' to parse: {:?}", mat_src, result.err());
    }
}
