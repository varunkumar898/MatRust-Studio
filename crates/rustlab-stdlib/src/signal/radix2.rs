//! Educational reference implementation: Radix-2 Cooley-Tukey Discrete Fourier Transform.
//!
//! # Theory
//! The radix-2 Cooley-Tukey algorithm evaluates the DFT in O(N log N) time for arrays
//! whose length N is a power of 2, based on the Danielson-Lanczos lemma (1942):
//!
//! A DFT of length N can be decomposed into two interleaved DFTs of length N/2:
//! X[k] = E[k] + e^(-2*pi*i*k/N) * O[k]
//! X[k + N/2] = E[k] - e^(-2*pi*i*k/N) * O[k]
//!
//! where E[k] is the DFT of even-indexed elements, and O[k] is the DFT of odd-indexed elements.

use num_complex::Complex64;
use std::f64::consts::PI;

/// Computes the 1D Discrete Fourier Transform of a complex slice with length 2^k.
pub fn fft_radix2_reference(input: &[Complex64], inverse: bool) -> Vec<Complex64> {
    let n = input.len();
    if n <= 1 {
        return input.to_vec();
    }
    assert!(
        n.is_power_of_two(),
        "Radix-2 FFT requires length to be a power of 2"
    );

    let even: Vec<Complex64> = input.iter().step_by(2).copied().collect();
    let odd: Vec<Complex64> = input.iter().skip(1).step_by(2).copied().collect();

    let fft_even = fft_radix2_reference(&even, inverse);
    let fft_odd = fft_radix2_reference(&odd, inverse);

    let mut combined = vec![Complex64::new(0.0, 0.0); n];
    let sign = if inverse { 1.0 } else { -1.0 };

    for k in 0..(n / 2) {
        let angle = sign * 2.0 * PI * (k as f64) / (n as f64);
        let twiddle = Complex64::new(angle.cos(), angle.sin()) * fft_odd[k];
        combined[k] = fft_even[k] + twiddle;
        combined[k + n / 2] = fft_even[k] - twiddle;
    }

    if inverse {
        for x in &mut combined {
            *x *= 0.5;
        }
    }
    combined
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radix2_roundtrip() {
        let input = vec![
            Complex64::new(1.0, 0.0),
            Complex64::new(2.0, 0.0),
            Complex64::new(3.0, 0.0),
            Complex64::new(4.0, 0.0),
        ];
        let forward = fft_radix2_reference(&input, false);
        let recovered = fft_radix2_reference(&forward, true);

        for (orig, rec) in input.iter().zip(recovered.iter()) {
            assert!((orig.re - rec.re).abs() < 1e-12);
            assert!((orig.im - rec.im).abs() < 1e-12);
        }
    }
}
