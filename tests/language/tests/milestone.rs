use matlab_runtime::{exec_source, format_value, Env, StatementOutput};

fn matrix_of(env: &Env, name: &str) -> matlab_values::DenseMatrix {
    env.get(name)
        .and_then(|v| v.as_matrix().cloned())
        .unwrap_or_else(|| panic!("variable '{}' missing or not a matrix", name))
}

#[test]
fn milestone_matrix_script_executes() {
    // The milestone script from the project brief:
    //   A = [1 2; 3 4];
    //   B = [5 6; 7 8];
    //   C = A * B;
    //   disp(C);
    // Expected: C = [19 22; 43 50]
    let source = "A = [1 2; 3 4];\nB = [5 6; 7 8];\nC = A * B;\ndisp(C);\n";
    let result =
        matlab_runtime::run_source(source).expect("milestone script should parse and execute");

    let c = matrix_of(&result.env, "C");
    assert_eq!(c.rows, 2);
    assert_eq!(c.cols, 2);
    assert_eq!(&c.data, &[19.0, 22.0, 43.0, 50.0]);

    // Everything was suppressed with ';'; disp(C) prints exactly once.
    assert!(
        result.outputs.is_empty(),
        "suppressed statements must produce no output records"
    );
    assert_eq!(result.stdout.len(), 1);
    let expected_disp = format_value(&matlab_values::Value::Matrix(c.clone()));
    assert_eq!(result.stdout[0], expected_disp);
}

#[test]
fn unsuppressed_outputs_are_labeled() {
    // Without ';', assignments print as 'A =' and bare expressions print as 'ans ='
    let mut env = Env::new();
    let result = exec_source(&mut env, "A = [1 2; 3 0]\nA(2,2)+1\n").expect("run");
    assert_eq!(result.outputs.len(), 2);
    let first = &result.outputs[0];
    let second = &result.outputs[1];
    match (&first.label, &second.label, &first.value) {
        (Some(name), None, label_value) => {
            assert_eq!(name, "A");
            assert!(matches!(label_value, matlab_values::Value::Matrix(_)));
        }
        other => panic!("unexpected labels: {:?}", other),
    }
}

#[test]
fn linear_index_output_value() {
    let mut env = Env::new();
    let result = exec_source(&mut env, "A = [1 2; 3 4];\nA(2)\n").expect("run");
    // Second statement prints as ans = A(2) == 3 (column-major linear indexing)
    assert_eq!(result.outputs.len(), 1);
    let out = &result.outputs[0];
    assert_eq!(out.label, None);
    assert!(out_value_is_3(out).is_ok());
}

fn out_value_is_3(out: &StatementOutput) -> Result<(), ()> {
    if let matlab_values::Value::Matrix(m) = &out.value {
        if m.as_scalar() == Some(3.0) {
            return Ok(());
        }
    }
    Err(())
}
