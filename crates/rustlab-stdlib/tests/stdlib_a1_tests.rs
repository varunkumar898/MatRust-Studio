//! Milestone A1 Comprehensive Integration & Golden Tests.
//! Validates format recycling (sprintf, fprintf, disp, num2str, mat2str),
//! string and regex library (strsplit, strjoin, strrep, strtrim, upper, lower,
//! strcmp, strcmpi, strncmp, contains, startsWith, endsWith, regexp, regexprep),
//! diagnostics (error, warning, assert), IEEE-754 NaN/Inf predicates,
//! and path discovery (which, exist, addpath, rmpath, scripts vs functions).

use rustlab_math::Value;
use rustlab_stdlib::new_interpreter;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_sprintf_format_recycling_golden() {
    // Golden test cited from MATLAB Documentation:
    // sprintf('%d ', [1 2 3 4]) -> "1 2 3 4 "
    let mut interp = new_interpreter();
    let res = interp
        .eval_script("s = sprintf('%d ', [1 2 3 4]);")
        .unwrap()
        .unwrap();

    let text = match res {
        Value::Char(ca) => ca.as_slice().iter().collect::<String>(),
        other => panic!("Expected char vector, got {:?}", other),
    };
    assert_eq!(text, "1 2 3 4 ");

    // 2D column-major format recycling:
    // A = [1 2; 3 4] in column-major order is 1, 3, 2, 4.
    // sprintf('%0.1f, %0.1f\n', [1 2; 3 4]) cycles twice: (1, 3) then (2, 4)
    let res2 = interp
        .eval_script("s2 = sprintf('%0.1f, %0.1f\\n', [1 2; 3 4]);")
        .unwrap()
        .unwrap();
    let text2 = match res2 {
        Value::Char(ca) => ca.as_slice().iter().collect::<String>(),
        other => panic!("Expected char vector, got {:?}", other),
    };
    assert_eq!(text2, "1.0, 3.0\n2.0, 4.0\n");

    // Zipped multi-array recycling:
    let res3 = interp
        .eval_script("s3 = sprintf('%d -> %d; ', [1; 2], [10; 20]);")
        .unwrap()
        .unwrap();
    let text3 = match res3 {
        Value::Char(ca) => ca.as_slice().iter().collect::<String>(),
        other => panic!("Expected char vector, got {:?}", other),
    };
    assert_eq!(text3, "1 -> 10; 2 -> 20; ");
}

#[test]
fn test_num2str_and_mat2str() {
    let mut interp = new_interpreter();

    // num2str scalar
    let r1 = interp.eval_script("num2str(42)").unwrap().unwrap();
    let t1: String = match r1 {
        Value::Char(ca) => ca.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(t1, "42");

    // num2str with format string
    let r2 = interp.eval_script("num2str(pi, '%.3f')").unwrap().unwrap();
    let t2: String = match r2 {
        Value::Char(ca) => ca.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(t2, "3.142");

    // mat2str 2D matrix
    let r3 = interp.eval_script("mat2str([1 2; 3 4])").unwrap().unwrap();
    let t3: String = match r3 {
        Value::Char(ca) => ca.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(t3, "[1 2; 3 4]");

    // mat2str complex numbers
    let r4 = interp
        .eval_script("mat2str([1+2i, 3-4i])")
        .unwrap()
        .unwrap();
    let t4: String = match r4 {
        Value::Char(ca) => ca.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(t4, "[1+2i 3-4i]");

    // num2str row vector of integers
    let r5 = interp
        .eval_script("num2str([10000 25000])")
        .unwrap()
        .unwrap();
    let t5: String = match r5 {
        Value::Char(ca) => ca.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(t5, "10000  25000");

    // num2str multi-row matrix with non-integers: A3.1.5 requires the
    // MATLAB `%g`-style rule (maximum 5 significant digits, no trailing
    // zeros), right-justified per column. Source: MathWorks num2str.
    let r6 = interp
        .eval_script("num2str([1.234 56.78; 9.1 0.2])")
        .unwrap()
        .unwrap();
    let t6: String = match r6 {
        Value::Char(ca) => ca.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(t6, "1.234  56.78\n  9.1    0.2");
}

#[test]
fn test_string_library_core() {
    let mut interp = new_interpreter();

    // strsplit with default whitespace
    let r1 = interp
        .eval_script("c = strsplit('hello world  rustlab');")
        .unwrap()
        .unwrap();
    if let Value::Cell(ca) = r1 {
        assert_eq!(ca.numel(), 3);
    } else {
        panic!("Expected cell array");
    }

    // strsplit with custom delimiter
    let r2 = interp
        .eval_script("c2 = strsplit('a,b,c', ',');")
        .unwrap()
        .unwrap();
    if let Value::Cell(ca) = r2 {
        assert_eq!(ca.numel(), 3);
    } else {
        panic!();
    }

    // strjoin with custom delimiter
    let r3 = interp
        .eval_script("s = strjoin({'alpha', 'beta', 'gamma'}, ' - ');")
        .unwrap()
        .unwrap();
    let t3: String = match r3 {
        Value::Char(ca) => ca.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(t3, "alpha - beta - gamma");

    // strrep
    let r4 = interp
        .eval_script("s2 = strrep('this is bad', 'bad', 'great');")
        .unwrap()
        .unwrap();
    let t4: String = match r4 {
        Value::Char(ca) => ca.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(t4, "this is great");

    // strtrim
    let r5 = interp
        .eval_script("s3 = strtrim('   padded string   ');")
        .unwrap()
        .unwrap();
    let t5: String = match r5 {
        Value::Char(ca) => ca.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(t5, "padded string");

    // upper and lower
    let r6 = interp.eval_script("upper('hello world')").unwrap().unwrap();
    let t6: String = match r6 {
        Value::Char(ca) => ca.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(t6, "HELLO WORLD");

    let r7 = interp.eval_script("lower('HELLO WORLD')").unwrap().unwrap();
    let t7: String = match r7 {
        Value::Char(ca) => ca.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(t7, "hello world");

    // strcmp, strcmpi, strncmp
    assert_eq!(
        interp.eval_script("strcmp('abc', 'abc')").unwrap().unwrap(),
        Value::logical_scalar(true)
    );
    assert_eq!(
        interp.eval_script("strcmp('abc', 'Abc')").unwrap().unwrap(),
        Value::logical_scalar(false)
    );
    assert_eq!(
        interp
            .eval_script("strcmpi('abc', 'ABC')")
            .unwrap()
            .unwrap(),
        Value::logical_scalar(true)
    );
    assert_eq!(
        interp
            .eval_script("strncmp('abcdef', 'abcXYZ', 3)")
            .unwrap()
            .unwrap(),
        Value::logical_scalar(true)
    );

    // contains, startsWith, endsWith
    assert_eq!(
        interp
            .eval_script("contains('hello world', 'world')")
            .unwrap()
            .unwrap(),
        Value::logical_scalar(true)
    );
    assert_eq!(
        interp
            .eval_script("contains('hello world', 'WORLD', 'IgnoreCase', true)")
            .unwrap()
            .unwrap(),
        Value::logical_scalar(true)
    );
    assert_eq!(
        interp
            .eval_script("startsWith('filename.m', 'file')")
            .unwrap()
            .unwrap(),
        Value::logical_scalar(true)
    );
    assert_eq!(
        interp
            .eval_script("endsWith('filename.m', '.m')")
            .unwrap()
            .unwrap(),
        Value::logical_scalar(true)
    );
}

#[test]
fn test_regexp_and_regexprep() {
    let mut interp = new_interpreter();

    // Default: 1-based start indices
    // 'ab12cd34' digits start at index 3 and 7
    let r1 = interp
        .eval_script("idx = regexp('ab12cd34', '\\d+');")
        .unwrap()
        .unwrap();
    if let Value::Double(da) = r1 {
        assert_eq!(da.as_slice(), &[3.0, 7.0]);
    } else {
        panic!("Expected double array for start indices");
    }

    // 'match' option: cell array of matched substrings
    let r2 = interp
        .eval_script("m = regexp('apple, orange, banana', '\\w+', 'match');")
        .unwrap()
        .unwrap();
    if let Value::Cell(ca) = r2 {
        assert_eq!(ca.numel(), 3);
    } else {
        panic!("Expected cell array for 'match'");
    }

    // 'names' option: struct with named capture groups
    let r3 = interp
        .eval_script("st = regexp('John 42', '(?<name>[A-Za-z]+) (?<age>\\d+)', 'names');")
        .unwrap()
        .unwrap();
    if let Value::Struct(st) = r3 {
        let name_val = st.get_field("name").unwrap();
        let age_val = st.get_field("age").unwrap();
        let name_str: String = match name_val {
            Value::Char(ca) => ca.as_slice().iter().collect(),
            _ => panic!(),
        };
        let age_str: String = match age_val {
            Value::Char(ca) => ca.as_slice().iter().collect(),
            _ => panic!(),
        };
        assert_eq!(name_str, "John");
        assert_eq!(age_str, "42");
    } else {
        panic!("Expected struct for 'names'");
    }

    // regexprep with replacement
    let r4 = interp
        .eval_script("out = regexprep('hello 123 world 456', '\\d+', 'NUM');")
        .unwrap()
        .unwrap();
    let t4: String = match r4 {
        Value::Char(ca) => ca.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(t4, "hello NUM world NUM");

    // regexprep with group capture reference $1
    let r5 = interp
        .eval_script("out2 = regexprep('foo=bar', '(\\w+)=(\\w+)', '$2=$1');")
        .unwrap()
        .unwrap();
    let t5: String = match r5 {
        Value::Char(ca) => ca.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(t5, "bar=foo");
}

#[test]
fn test_diagnostics_error_and_assert() {
    let mut interp = new_interpreter();

    // assert passes
    let pass_res = interp.eval_script("assert(1 == 1);");
    assert!(pass_res.is_ok());

    // assert fails with custom message
    let fail_res = interp.eval_script("assert(1 == 2, 'Assertion failed: numbers unequal');");
    assert!(fail_res.is_err());
    let err_str = format!("{}", fail_res.err().unwrap());
    assert!(err_str.contains("Assertion failed: numbers unequal"));

    // error caught by try-catch
    let try_res = interp
        .eval_script(
            r#"
            try
                error('MYAPP:failed', 'Fatal error triggered');
            catch err
                msg = err.message;
                id = err.identifier;
            end
        "#,
        )
        .unwrap();
    assert!(try_res.is_some());

    let msg = interp.eval_script("msg").unwrap().unwrap();
    let id = interp.eval_script("id").unwrap().unwrap();
    let msg_str: String = match msg {
        Value::Char(c) => c.as_slice().iter().collect(),
        _ => panic!(),
    };
    let id_str: String = match id {
        Value::Char(c) => c.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(msg_str, "Fatal error triggered");
    assert_eq!(id_str, "MYAPP:failed");
}

#[test]
fn test_ieee754_nan_inf_semantics() {
    let mut interp = new_interpreter();

    // isnan, isinf, isfinite
    interp.eval_script("x = [1, NaN, Inf, -Inf, 5];").unwrap();

    let is_nan = interp.eval_script("isnan(x)").unwrap().unwrap();
    if let Value::Logical(la) = is_nan {
        assert_eq!(la.as_slice(), &[false, true, false, false, false]);
    } else {
        panic!();
    }

    let is_inf = interp.eval_script("isinf(x)").unwrap().unwrap();
    if let Value::Logical(la) = is_inf {
        assert_eq!(la.as_slice(), &[false, false, true, true, false]);
    } else {
        panic!();
    }

    let is_fin = interp.eval_script("isfinite(x)").unwrap().unwrap();
    if let Value::Logical(la) = is_fin {
        assert_eq!(la.as_slice(), &[true, false, false, false, true]);
    } else {
        panic!();
    }

    // IEEE-754 comparison semantics:
    // NaN == NaN -> false
    let eq_nan = interp.eval_script("NaN == NaN").unwrap().unwrap();
    assert_eq!(eq_nan, Value::logical_scalar(false));

    // NaN ~= NaN -> true
    let ne_nan = interp.eval_script("NaN ~= NaN").unwrap().unwrap();
    assert_eq!(ne_nan, Value::logical_scalar(true));

    // NaN > 0 -> false
    let gt_nan = interp.eval_script("NaN > 0").unwrap().unwrap();
    assert_eq!(gt_nan, Value::logical_scalar(false));
}

#[test]
fn test_path_which_exist_and_scripts() {
    let tmp = tempdir().unwrap();
    let tmp_path = tmp.path().to_str().unwrap().replace('\\', "/");

    let script_file = tmp.path().join("my_script.m");
    fs::write(
        &script_file,
        r#"
        % A script modifies caller workspace directly
        val_from_script = 42 * 2;
    "#,
    )
    .unwrap();

    let func_file = tmp.path().join("my_func.m");
    fs::write(
        &func_file,
        r#"
        function y = my_func(x)
            y = x * 10;
        end
    "#,
    )
    .unwrap();

    let mut interp = new_interpreter();

    // Add path
    let addpath_cmd = format!("addpath('{tmp_path}');");
    interp.eval_script(&addpath_cmd).unwrap();

    // Check exist
    // 5 = builtin (sin)
    let exist_sin = interp.eval_script("exist('sin')").unwrap().unwrap();
    assert_eq!(exist_sin, Value::scalar(5.0));

    // 2 = file/function on path (my_func)
    let exist_func = interp.eval_script("exist('my_func')").unwrap().unwrap();
    assert_eq!(exist_func, Value::scalar(2.0));

    // which
    let which_sin = interp.eval_script("which('sin')").unwrap().unwrap();
    let sin_text: String = match which_sin {
        Value::Char(c) => c.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert!(sin_text.contains("built-in function"));

    // Execute function from M-file
    let func_res = interp.eval_script("res = my_func(5);").unwrap().unwrap();
    assert_eq!(func_res, Value::scalar(50.0));

    // Execute script from M-file
    interp.eval_script("my_script").unwrap();
    let script_var = interp.eval_script("val_from_script").unwrap().unwrap();
    assert_eq!(script_var, Value::scalar(84.0));

    // which on variable
    let which_var = interp
        .eval_script("which('val_from_script')")
        .unwrap()
        .unwrap();
    let var_text: String = match which_var {
        Value::Char(c) => c.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert!(var_text.contains("is a variable"));

    // rmpath
    let rmpath_cmd = format!("rmpath('{tmp_path}');");
    interp.eval_script(&rmpath_cmd).unwrap();
}

#[test]
fn test_number_display_format_short_golden() {
    let mut interp = new_interpreter();

    // 1. Integer-valued doubles print without decimals: 1 4 9, not 1.0000 4.0000 9.0000
    let r1 = interp.eval_script("[1, 4, 9]").unwrap().unwrap();
    let s1 = format!("{}", r1);
    assert!(
        !s1.contains(".0000"),
        "Integer-valued vector must not contain .0000"
    );
    assert!(s1.contains("1") && s1.contains("4") && s1.contains("9"));

    // 2. Scalars: integers as integers, non-integers with 4 decimals
    let r2 = interp.eval_script("42").unwrap().unwrap();
    assert_eq!(format!("{}", r2).trim(), "42");

    let r3 = interp.eval_script("pi").unwrap().unwrap();
    assert_eq!(format!("{}", r3).trim(), "3.1416");

    // 3. Matrix formatting: non-integers with 4 decimals, columns aligned
    let r4 = interp.eval_script("[1.2, 3.4; 5.6, 7.8]").unwrap().unwrap();
    let s4 = format!("{}", r4);
    assert!(s4.contains("1.2000") && s4.contains("3.4000"));
    assert!(s4.contains("5.6000") && s4.contains("7.8000"));

    // 4. Large integer vector formatted with exact integer columns
    let r5 = interp.eval_script("[10000, 25000]").unwrap().unwrap();
    let s5 = format!("{}", r5);
    assert_eq!(s5, "    10000  25000");

    // 5. NaN and Inf shown as NaN/Inf
    let r6 = interp.eval_script("[NaN, Inf, -Inf, 5]").unwrap().unwrap();
    let s6 = format!("{}", r6);
    assert!(s6.contains("NaN"));
    assert!(s6.contains("Inf"));
    assert!(s6.contains("-Inf"));

    // 6. Scientific notation for values with magnitude < 1e-3 or >= 1e4 (e.g. tiny 8.8818e-16)
    let r7 = interp.eval_script("1e-16").unwrap().unwrap();
    assert_eq!(format!("{}", r7).trim(), "1.0000e-16");

    let r8 = interp.eval_script("8.8818e-16").unwrap().unwrap();
    assert_eq!(format!("{}", r8).trim(), "8.8818e-16");

    // 7. Matrix column uniform width: all columns in a matrix share the same column width
    let r9 = interp
        .eval_script("[-0.7071, 0.7071; 0.7071, 0.7071]")
        .unwrap()
        .unwrap();
    let s9 = format!("{}", r9);
    for line in s9.lines() {
        if line.trim().is_empty() {
            continue;
        }
        // Each row should have two aligned columns with equal cell width
        let cols: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(cols.len(), 2, "Row must contain 2 columns");
    }
}

#[test]
fn test_sprintf_fprintf_edge_cases() {
    let mut interp = new_interpreter();

    // No specifiers + args must not loop forever
    let r1 = interp
        .eval_script("sprintf('hello', 1, 2, 3)")
        .unwrap()
        .unwrap();
    let s1: String = match r1 {
        Value::Char(c) => c.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(s1, "hello");

    // sprintf('hello %d') -> 'hello '
    let r2 = interp.eval_script("sprintf('hello %d')").unwrap().unwrap();
    let s2: String = match r2 {
        Value::Char(c) => c.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(s2, "hello ");

    // sprintf('%d, %d\n', [1 2 3]) -> "1, 2\n3, " (stop at first specifier with no remaining data)
    let r3 = interp
        .eval_script("sprintf('%d, %d\\n', [1 2 3])")
        .unwrap()
        .unwrap();
    let s3: String = match r3 {
        Value::Char(c) => c.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(s3, "1, 2\n3, ");

    // %% literal
    let r4 = interp
        .eval_script("sprintf('100%% complete')")
        .unwrap()
        .unwrap();
    let s4: String = match r4 {
        Value::Char(c) => c.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(s4, "100% complete");

    // %s with char array
    let r5 = interp
        .eval_script("sprintf('item: %s', 'book')")
        .unwrap()
        .unwrap();
    let s5: String = match r5 {
        Value::Char(c) => c.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(s5, "item: book");

    // %5.2f, %-8s, %e, %g, %c, %x
    let r6 = interp
        .eval_script("sprintf('%5.2f | %-8s | %c | %x', 3.14159, 'left', 'Z', 255)")
        .unwrap()
        .unwrap();
    let s6: String = match r6 {
        Value::Char(c) => c.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(s6, " 3.14 | left     | Z | ff");

    // Empty array argument behavior: stops before missing data
    let r7 = interp
        .eval_script("sprintf('empty: %d', [])")
        .unwrap()
        .unwrap();
    let s7: String = match r7 {
        Value::Char(c) => c.as_slice().iter().collect(),
        _ => panic!(),
    };
    assert_eq!(s7, "empty: ");
}

#[test]
fn test_nd_array_trailing_end_expansion() {
    let mut interp = new_interpreter();

    // Create 2 x 3 x 4 array (24 elements)
    interp.eval_script("A = reshape(1:24, [2, 3, 4]);").unwrap();

    // When indexed with 2 subscripts A(r, c), c absorbs remaining dims (3 * 4 = 12)
    // A(1, end) must have end = 12, referring to element at (r=1, c=12), which is element 23 (1-based)!
    let r1 = interp.eval_script("val = A(1, end);").unwrap().unwrap();
    assert_eq!(r1, Value::scalar(23.0));

    let r2 = interp.eval_script("val2 = A(2, end);").unwrap().unwrap();
    assert_eq!(r2, Value::scalar(24.0));
}

#[test]
fn test_nested_anonymous_functions_and_closure_handles() {
    let mut interp = new_interpreter();

    // 1. Nested anonymous functions: @(x) @(y) x + y
    interp.eval_script("f_add = @(x) @(y) x + y;").unwrap();
    interp.eval_script("add10 = f_add(10);").unwrap();
    let r1 = interp.eval_script("res = add10(5);").unwrap().unwrap();
    assert_eq!(r1, Value::scalar(15.0));

    // 2. Handles capturing handles
    interp.eval_script("h1 = @(x) x * 2;").unwrap();
    interp.eval_script("h2 = @(y) h1(y) + 3;").unwrap();
    let r2 = interp.eval_script("res2 = h2(4);").unwrap().unwrap();
    assert_eq!(r2, Value::scalar(11.0));

    // 3. Closures inside cellfun
    interp.eval_script("c = {1, 2, 3};").unwrap();
    interp.eval_script("factor = 10;").unwrap();
    let r3 = interp
        .eval_script("res3 = cellfun(@(x) x * factor, c);")
        .unwrap()
        .unwrap();
    if let Value::Double(da) = r3 {
        assert_eq!(da.as_slice(), &[10.0, 20.0, 30.0]);
    } else {
        panic!();
    }
}

#[test]
fn test_boolean_mask_shape_rules_and_deletion() {
    let mut interp = new_interpreter();

    // 1. Row vector A with mask -> row vector result (1 x K)
    let r1 = interp
        .eval_script("row = [10, 20, 30, 40]; sub_row = row(row > 15);")
        .unwrap()
        .unwrap();
    assert_eq!(r1.shape(), &[1, 3]);

    // 2. Column vector A with mask -> column vector result (K x 1)
    let r2 = interp
        .eval_script("col = [10; 20; 30; 40]; sub_col = col(col > 15);")
        .unwrap()
        .unwrap();
    assert_eq!(r2.shape(), &[3, 1]);

    // 3. 2D matrix A with mask -> column vector result (K x 1)
    let r3 = interp
        .eval_script("mat = [10, 20; 30, 40]; sub_mat = mat(mat > 15);")
        .unwrap()
        .unwrap();
    assert_eq!(r3.shape(), &[3, 1]);

    // 4. A(mask) = [] deletion
    interp
        .eval_script("A = [1, 2, 3, 4, 5]; A(A > 3) = [];")
        .unwrap();
    let rem = interp.eval_script("A").unwrap().unwrap();
    if let Value::Double(da) = rem {
        assert_eq!(da.as_slice(), &[1.0, 2.0, 3.0]);
        assert_eq!(da.shape(), &[1, 3]);
    } else {
        panic!();
    }

    // 5. A(indices) = [] linear deletion
    interp
        .eval_script("B = [10, 20, 30, 40]; B([2, 4]) = [];")
        .unwrap();
    let rem_b = interp.eval_script("B").unwrap().unwrap();
    if let Value::Double(da) = rem_b {
        assert_eq!(da.as_slice(), &[10.0, 30.0]);
    } else {
        panic!();
    }
}

#[test]
fn test_nan_comparison_semantics_regression() {
    let mut interp = new_interpreter();

    // NaN == NaN is false
    let r1 = interp.eval_script("NaN == NaN").unwrap().unwrap();
    assert_eq!(r1, Value::logical_scalar(false));

    // x == x with NaN element
    let r2 = interp
        .eval_script("x = [1, NaN, 3]; x == x")
        .unwrap()
        .unwrap();
    if let Value::Logical(la) = r2 {
        assert_eq!(la.as_slice(), &[true, false, true]);
    } else {
        panic!();
    }
}
