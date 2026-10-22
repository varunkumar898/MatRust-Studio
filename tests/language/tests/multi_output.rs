use matlab_runtime::run_source;

#[test]
fn meshgrid_2d() {
    let r = run_source("[X, Y] = meshgrid([1 2 3], [4 5])").unwrap();
    let X = r.env.get("X").unwrap().as_matrix().unwrap();
    let Y = r.env.get("Y").unwrap().as_matrix().unwrap();

    assert_eq!(X.rows, 2);
    assert_eq!(X.cols, 3);
    assert_eq!(Y.rows, 2);
    assert_eq!(Y.cols, 3);

    // X varies along columns
    assert_eq!(X.data, vec![1.0, 2.0, 3.0, 1.0, 2.0, 3.0]);
    // Y varies along rows
    assert_eq!(Y.data, vec![4.0, 4.0, 4.0, 5.0, 5.0, 5.0]);
}

#[test]
fn meshgrid_single_output() {
    let r = run_source("X = meshgrid([1 2 3])").unwrap();
    let X = r.env.get("X").unwrap().as_matrix().unwrap();
    assert_eq!(X.rows, 3);
    assert_eq!(X.cols, 3);
}

#[test]
fn ndgrid_2d() {
    let r = run_source("[X, Y] = ndgrid([1 2 3], [4 5])").unwrap();
    let X = r.env.get("X").unwrap().as_matrix().unwrap();
    let Y = r.env.get("Y").unwrap().as_matrix().unwrap();

    assert_eq!(X.rows, 3);
    assert_eq!(X.cols, 2);
    assert_eq!(Y.rows, 3);
    assert_eq!(Y.cols, 2);

    // X varies along rows (ndgrid behavior)
    assert_eq!(X.data, vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0]);
    // Y varies along columns
    assert_eq!(Y.data, vec![4.0, 5.0, 4.0, 5.0, 4.0, 5.0]);
}

#[test]
fn multi_output_size() {
    let r = run_source("[r, c] = size([1 2 3; 4 5 6])").unwrap();
    let r_val = r.env.get("r").unwrap().as_matrix().unwrap();
    let c_val = r.env.get("c").unwrap().as_matrix().unwrap();
    assert_eq!(r_val.data[0], 2.0);
    assert_eq!(c_val.data[0], 3.0);
}

#[test]
fn multi_output_invalid() {
    // Multi-output without a function call should error
    assert!(run_source("[X, Y] = 42").is_err());
    // Multi-output with non-multi function should fall back to single output
    let r = run_source("[X, Y] = zeros(2,3)").unwrap();
    let X = r.env.get("X").unwrap().as_matrix().unwrap();
    let Y = r.env.get("Y").unwrap().as_matrix().unwrap();
    // zeros returns single output; Y gets empty
    assert_eq!(X.rows, 2);
    assert_eq!(X.cols, 3);
    assert_eq!(Y.rows, 0);
    assert_eq!(Y.cols, 0);
}
