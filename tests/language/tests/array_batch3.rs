use matlab_runtime::run_source;

#[test]
fn sort_vector() {
    let r = run_source("A = [3 1 2]; B = sort(A)").unwrap();
    let B = r.env.get("B").unwrap().as_matrix().unwrap();
    assert_eq!(B.data, vec![1.0, 2.0, 3.0]);
}

#[test]
fn sort_matrix_columns() {
    let r = run_source("A = [3 4; 1 2]; B = sort(A)").unwrap();
    let B = r.env.get("B").unwrap().as_matrix().unwrap();
    // Sorts each column: [1 2; 3 4] -> row-major [1, 2, 3, 4]
    assert_eq!(B.data, vec![1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn sort_descending_not_supported() {
    // sort with dim arg
    let r = run_source("A = [3 1; 2 4]; B = sort(A, 2)").unwrap();
    let B = r.env.get("B").unwrap().as_matrix().unwrap();
    // Sorts each row
    assert_eq!(B.data, vec![1.0, 3.0, 2.0, 4.0]);
}

#[test]
fn sortrows_basic() {
    let r = run_source("A = [3 1; 1 2; 2 3]; B = sortrows(A)").unwrap();
    let B = r.env.get("B").unwrap().as_matrix().unwrap();
    // Sorted by first column
    assert_eq!(B.data, vec![1.0, 2.0, 2.0, 3.0, 3.0, 1.0]);
}

#[test]
fn unique_basic() {
    let r = run_source("u = unique([3 1 2 1 3])").unwrap();
    let u = r.env.get("u").unwrap().as_matrix().unwrap();
    assert_eq!(u.data, vec![1.0, 2.0, 3.0]);
    assert_eq!(u.rows, 3);
    assert_eq!(u.cols, 1);
}

#[test]
fn unique_empty() {
    let r = run_source("u = unique([])").unwrap();
    let u = r.env.get("u").unwrap().as_matrix().unwrap();
    assert_eq!(u.rows, 0);
}

#[test]
fn ismember_basic() {
    let r = run_source("m = ismember([1 2 3 4], [2 4])").unwrap();
    let m = r.env.get("m").unwrap().as_matrix().unwrap();
    assert_eq!(m.data, vec![0.0, 1.0, 0.0, 1.0]);
}

#[test]
fn setdiff_basic() {
    let r = run_source("s = setdiff([1 2 3 4], [2 4])").unwrap();
    let s = r.env.get("s").unwrap().as_matrix().unwrap();
    assert_eq!(s.data, vec![1.0, 3.0]);
}

#[test]
fn union_basic() {
    let r = run_source("u = union([1 2 3], [2 3 4])").unwrap();
    let u = r.env.get("u").unwrap().as_matrix().unwrap();
    assert_eq!(u.data, vec![1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn intersect_basic() {
    let r = run_source("i = intersect([1 2 3], [2 3 4])").unwrap();
    let i = r.env.get("i").unwrap().as_matrix().unwrap();
    assert_eq!(i.data, vec![2.0, 3.0]);
}

#[test]
fn find_vector() {
    let r = run_source("A = [0 3 0 4]; I = find(A)").unwrap();
    let I = r.env.get("I").unwrap().as_matrix().unwrap();
    assert_eq!(I.data, vec![2.0, 4.0]);
}

#[test]
fn find_matrix() {
    let r = run_source("A = [1 0; 0 2]; I = find(A)").unwrap();
    let I = r.env.get("I").unwrap().as_matrix().unwrap();
    // Column-major: A(1,1)=1, A(2,1)=0, A(1,2)=0, A(2,2)=2
    // Nonzero: index 1 (row1,col1), index 4 (row2,col2)
    assert_eq!(I.data, vec![1.0, 4.0]);
}

#[test]
fn find_multi_output() {
    let r = run_source("A = [1 0; 0 2]; [r, c] = find(A)").unwrap();
    let rv = r.env.get("r").unwrap().as_matrix().unwrap();
    let cv = r.env.get("c").unwrap().as_matrix().unwrap();
    assert_eq!(rv.data, vec![1.0, 2.0]);
    assert_eq!(cv.data, vec![1.0, 2.0]);
}

#[test]
fn any_all_vectors() {
    let r = run_source("a = any([0 1 0]); al = all([1 2 3])").unwrap();
    let a = r.env.get("a").unwrap().as_matrix().unwrap();
    let al = r.env.get("al").unwrap().as_matrix().unwrap();
    // any([0 1 0]) default dim=2 (first non-singleton) -> 1x1 = [1]
    assert_eq!(a.data, vec![1.0]);
    // all([1 2 3]) default dim=2 -> 1x1 = [1]
    assert_eq!(al.data, vec![1.0]);
}

#[test]
fn any_all_matrices() {
    let r = run_source("A = [1 0; 0 1]; a = any(A); al = all(A)").unwrap();
    let a = r.env.get("a").unwrap().as_matrix().unwrap();
    let al = r.env.get("al").unwrap().as_matrix().unwrap();
    // any: per column - col1 has 1, col2 has 1
    assert_eq!(a.data, vec![1.0, 1.0]);
    // all: per column - col1 has 0, col2 has 0
    assert_eq!(al.data, vec![0.0, 0.0]);
}

#[test]
fn cumsum_basic() {
    let r = run_source("A = [1 2 3; 4 5 6]; S = cumsum(A)").unwrap();
    let S = r.env.get("S").unwrap().as_matrix().unwrap();
    assert_eq!(S.data, vec![1.0, 2.0, 3.0, 5.0, 7.0, 9.0]);
}

#[test]
fn cumprod_basic() {
    let r = run_source("A = [1 2 3; 4 5 6]; P = cumprod(A)").unwrap();
    let P = r.env.get("P").unwrap().as_matrix().unwrap();
    assert_eq!(P.data, vec![1.0, 2.0, 3.0, 4.0, 10.0, 18.0]);
}

#[test]
fn diff_basic() {
    let r = run_source("A = [1 2 3 4]; D = diff(A)").unwrap();
    let D = r.env.get("D").unwrap().as_matrix().unwrap();
    // diff along columns (rows), for 1x4 input -> 0x4 empty
    assert_eq!(D.rows, 0);
    assert_eq!(D.cols, 4);
}

#[test]
fn diff_matrix() {
    let r = run_source("A = [1 2; 4 5; 7 8]; D = diff(A)").unwrap();
    let D = r.env.get("D").unwrap().as_matrix().unwrap();
    // Each column diff
    assert_eq!(D.rows, 2);
    assert_eq!(D.cols, 2);
    assert_eq!(D.data, vec![3.0, 3.0, 3.0, 3.0]);
}

#[test]
fn invalid_args() {
    assert!(run_source("sort()").is_err());
    assert!(run_source("find()").is_err());
    assert!(run_source("unique(1, 2)").is_err());
    assert!(run_source("setdiff([1], [2], [3])").is_err());
    assert!(run_source("cumsum()").is_err());
    assert!(run_source("diff()").is_err());
}

#[test]
fn empty_inputs() {
    let r = run_source("u = unique([])").unwrap();
    let u = r.env.get("u").unwrap().as_matrix().unwrap();
    assert_eq!(u.rows, 0);

    let r = run_source("f = find([])").unwrap();
    let f = r.env.get("f").unwrap().as_matrix().unwrap();
    assert_eq!(f.rows, 0);

    let r = run_source("a = any([])").unwrap();
    let a = r.env.get("a").unwrap().as_matrix().unwrap();
    assert_eq!(a.data, vec![]);

    let r = run_source("d = diff([1 2 3])").unwrap();
    let d = r.env.get("d").unwrap().as_matrix().unwrap();
    assert_eq!(d.rows, 0);
}
