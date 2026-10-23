//! Gate 0: `eig_with(max_iter)` must honor the caller's iteration limit.
//!
//! The non-symmetric matrix below needs more than one shifted-QR sweep to
//! reduce its Hessenberg form to working precision. With `max_iter = 1` the
//! routine must return the documented non-convergence Diagnostic instead of
//! looping to the default limit or panicking.

use matlab_diagnostics::DiagnosticSeverity;
use matlab_linalg::decomposition::eig_with;
use matlab_values::DenseMatrix;

#[test]
fn eig_with_max_iter_one_reports_nonconvergence() {
    let companion = DenseMatrix {
        rows: 3,
        cols: 3,
        data: vec![
            1.0, 2.0, 3.0, //
            4.0, 5.0, 6.0, //
            7.0, 8.0, 10.0,
        ],
    };

    let err = eig_with(&companion, None, 1)
        .expect_err("one QR iteration must not count as convergence for this matrix");

    assert_eq!(err.severity, DiagnosticSeverity::Error);
    assert_eq!(
        err.message,
        "Eigenvalue iteration did not converge to working precision."
    );
}
