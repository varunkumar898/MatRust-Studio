//! G0.1-VERIFY diagnostic probe: compare public Value::add, a manual slice
//! loop, and the interpreter pipeline in one release profile and one harness.
//! This is an example binary, not part of the shipped `rustlab.exe`; it is kept
//! so the add-path measurement can be reproduced.

use rustlab_math::{DenseArray, Value};
use rustlab_stdlib::new_interpreter;
use std::hint::black_box;
use std::time::Instant;

const N: usize = 1_000_000;

fn bench<F: FnMut()>(name: &str, mut f: F) {
    for _ in 0..5 {
        f();
    }
    let mut times = Vec::new();
    for _ in 0..15 {
        let t0 = Instant::now();
        f();
        times.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!(
        "{name:<22} median {:8.3} ms  min {:8.3}  max {:8.3}",
        times[times.len() / 2],
        times[0],
        times[times.len() - 1]
    );
}

fn main() {
    let da = DenseArray::from_vec_unchecked(vec![N, 1], vec![1.0f64; N]);
    let db = DenseArray::from_vec_unchecked(vec![N, 1], vec![2.0f64; N]);
    let a = Value::Double(da.clone());
    let b = Value::Double(db.clone());

    bench("Value::add", || {
        let _ = black_box(a.add(&b));
    });
    bench("manual zip collect", || {
        let out: Vec<f64> = da
            .as_slice()
            .iter()
            .zip(db.as_slice().iter())
            .map(|(&x, &y)| x + y)
            .collect();
        black_box(out);
    });
    let mut interp = new_interpreter();
    interp
        .eval_script("A = ones(1000000,1); B = 2*ones(1000000,1);")
        .unwrap();
    let stmts = rustlab_parse::parse("C = A + B;").unwrap();
    bench("eval_pipeline", || {
        let _ = black_box(interp.eval_stmts(&stmts));
    });
}
