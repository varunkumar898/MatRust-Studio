// Temporary scratch: raw residual diagnostics for the companion 5x5 and a copy
// of the seeded 4x4 sweep with per-column numbers.
use matlab_linalg::decomposition::eig_general;
use matlab_values::{Complex64, ComplexDenseMatrix, DenseMatrix};

struct Rng(u64);
impl Rng {
    fn next_f64(&mut self) -> f64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        let frac = (x >> 11) as f64 / (1u64 << 53) as f64;
        (frac - 0.5) * 10.0
    }
}

#[test]
fn debug_raw_residuals() {
    // Companion of (x-1)(x-2)(x-3)(x-4)(x-5)
    let comp = DenseMatrix {
        rows: 5,
        cols: 5,
        data: vec![
            15.0, 1.0, 0.0, 0.0, 0.0, -85.0, 0.0, 1.0, 0.0, 0.0, 225.0, 0.0, 0.0, 1.0, 0.0, -274.0,
            0.0, 0.0, 0.0, 1.0, 120.0, 0.0, 0.0, 0.0, 0.0,
        ],
    };
    let frob = comp.data.iter().map(|x| x * x).sum::<f64>().sqrt();
    let bound = 100.0 * f64::EPSILON * 5.0 * frob;
    let (d, v) = eig_general(&comp, None).unwrap();
    let mut total2 = 0.0f64;
    for j in 0..5 {
        let mut acc = 0.0f64;
        for i in 0..5 {
            let mut s = Complex64::new(0.0, 0.0);
            for k in 0..5 {
                s = s + Complex64::real(comp.data[i * 5 + k]) * v.data[k * 5 + j];
            }
            s = s - d.data[j] * v.data[i * 5 + j];
            acc += s.re * s.re + s.im * s.im;
        }
        total2 += acc;
        println!(
            "companion col{j}: |r|2={:.3e} d={:.13e}+{:.3e}i",
            acc.sqrt(),
            d.data[j].re,
            d.data[j].im
        );
    }
    println!(
        "companion: |A V - V D|_F = {:.3e}; c=100 bound = {:.3e}; ratio = {:.2e}",
        total2.sqrt(),
        bound,
        total2.sqrt() / bound
    );
}
