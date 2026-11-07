//! "Nice ticks" rule for axis tick placement.
//!
//! Given a data range `[min, max]` and a target number of ticks, compute a
//! step size snapped to a "nice" number (1, 2, or 5 times a power of ten),
//! then emit tick positions from `floor(min / step) * step` up through
//! `ceil(max / step) * step`. This keeps tick labels on round numbers and
//! produces deterministic, stable output across platforms.
//!
//! Reference rule: D3 `ticks.step` / matplotlib `MaxNLocator` style nice
//! number selection, adapted to RustLab's clean-room specification.

/// Return nice tick positions covering `[min, max]` with about `count` ticks.
pub fn nice_ticks(min: f64, max: f64, count: usize) -> Vec<f64> {
    if count < 2 {
        return vec![min, max];
    }
    if min.is_nan() || max.is_nan() {
        return vec![min, max];
    }
    if (min - max).abs() < 1e-12 {
        // Degenerate range: fabricate a small span around the single value.
        let center = min;
        let half = if center.abs() < 1e-12 {
            0.5
        } else {
            center.abs() * 0.05
        };
        return nice_ticks(center - half, center + half, count);
    }
    let range = max - min;
    let raw_step = range / (count - 1) as f64;
    let step = nice_step(raw_step);
    let start = (min / step).floor() * step;
    let mut ticks = Vec::new();
    let mut t = start;
    // Small epsilon guards against floating-point drift past `max`.
    let eps = step * 1e-9;
    while t <= max + eps {
        // Snap tiny floating-point noise so identical math yields identical text.
        ticks.push((t * 1e6).round() / 1e6);
        t += step;
    }
    ticks
}

/// Snap `step` to a "nice" number: mantissa in {1, 2, 5} × 10ⁿ.
fn nice_step(step: f64) -> f64 {
    if step <= 0.0 || !step.is_finite() {
        return 1.0;
    }
    let magnitude = 10.0_f64.powi((step.log10().floor() as i32).clamp(-20, 20));
    let residual = step / magnitude;
    let mantissa = if residual <= 1.0 {
        1.0
    } else if residual <= 2.0 {
        2.0
    } else if residual <= 5.0 {
        5.0
    } else {
        10.0
    };
    magnitude * mantissa
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_are_monotonic_and_cover_range() {
        let ticks = nice_ticks(0.0, 10.0, 6);
        assert!(ticks.windows(2).all(|w| w[1] > w[0]));
        assert!((ticks[0] - 0.0).abs() < 1e-9);
        assert!(ticks.last().copied().unwrap_or(0.0) >= 10.0 - 1e-9);
    }

    #[test]
    fn ticks_use_nice_steps() {
        let ticks = nice_ticks(0.0, 100.0, 6);
        // Expect steps like 20: 0, 20, 40, 60, 80, 100
        assert!(ticks.len() >= 2);
        let step = ticks[1] - ticks[0];
        assert!((step - 20.0).abs() < 1e-9 || (step - 25.0).abs() < 1e-9);
    }

    #[test]
    fn degenerate_range_does_not_panic() {
        let ticks = nice_ticks(3.0, 3.0, 5);
        assert!(ticks.len() >= 2);
        assert!(ticks.iter().all(|t| t.is_finite()));
    }

    #[test]
    fn negative_range() {
        let ticks = nice_ticks(-50.0, 10.0, 7);
        assert!(ticks.windows(2).all(|w| w[1] > w[0]));
        assert!(ticks[0] <= -50.0 + 1e-9);
        assert!(ticks.last().copied().unwrap_or(0.0) >= 10.0 - 1e-9);
    }

    #[test]
    fn deterministic_across_calls() {
        let a = nice_ticks(-3.3, 7.7, 8);
        let b = nice_ticks(-3.3, 7.7, 8);
        assert_eq!(a, b);
    }
}
