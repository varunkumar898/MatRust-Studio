//! Deterministic pseudo-random-number generator (session-held).
//!
//! Design:
//! - xoshiro256++ generator, seeded through SplitMix64. Well-documented,
//!   high-quality, tiny, no external dependencies (std only).
//! - `ExecIo` and `Session` carry one `Rng`, so successive `rand` calls in a
//!   session advance the same stream (MATLAB's model). A fresh session always
//!   starts from the same fixed seed, keeping CLI runs and tests reproducible;
//!   `rng(seed)` reseeds explicitly.
//! - `rand` produces values in the open interval (0, 1) (MATLAB documents
//!   rand as (0,1)); samples that degenerate to 0 are rejected.
//! - `randn` uses the Box–Muller transform.

#[derive(Debug, Clone, PartialEq)]
pub struct Rng {
    state: [u64; 4],
    /// One spare variate from the last Box–Muller pair (MATLAB's randn also
    /// consumes its stream in a defined order; keeping the spare avoids
    /// reseeding on every second call while remaining deterministic).
    spare: Option<f64>,
}

impl Default for Rng {
    fn default() -> Self {
        Self::new(DEFAULT_SEED)
    }
}

/// Default seed so unseeded runs are repeatable for tests and demos.
pub const DEFAULT_SEED: u64 = 0x5EED_2026_10_22;

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut state = [0u64; 4];
        // SplitMix64 seeding of four state words.
        let mut sm = seed;
        for slot in state.iter_mut() {
            sm = sm.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = sm;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_8CE4_65AF);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^= z >> 31;
            *slot = z;
        }
        Self { state, spare: None }
    }

    /// xoshiro256++ step.
    pub fn next_u64(&mut self) -> u64 {
        let s = &mut self.state;
        let result = s[0].wrapping_add(s[3]).rotate_left(23).wrapping_add(s[0]);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        result
    }

    /// Uniform (0, 1) double: 52 random bits over the coarsest matrix.
    pub fn rand(&mut self) -> f64 {
        loop {
            let candidate = (self.next_u64() >> 11) as f64 * (-53f64).exp2();
            if candidate > 0.0 && candidate < 1.0 {
                return candidate;
            }
        }
    }

    /// Uniform integer in the inclusive [lo, hi] range; hi must be >= lo.
    /// Values are drawn from the 53-bit float stream to stay uniform.
    pub fn randi_inclusive(&mut self, lo: i64, hi: i64) -> i64 {
        debug_assert!(hi >= lo, "randi range: hi < lo");
        let span = (hi - lo) as f64 + 1.0;
        lo + (self.rand() * span).floor() as i64
    }

    /// Standard normal variate via Box–Muller.
    pub fn randn(&mut self) -> f64 {
        if let Some(spare) = self.spare.take() {
            return spare;
        }
        loop {
            let u1 = self.rand();
            let u2 = self.rand();
            let r = (-2.0 * u1.ln()).sqrt();
            let theta = 2.0 * std::f64::consts::PI * u2;
            let z0 = r * theta.cos();
            let z1 = r * theta.sin();
            if z0.is_finite() {
                self.spare = Some(z1);
                return z0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        let va: Vec<f64> = (0..8).map(|_| a.rand()).collect();
        let vb: Vec<f64> = (0..8).map(|_| b.rand()).collect();
        assert_eq!(va, vb);
    }

    #[test]
    fn rand_is_strictly_in_unit_interval() {
        let mut r = Rng::new(42);
        for _ in 0..10_000 {
            let v = r.rand();
            assert!(v > 0.0 && v < 1.0);
        }
    }

    #[test]
    fn randi_respects_bounds() {
        let mut r = Rng::new(11);
        for _ in 0..10_000 {
            let v = r.randi_inclusive(3, 7);
            assert!((3..=7).contains(&v));
        }
        // Degenerate range always yields the single value.
        assert_eq!(r.randi_inclusive(5, 5), 5);
    }

    #[test]
    fn randn_sphere_stats_are_plausible() {
        let mut r = Rng::new(3);
        let n: f64 = 20_000.0;
        let mean: f64 = (0..20_000).map(|_| r.randn()).sum::<f64>() / n;
        assert!(mean.abs() < 0.05, "mean {mean} too far from 0");
    }

    #[test]
    fn reseeding_resets_the_stream() {
        let mut r = Rng::new(99);
        let first = r.rand();
        r.rand();
        r.rand();
        let mut r2 = Rng::new(99);
        assert_eq!(first, r2.rand());
    }
}
