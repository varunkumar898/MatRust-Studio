use matlab_runtime::Session;
use std::time::{SystemTime, UNIX_EPOCH};

/// Phase D engineering simulation: first-order step response
/// y(t) = 1 - exp(-t/tau), checked against the analytical formula.
#[test]
fn engineering_stage_d_first_order_response() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let mut session = Session::new();
    session.exec("t = 0:0.1:5; tau = 1.0;", None).unwrap();
    session.exec("y = 1 - exp(-t / tau);", None).unwrap();

    let t = session
        .env
        .get("t")
        .and_then(|v| v.as_matrix())
        .expect("t is a matrix");
    let y = session
        .env
        .get("y")
        .and_then(|v| v.as_matrix())
        .expect("y is a matrix");
    assert_eq!((t.rows, t.cols), (1, 51));
    assert_eq!((y.rows, y.cols), (1, 51));
    assert!((y.data[0] - 0.0).abs() < 1e-12);
    assert!((y.data[10] - (1.0 - (-1.0_f64).exp())).abs() < 1e-12);
    assert!((y.data[50] - (1.0 - (-5.0_f64).exp())).abs() < 1e-12);

    session.exec("plot(t, y);", None).unwrap();
    assert_eq!(session.graphics.figures.len(), 1);
    match &session.graphics.figures[0].items[0] {
        matlab_runtime::PlotItem::Line { x, y, .. } => {
            assert_eq!(x.len(), 51);
            assert_eq!(y.len(), 51);
            assert!((x[10] - 1.0).abs() < 1e-12);
            assert!((y[10] - (1.0 - (-1.0_f64).exp())).abs() < 1e-12);
        }
        other => panic!("expected a line plot, got {:?}", other),
    }

    // Export both series. csvwrite writes one matrix per call; the parser does
    // not yet support nested concatenation such as [t' y].
    let csv_t = std::env::temp_dir().join(format!(
        "matrust_phase_d_t_{}_{}.csv",
        std::process::id(),
        nonce
    ));
    let csv_y = std::env::temp_dir().join(format!(
        "matrust_phase_d_y_{}_{}.csv",
        std::process::id(),
        nonce
    ));
    session
        .exec(
            &format!("csvwrite('{}', t');", csv_t.to_string_lossy()),
            None,
        )
        .expect("export t CSV");
    session
        .exec(
            &format!("csvwrite('{}', y');", csv_y.to_string_lossy()),
            None,
        )
        .expect("export y CSV");

    let written_t = std::fs::read_to_string(&csv_t).expect("exported t CSV exists");
    let lines_t: Vec<&str> = written_t.lines().collect();
    assert_eq!(lines_t.len(), 51, "t CSV has one row per sample");
    let first_t: f64 = lines_t[0].trim().parse().expect("numeric first t");
    let last_t: f64 = lines_t[50].trim().parse().expect("numeric last t");
    assert!((first_t - 0.0).abs() < 1e-12);
    assert!((last_t - 5.0).abs() < 1e-12);

    let written_y = std::fs::read_to_string(&csv_y).expect("exported y CSV exists");
    let lines_y: Vec<&str> = written_y.lines().collect();
    assert_eq!(lines_y.len(), 51, "y CSV has one row per sample");
    let first_y: f64 = lines_y[0].trim().parse().expect("numeric first y");
    let last_y: f64 = lines_y[50].trim().parse().expect("numeric last y");
    assert!((first_y - 0.0).abs() < 1e-12);
    assert!((last_y - (1.0 - (-5.0_f64).exp())).abs() < 1e-12);

    session
        .exec(
            &format!("back_t = csvread('{}');", csv_t.to_string_lossy()),
            None,
        )
        .expect("read exported t CSV");
    let back_t = session
        .env
        .get("back_t")
        .and_then(|v| v.as_matrix())
        .expect("read-back t matrix");
    assert_eq!((back_t.rows, back_t.cols), (51, 1));
    assert!((back_t.data[0] - 0.0).abs() < 1e-12);
    assert!((back_t.data[50] - 5.0).abs() < 1e-12);

    session
        .exec(
            &format!("back_y = csvread('{}');", csv_y.to_string_lossy()),
            None,
        )
        .expect("read exported y CSV");
    let back_y = session
        .env
        .get("back_y")
        .and_then(|v| v.as_matrix())
        .expect("read-back y matrix");
    assert_eq!((back_y.rows, back_y.cols), (51, 1));
    assert!((back_y.data[0] - 0.0).abs() < 1e-12);
    assert!((back_y.data[50] - (1.0 - (-5.0_f64).exp())).abs() < 1e-12);

    std::fs::remove_file(&csv_t).expect("t temporary CSV cleaned up");
    std::fs::remove_file(&csv_y).expect("y temporary CSV cleaned up");
}
