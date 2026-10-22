use matlab_values::Value;

fn run(source: &str) -> Result<matlab_runtime::RunResult, matlab_diagnostics::Diagnostic> {
    matlab_runtime::run_source(source)
}

fn env_matrix(result: &matlab_runtime::RunResult, name: &str) -> matlab_values::DenseMatrix {
    result
        .env
        .get(name)
        .and_then(|v| v.as_matrix().cloned())
        .unwrap_or_else(|| panic!("variable '{}' missing", name))
}

#[test]
fn size_zeros_ones_eye() {
    let result = run(
        "A = [1 2; 3 4];\ns1 = size(A);\nr = size(A,1);\nc = size(A,2);\nZ = zeros(2,3);\nO = ones(2);\nI = eye(3);\n",
    )
    .expect("run");
    let s1 = env_matrix(&result, "s1");
    assert_eq!(&s1.data, &[2.0, 2.0]);
    assert_eq!(env_matrix(&result, "r").as_scalar(), Some(2.0));
    let c = env_matrix(&result, "c");
    assert_eq!(c.as_scalar(), Some(2.0));

    let z = env_matrix(&result, "Z");
    assert_eq!(&z.data, &[0.0; 6]);
    let o = env_matrix(&result, "O");
    assert_eq!(o.data, vec![1.0; 4]);
    let i = env_matrix(&result, "I");
    assert_eq!(i.rows, 3);
    assert_eq!(i.cols, 3);
    assert_eq!(i.data[0], 1.0);
    assert_eq!(i.data[4], 1.0);
    assert_eq!(i.data[8], 1.0);
    assert_eq!(i.data[1], 0.0);
}

#[test]
fn length_and_numel() {
    let result = run("A = [1 2 3];\nl = length(A);\nn = numel(A);\n").expect("run");
    assert_eq!(env_matrix(&result, "l").as_scalar(), Some(3.0));
    assert_eq!(env_matrix(&result, "n").as_scalar(), Some(3.0));
}

#[test]
fn column_reductions_match_matlab_defaults() {
    // sum/max/min without dimension arguments reduce along columns
    let result = run("A = [1 2; 3 4];\ns = sum(A);\nm = max(A);\nn = min(A);\n").expect("run");
    let s = env_matrix(&result, "s");
    assert_eq!(&s.data, &[4.0, 6.0]);
    let m = env_matrix(&result, "m");
    assert_eq!(&m.data, &[3.0, 4.0]);
    let n = env_matrix(&result, "n");
    assert_eq!(&n.data, &[1.0, 2.0]);
}

#[test]
fn scalar_math_functions() {
    let result = run(
        "a = abs(-3.5);\nb = sqrt(9);\nc = floor(1.7);\nd = ceil(1.2);\ne = exp(0);\nf = log(1);\n",
    )
    .expect("run");
    let expect = |name: &str, want: f64| {
        let v = result.env.get(name).unwrap();
        match v {
            Value::Matrix(m) => assert_eq!(m.as_scalar(), Some(want), "{}", name),
            other => panic!("{} not a matrix: {:?}", name, other),
        }
    };
    expect("a", 3.5);
    expect("b", 3.0);
    expect("c", 1.0);
    expect("d", 2.0);
    expect("e", 1.0);
    expect("f", 0.0);
}

#[test]
fn pi_constant() {
    let result = run("p = pi;\n").expect("run");
    let p = env_matrix(&result, "p");
    assert!((p.as_scalar().unwrap() - std::f64::consts::PI).abs() < f64::EPSILON);
}

#[test]
fn unknown_function_or_variable_is_rejected() {
    let err = run("foo(1)\n").expect_err("undefined function");
    assert!(format!("{}", err).contains("foo"));

    let err = run("x + 1\n").expect_err("undefined variable");
    assert!(format!("{}", err).contains("x"));
}

#[test]
fn strings_are_stored_but_not_arithmetic_for_now() {
    let result = run("s = 'abc';\n").expect("run");
    match result.env.get("s").unwrap() {
        Value::String(x) => assert_eq!(x, "abc"),
        other => panic!("expected string, got {:?}", other),
    }

    let err = run("s = 'abc';\nt = s + 1;\n").expect_err("string arithmetic");
    assert!(format!("{}", err).contains("Conversion to double from char"));
}

#[test]
fn unterminated_matrix_literal_is_a_parse_error() {
    let err = run("A = [1 2; 3 4\n").expect_err("unterminated [");
    assert!(format!("{}", err).contains("Unterminated"));
}
