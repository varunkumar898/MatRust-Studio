use matlab_runtime::run_source;

// ============================================================================
// Group A: Central Tendency — mean, median, mode
// ============================================================================

#[test]
fn mean_vector_row_and_column() {
    let r =
        run_source("m_row = mean([1, 2, 3, 4, 5]); m_col = mean([10; 20; 30]); m_scal = mean(42)")
            .unwrap();
    let m_row = r.env.get("m_row").unwrap().as_matrix().unwrap();
    assert_eq!(m_row.rows, 1);
    assert_eq!(m_row.cols, 1);
    assert_eq!(m_row.data[0], 3.0);

    let m_col = r.env.get("m_col").unwrap().as_matrix().unwrap();
    assert_eq!(m_col.rows, 1);
    assert_eq!(m_col.cols, 1);
    assert_eq!(m_col.data[0], 20.0);

    let m_scal = r.env.get("m_scal").unwrap().as_matrix().unwrap();
    assert_eq!(m_scal.data[0], 42.0);
}

#[test]
fn mean_matrix_dimensions_and_all() {
    let r = run_source(
        "M = [1, 2, 3; 4, 5, 6]; \
         m_default = mean(M); \
         m_dim1 = mean(M, 1); \
         m_dim2 = mean(M, 2); \
         m_all = mean(M, 'all'); \
         m_dim3 = mean(M, 3)",
    )
    .unwrap();

    let m_def = r.env.get("m_default").unwrap().as_matrix().unwrap();
    assert_eq!(m_def.rows, 1);
    assert_eq!(m_def.cols, 3);
    assert_eq!(m_def.data, vec![2.5, 3.5, 4.5]);

    let m_d1 = r.env.get("m_dim1").unwrap().as_matrix().unwrap();
    assert_eq!(m_d1.rows, 1);
    assert_eq!(m_d1.cols, 3);
    assert_eq!(m_d1.data, vec![2.5, 3.5, 4.5]);

    let m_d2 = r.env.get("m_dim2").unwrap().as_matrix().unwrap();
    assert_eq!(m_d2.rows, 2);
    assert_eq!(m_d2.cols, 1);
    assert_eq!(m_d2.data, vec![2.0, 5.0]);

    let m_all = r.env.get("m_all").unwrap().as_matrix().unwrap();
    assert_eq!(m_all.rows, 1);
    assert_eq!(m_all.cols, 1);
    assert_eq!(m_all.data[0], 3.5);

    // Dimension 3 (higher singleton dim) leaves 2x3 matrix unchanged
    let m_d3 = r.env.get("m_dim3").unwrap().as_matrix().unwrap();
    assert_eq!(m_d3.rows, 2);
    assert_eq!(m_d3.cols, 3);
    assert_eq!(m_d3.data, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
}

#[test]
fn mean_empty_and_nan() {
    let r = run_source("m_empty = mean([]); m_nan = mean([1, asind(2), 3])").unwrap();
    let m_empty = r.env.get("m_empty").unwrap().as_matrix().unwrap();
    assert!(m_empty.data[0].is_nan());

    let m_nan = r.env.get("m_nan").unwrap().as_matrix().unwrap();
    assert!(m_nan.data[0].is_nan());
}

#[test]
fn median_odd_even_and_matrix() {
    let r = run_source(
        "med_odd = median([1, 5, 2]); \
         med_even = median([1, 4, 2, 3]); \
         M = [1, 10; 3, 20; 2, 30]; \
         med_m_default = median(M); \
         med_m_dim2 = median(M, 2); \
         med_all = median(M, 'all')",
    )
    .unwrap();

    let odd = r.env.get("med_odd").unwrap().as_matrix().unwrap();
    assert_eq!(odd.data[0], 2.0);

    let even = r.env.get("med_even").unwrap().as_matrix().unwrap();
    assert_eq!(even.data[0], 2.5);

    let m_def = r.env.get("med_m_default").unwrap().as_matrix().unwrap();
    assert_eq!(m_def.rows, 1);
    assert_eq!(m_def.cols, 2);
    assert_eq!(m_def.data, vec![2.0, 20.0]);

    let m_d2 = r.env.get("med_m_dim2").unwrap().as_matrix().unwrap();
    assert_eq!(m_d2.rows, 3);
    assert_eq!(m_d2.cols, 1);
    assert_eq!(m_d2.data, vec![5.5, 11.5, 16.0]);

    let m_all = r.env.get("med_all").unwrap().as_matrix().unwrap();
    // Elements: 1, 2, 3, 10, 20, 30 -> median is (3+10)/2 = 6.5
    assert_eq!(m_all.data[0], 6.5);
}

#[test]
fn mode_unimodal_and_ties() {
    let r = run_source(
        "m1 = mode([1, 2, 2, 3]); \
         m_tie = mode([3, 1, 3, 1, 2]); \
         m_distinct = mode([5, 9, 2]); \
         M = [1, 2; 1, 3; 4, 2]; \
         m_mat = mode(M); \
         m_mat_dim2 = mode(M, 2); \
         m_mat_all = mode(M, 'all')",
    )
    .unwrap();

    assert_eq!(r.env.get("m1").unwrap().as_matrix().unwrap().data[0], 2.0);
    // In ties (1 and 3 both appear twice), MATLAB returns the smallest value (1)
    assert_eq!(
        r.env.get("m_tie").unwrap().as_matrix().unwrap().data[0],
        1.0
    );
    // All unique: smallest value is 2
    assert_eq!(
        r.env.get("m_distinct").unwrap().as_matrix().unwrap().data[0],
        2.0
    );

    let m_mat = r.env.get("m_mat").unwrap().as_matrix().unwrap();
    assert_eq!(m_mat.rows, 1);
    assert_eq!(m_mat.cols, 2);
    assert_eq!(m_mat.data, vec![1.0, 2.0]);

    let m_d2 = r.env.get("m_mat_dim2").unwrap().as_matrix().unwrap();
    assert_eq!(m_d2.rows, 3);
    assert_eq!(m_d2.cols, 1);
    assert_eq!(m_d2.data, vec![1.0, 1.0, 2.0]);

    // All elements: [1, 1, 4, 2, 3, 2] -> 1 appears 2 times, 2 appears 2 times -> smallest is 1
    assert_eq!(
        r.env.get("m_mat_all").unwrap().as_matrix().unwrap().data[0],
        1.0
    );
}

// ============================================================================
// Group B: Measures of Spread — std, var, range, iqr
// ============================================================================

#[test]
fn var_sample_and_population() {
    let r = run_source(
        "v_w0 = var([1, 2, 3, 4, 5]); \
         v_w0_expl = var([1, 2, 3, 4, 5], 0); \
         v_w1 = var([1, 2, 3, 4, 5], 1); \
         v_single = var(10); \
         v_empty = var([])",
    )
    .unwrap();

    // Sample variance (N-1): 10 / 4 = 2.5
    assert_eq!(r.env.get("v_w0").unwrap().as_matrix().unwrap().data[0], 2.5);
    assert_eq!(
        r.env.get("v_w0_expl").unwrap().as_matrix().unwrap().data[0],
        2.5
    );

    // Population variance (N): 10 / 5 = 2.0
    assert_eq!(r.env.get("v_w1").unwrap().as_matrix().unwrap().data[0], 2.0);

    // Singleton input returns 0
    assert_eq!(
        r.env.get("v_single").unwrap().as_matrix().unwrap().data[0],
        0.0
    );

    // Empty input returns NaN
    assert!(r.env.get("v_empty").unwrap().as_matrix().unwrap().data[0].is_nan());
}

#[test]
fn std_sample_and_population() {
    let r = run_source(
        "s_w0 = std([1, 2, 3, 4, 5]); \
         s_w1 = std([1, 2, 3, 4, 5], 1); \
         s_single = std(42); \
         s_empty = std([])",
    )
    .unwrap();

    let s0 = r.env.get("s_w0").unwrap().as_matrix().unwrap().data[0];
    assert!((s0 - 2.5f64.sqrt()).abs() < 1e-12);

    let s1 = r.env.get("s_w1").unwrap().as_matrix().unwrap().data[0];
    assert!((s1 - 2.0f64.sqrt()).abs() < 1e-12);

    assert_eq!(
        r.env.get("s_single").unwrap().as_matrix().unwrap().data[0],
        0.0
    );
    assert!(r.env.get("s_empty").unwrap().as_matrix().unwrap().data[0].is_nan());
}

#[test]
fn var_std_matrix_reduction() {
    let r = run_source(
        "M = [1, 2; 3, 4]; \
         v_col = var(M); \
         v_pop_col = var(M, 1); \
         v_row = var(M, 0, 2); \
         v_all = var(M, 0, 'all'); \
         s_col = std(M); \
         s_all = std(M, 1, 'all')",
    )
    .unwrap();

    let v_col = r.env.get("v_col").unwrap().as_matrix().unwrap();
    assert_eq!(v_col.rows, 1);
    assert_eq!(v_col.cols, 2);
    assert_eq!(v_col.data, vec![2.0, 2.0]);

    let v_pop = r.env.get("v_pop_col").unwrap().as_matrix().unwrap();
    assert_eq!(v_pop.rows, 1);
    assert_eq!(v_pop.cols, 2);
    assert_eq!(v_pop.data, vec![1.0, 1.0]);

    let v_row = r.env.get("v_row").unwrap().as_matrix().unwrap();
    assert_eq!(v_row.rows, 2);
    assert_eq!(v_row.cols, 1);
    assert_eq!(v_row.data, vec![0.5, 0.5]);

    // all: var of [1, 2, 3, 4] with w=0: mean=2.5, sum diff sq = 5 -> 5/3
    let v_all = r.env.get("v_all").unwrap().as_matrix().unwrap().data[0];
    assert!((v_all - (5.0 / 3.0)).abs() < 1e-12);

    let s_col = r.env.get("s_col").unwrap().as_matrix().unwrap();
    assert!((s_col.data[0] - 2.0f64.sqrt()).abs() < 1e-12);
    assert!((s_col.data[1] - 2.0f64.sqrt()).abs() < 1e-12);

    // std with w=1 of [1, 2, 3, 4]: var=1.25, std=sqrt(1.25)
    let s_all = r.env.get("s_all").unwrap().as_matrix().unwrap().data[0];
    assert!((s_all - 1.25f64.sqrt()).abs() < 1e-12);
}

#[test]
fn range_basic_and_matrix() {
    let r = run_source(
        "r_vec = range([1, 5, 2]); \
         r_col = range([10; 2; 7]); \
         M = [1, 10; 5, 2]; \
         r_mat = range(M); \
         r_mat_d2 = range(M, 2); \
         r_all = range(M, 'all'); \
         r_empty = range([])",
    )
    .unwrap();

    assert_eq!(
        r.env.get("r_vec").unwrap().as_matrix().unwrap().data[0],
        4.0
    );
    assert_eq!(
        r.env.get("r_col").unwrap().as_matrix().unwrap().data[0],
        8.0
    );

    let r_mat = r.env.get("r_mat").unwrap().as_matrix().unwrap();
    assert_eq!(r_mat.rows, 1);
    assert_eq!(r_mat.cols, 2);
    assert_eq!(r_mat.data, vec![4.0, 8.0]);

    let r_d2 = r.env.get("r_mat_d2").unwrap().as_matrix().unwrap();
    assert_eq!(r_d2.rows, 2);
    assert_eq!(r_d2.cols, 1);
    assert_eq!(r_d2.data, vec![9.0, 3.0]);

    assert_eq!(
        r.env.get("r_all").unwrap().as_matrix().unwrap().data[0],
        9.0
    );
    assert!(r.env.get("r_empty").unwrap().as_matrix().unwrap().data[0].is_nan());
}

#[test]
fn iqr_basic_and_matrix() {
    let r = run_source(
        "iqr_odd = iqr([1, 2, 3, 4, 5]); \
         iqr_even = iqr([1, 2, 3, 4, 5, 6, 7, 8]); \
         M = [1, 10; 2, 20; 3, 30; 4, 40; 5, 50]; \
         iqr_m = iqr(M); \
         iqr_m_all = iqr(M, 'all'); \
         iqr_empty = iqr([])",
    )
    .unwrap();

    // 5 elements: p75=4.25, p25=1.75 -> iqr=2.5 (MATLAB linear empirical quantile)
    assert_eq!(
        r.env.get("iqr_odd").unwrap().as_matrix().unwrap().data[0],
        2.5
    );

    // 8 elements: p75=6.5, p25=2.5 -> iqr=4.0 (MATLAB linear empirical quantile)
    assert_eq!(
        r.env.get("iqr_even").unwrap().as_matrix().unwrap().data[0],
        4.0
    );

    let iqr_m = r.env.get("iqr_m").unwrap().as_matrix().unwrap();
    assert_eq!(iqr_m.rows, 1);
    assert_eq!(iqr_m.cols, 2);
    assert_eq!(iqr_m.data, vec![2.5, 25.0]);

    assert!(r.env.get("iqr_empty").unwrap().as_matrix().unwrap().data[0].is_nan());
}

// ============================================================================
// Group C: Products, Minima, and Maxima — prod, min, max, sum
// ============================================================================

#[test]
fn prod_basic_and_matrix() {
    let r = run_source(
        "p_row = prod([2, 3, 4]); \
         p_col = prod([2; 3; 4]); \
         p_empty = prod([]); \
         M = [1, 2; 3, 4]; \
         p_mat = prod(M); \
         p_mat_d2 = prod(M, 2); \
         p_all = prod(M, 'all')",
    )
    .unwrap();

    assert_eq!(
        r.env.get("p_row").unwrap().as_matrix().unwrap().data[0],
        24.0
    );
    assert_eq!(
        r.env.get("p_col").unwrap().as_matrix().unwrap().data[0],
        24.0
    );
    // MATLAB convention: empty product is 1.0
    assert_eq!(
        r.env.get("p_empty").unwrap().as_matrix().unwrap().data[0],
        1.0
    );

    let p_mat = r.env.get("p_mat").unwrap().as_matrix().unwrap();
    assert_eq!(p_mat.rows, 1);
    assert_eq!(p_mat.cols, 2);
    assert_eq!(p_mat.data, vec![3.0, 8.0]);

    let p_d2 = r.env.get("p_mat_d2").unwrap().as_matrix().unwrap();
    assert_eq!(p_d2.rows, 2);
    assert_eq!(p_d2.cols, 1);
    assert_eq!(p_d2.data, vec![2.0, 12.0]);

    assert_eq!(
        r.env.get("p_all").unwrap().as_matrix().unwrap().data[0],
        24.0
    );
}

#[test]
fn sum_extended_dimensions_and_all() {
    let r = run_source(
        "s_row = sum([1, 2, 3]); \
         s_col = sum([1; 2; 3]); \
         s_empty = sum([]); \
         M = [1, 2; 3, 4]; \
         s_mat = sum(M); \
         s_mat_d2 = sum(M, 2); \
         s_all = sum(M, 'all')",
    )
    .unwrap();

    assert_eq!(
        r.env.get("s_row").unwrap().as_matrix().unwrap().data[0],
        6.0
    );
    assert_eq!(
        r.env.get("s_col").unwrap().as_matrix().unwrap().data[0],
        6.0
    );
    assert_eq!(
        r.env.get("s_empty").unwrap().as_matrix().unwrap().data[0],
        0.0
    );

    let s_mat = r.env.get("s_mat").unwrap().as_matrix().unwrap();
    assert_eq!(s_mat.data, vec![4.0, 6.0]);

    let s_d2 = r.env.get("s_mat_d2").unwrap().as_matrix().unwrap();
    assert_eq!(s_d2.data, vec![3.0, 7.0]);

    assert_eq!(
        r.env.get("s_all").unwrap().as_matrix().unwrap().data[0],
        10.0
    );
}

#[test]
fn min_max_reduction() {
    let r = run_source(
        "min_v = min([3, 1, 4, 1, 5]); \
         max_v = max([3, 1, 4, 1, 5]); \
         M = [3, 6; 1, 8]; \
         min_m = min(M); \
         max_m = max(M); \
         min_m_d2 = min(M, [], 2); \
         max_m_d2 = max(M, [], 2); \
         min_all = min(M, [], 'all'); \
         max_all = max(M, [], 'all')",
    )
    .unwrap();

    assert_eq!(
        r.env.get("min_v").unwrap().as_matrix().unwrap().data[0],
        1.0
    );
    assert_eq!(
        r.env.get("max_v").unwrap().as_matrix().unwrap().data[0],
        5.0
    );

    let min_m = r.env.get("min_m").unwrap().as_matrix().unwrap();
    assert_eq!(min_m.rows, 1);
    assert_eq!(min_m.cols, 2);
    assert_eq!(min_m.data, vec![1.0, 6.0]);

    let max_m = r.env.get("max_m").unwrap().as_matrix().unwrap();
    assert_eq!(max_m.rows, 1);
    assert_eq!(max_m.cols, 2);
    assert_eq!(max_m.data, vec![3.0, 8.0]);

    let min_d2 = r.env.get("min_m_d2").unwrap().as_matrix().unwrap();
    assert_eq!(min_d2.rows, 2);
    assert_eq!(min_d2.cols, 1);
    assert_eq!(min_d2.data, vec![3.0, 1.0]);

    let max_d2 = r.env.get("max_m_d2").unwrap().as_matrix().unwrap();
    assert_eq!(max_d2.rows, 2);
    assert_eq!(max_d2.cols, 1);
    assert_eq!(max_d2.data, vec![6.0, 8.0]);

    assert_eq!(
        r.env.get("min_all").unwrap().as_matrix().unwrap().data[0],
        1.0
    );
    assert_eq!(
        r.env.get("max_all").unwrap().as_matrix().unwrap().data[0],
        8.0
    );
}

#[test]
fn min_max_elementwise_binary() {
    let r = run_source(
        "min_pair = min([1, 5, 2], [3, 2, 4]); \
         max_pair = max([1, 5, 2], [3, 2, 4]); \
         min_scal1 = min([1, 5, 2], 3); \
         max_scal2 = max(3, [1, 5, 2]); \
         M1 = [1, 4; 5, 2]; \
         M2 = [2, 3; 4, 6]; \
         min_m = min(M1, M2); \
         max_m = max(M1, M2)",
    )
    .unwrap();

    let min_pair = r.env.get("min_pair").unwrap().as_matrix().unwrap();
    assert_eq!(min_pair.data, vec![1.0, 2.0, 2.0]);

    let max_pair = r.env.get("max_pair").unwrap().as_matrix().unwrap();
    assert_eq!(max_pair.data, vec![3.0, 5.0, 4.0]);

    let min_scal = r.env.get("min_scal1").unwrap().as_matrix().unwrap();
    assert_eq!(min_scal.data, vec![1.0, 3.0, 2.0]);

    let max_scal = r.env.get("max_scal2").unwrap().as_matrix().unwrap();
    assert_eq!(max_scal.data, vec![3.0, 5.0, 3.0]);

    let min_m = r.env.get("min_m").unwrap().as_matrix().unwrap();
    assert_eq!(min_m.data, vec![1.0, 3.0, 4.0, 2.0]); // row-major: (1,1)=1, (1,2)=3, (2,1)=4, (2,2)=2

    let max_m = r.env.get("max_m").unwrap().as_matrix().unwrap();
    assert_eq!(max_m.data, vec![2.0, 4.0, 5.0, 6.0]);
}

#[test]
fn min_max_multi_output_indices() {
    let r = run_source(
        "[m1, idx1] = min([10, 20, 5, 30]); \
         [m2, idx2] = max([10, 20, 5, 30]); \
         M = [10, 5; 2, 8]; \
         [m_mat_min, idx_mat_min] = min(M); \
         [m_mat_max, idx_mat_max] = max(M); \
         [m_row_min, idx_row_min] = min(M, [], 2); \
         [m_all_min, idx_all_min] = min(M, [], 'all')",
    )
    .unwrap();

    // Vector min: 5 at 1-based index 3
    assert_eq!(r.env.get("m1").unwrap().as_matrix().unwrap().data[0], 5.0);
    assert_eq!(r.env.get("idx1").unwrap().as_matrix().unwrap().data[0], 3.0);

    // Vector max: 30 at 1-based index 4
    assert_eq!(r.env.get("m2").unwrap().as_matrix().unwrap().data[0], 30.0);
    assert_eq!(r.env.get("idx2").unwrap().as_matrix().unwrap().data[0], 4.0);

    // Matrix min along dim 1:
    // col 1: [10; 2] -> min 2 at row 2
    // col 2: [5; 8]  -> min 5 at row 1
    let m_mat_min = r.env.get("m_mat_min").unwrap().as_matrix().unwrap();
    let idx_mat_min = r.env.get("idx_mat_min").unwrap().as_matrix().unwrap();
    assert_eq!(m_mat_min.data, vec![2.0, 5.0]);
    assert_eq!(idx_mat_min.data, vec![2.0, 1.0]);

    // Matrix max along dim 1:
    // col 1: [10; 2] -> max 10 at row 1
    // col 2: [5; 8]  -> max 8 at row 2
    let m_mat_max = r.env.get("m_mat_max").unwrap().as_matrix().unwrap();
    let idx_mat_max = r.env.get("idx_mat_max").unwrap().as_matrix().unwrap();
    assert_eq!(m_mat_max.data, vec![10.0, 8.0]);
    assert_eq!(idx_mat_max.data, vec![1.0, 2.0]);

    // Matrix min along dim 2:
    // row 1: [10, 5] -> min 5 at col 2
    // row 2: [2, 8]  -> min 2 at col 1
    let m_row_min = r.env.get("m_row_min").unwrap().as_matrix().unwrap();
    let idx_row_min = r.env.get("idx_row_min").unwrap().as_matrix().unwrap();
    assert_eq!(m_row_min.data, vec![5.0, 2.0]);
    assert_eq!(idx_row_min.data, vec![2.0, 1.0]);

    // Matrix min all:
    // M in col-major: [10, 2, 5, 8]. Min is 2 at 1-based index 2.
    assert_eq!(
        r.env.get("m_all_min").unwrap().as_matrix().unwrap().data[0],
        2.0
    );
    assert_eq!(
        r.env.get("idx_all_min").unwrap().as_matrix().unwrap().data[0],
        2.0
    );
}

// ============================================================================
// Error Handling & Validation
// ============================================================================

#[test]
fn stats_argument_errors() {
    assert!(run_source("mean()").is_err());
    assert!(run_source("mean(1, 2, 3)").is_err());
    assert!(run_source("mean(1, 0)").is_err());
    assert!(run_source("mean(1, -1)").is_err());
    assert!(run_source("mean(1, 'foo')").is_err());

    assert!(run_source("std()").is_err());
    assert!(run_source("std([1, 2], 2)").is_err());
    assert!(run_source("std([1, 2], -1)").is_err());
    assert!(run_source("std([1, 2], 0, 1, 2)").is_err());

    assert!(run_source("var()").is_err());
    assert!(run_source("var([1, 2], 2)").is_err());

    assert!(run_source("min([1, 2], [1, 2, 3])").is_err());
    assert!(run_source("max([1, 2], [1, 2, 3])").is_err());
    assert!(run_source("min(1, 2, 3, 4)").is_err());
}

#[test]
fn stats_constant_data() {
    let r = run_source(
        "c_var = var([5, 5, 5, 5]); \
         c_std = std([7, 7, 7]); \
         c_range = range([3, 3, 3, 3]); \
         c_iqr = iqr([4, 4, 4, 4, 4]); \
         c_mode = mode([8, 8, 8])",
    )
    .unwrap();

    assert_eq!(
        r.env.get("c_var").unwrap().as_matrix().unwrap().data[0],
        0.0
    );
    assert_eq!(
        r.env.get("c_std").unwrap().as_matrix().unwrap().data[0],
        0.0
    );
    assert_eq!(
        r.env.get("c_range").unwrap().as_matrix().unwrap().data[0],
        0.0
    );
    assert_eq!(
        r.env.get("c_iqr").unwrap().as_matrix().unwrap().data[0],
        0.0
    );
    assert_eq!(
        r.env.get("c_mode").unwrap().as_matrix().unwrap().data[0],
        8.0
    );
}

#[test]
fn stats_empty_reductions() {
    let r = run_source(
        "e_min = min([]); \
         e_max = max([]); \
         e_sum = sum([]); \
         e_prod = prod([]); \
         e_mean = mean([])",
    )
    .unwrap();

    assert_eq!(r.env.get("e_min").unwrap().as_matrix().unwrap().rows, 0);
    assert_eq!(r.env.get("e_max").unwrap().as_matrix().unwrap().rows, 0);
    assert_eq!(
        r.env.get("e_sum").unwrap().as_matrix().unwrap().data[0],
        0.0
    );
    assert_eq!(
        r.env.get("e_prod").unwrap().as_matrix().unwrap().data[0],
        1.0
    );
    assert!(r.env.get("e_mean").unwrap().as_matrix().unwrap().data[0].is_nan());
}
