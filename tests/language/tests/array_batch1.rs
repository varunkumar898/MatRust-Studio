use matlab_runtime::run_source;

#[test]
fn zeros_ones_eye() {
    let r = run_source("A = zeros(2,3); B = ones(2,3); I = eye(3)").unwrap();
    assert!(r.outputs.iter().any(|o| o.label.as_deref() == Some("I")));
}

#[test]
fn linspace_result() {
    let r = run_source("x = linspace(0,10,5)").unwrap();
    assert!(r.outputs.iter().any(|o| o.label.as_deref() == Some("x")));
}

#[test]
fn diag_vector_and_matrix() {
    let r = run_source("D = diag([1 2 3]); v = diag(D)").unwrap();
    assert!(!r.outputs.is_empty());
}

#[test]
fn repmat_shape() {
    let r = run_source("R = repmat([1 2], 2, 1)").unwrap();
    assert!(r.outputs.iter().any(|o| o.label.as_deref() == Some("R")));
}

#[test]
fn size_length_numel() {
    let r = run_source("A = [1 2 3; 4 5 6]; s = size(A); len = length(A); n = numel(A)").unwrap();
    // Semicolons suppress display; verify via persistent session env.
    let env = r.env;
    let s = env.get("s").unwrap().as_matrix().unwrap();
    assert_eq!(s.rows, 1);
    assert_eq!(s.cols, 2);
    let len = env.get("len").unwrap().as_matrix().unwrap();
    assert_eq!(len.data[0], 3.0); // longest dim of 2x3
    let n = env.get("n").unwrap().as_matrix().unwrap();
    assert_eq!(n.data[0], 6.0);
}

#[test]
fn empty_and_degenerate() {
    let r = run_source("e = isempty([]); d = ndims([1 2])").unwrap();
    assert!(!r.outputs.is_empty());
}

#[test]
fn reshape_and_transpose() {
    let r = run_source("A = [1 2; 3 4; 5 6]; B = reshape(A,2,3); C = transpose(A)").unwrap();
    assert!(!r.outputs.is_empty());
}

#[test]
fn flip_flipud_fliplr_rot90() {
    let r = run_source(
        "A = [1 2; 3 4]; disp(flip(A)); disp(flipud(A)); disp(fliplr(A)); disp(rot90(A));",
    )
    .unwrap();
    assert!(!r.stdout.is_empty());
}

#[test]
fn circshift_triu_tril() {
    let r =
        run_source("A = [1 2; 3 4; 5 6]; disp(circshift(A,[1 0])); disp(triu(A)); disp(tril(A));")
            .unwrap();
    assert!(!r.stdout.is_empty());
}

#[test]
fn invalid_dims_errors() {
    assert!(run_source("zeros(-1,2);").is_err());
    assert!(run_source("zeros(2.5,2);").is_err());
}
