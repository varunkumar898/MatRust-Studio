use matlab_diagnostics::{Diagnostic, Span};
use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use crate::matrix::DenseMatrix;

/// Double-precision complex number `re + im*i`.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Complex64 {
    pub re: f64,
    pub im: f64,
}

impl Complex64 {
    #[inline(always)]
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    #[inline(always)]
    pub const fn real(re: f64) -> Self {
        Self { re, im: 0.0 }
    }

    #[inline(always)]
    pub const fn imag(im: f64) -> Self {
        Self { re: 0.0, im }
    }

    #[inline(always)]
    pub const fn i() -> Self {
        Self { re: 0.0, im: 1.0 }
    }

    #[inline(always)]
    pub const fn zero() -> Self {
        Self { re: 0.0, im: 0.0 }
    }

    #[inline(always)]
    pub const fn one() -> Self {
        Self { re: 1.0, im: 0.0 }
    }

    #[inline(always)]
    pub fn is_real(&self) -> bool {
        self.im == 0.0
    }

    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        self.re == 0.0 && self.im == 0.0
    }

    #[inline(always)]
    pub fn is_nan(&self) -> bool {
        self.re.is_nan() || self.im.is_nan()
    }

    #[inline(always)]
    pub fn is_infinite(&self) -> bool {
        self.re.is_infinite() || self.im.is_infinite()
    }

    #[inline(always)]
    pub fn is_finite(&self) -> bool {
        self.re.is_finite() && self.im.is_finite()
    }

    #[inline(always)]
    pub fn conj(self) -> Self {
        Self::new(self.re, -self.im)
    }

    #[inline(always)]
    pub fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }

    #[inline(always)]
    pub fn abs_sq(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    #[inline(always)]
    pub fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }

    /// Complex square root using stable branch cuts.
    pub fn sqrt(self) -> Self {
        if self.im == 0.0 {
            if self.re >= 0.0 {
                Self::real(self.re.sqrt())
            } else {
                Self::new(0.0, (-self.re).sqrt())
            }
        } else {
            let r = self.abs();
            let u = ((r + self.re.abs()) / 2.0).sqrt();
            let v = self.im.abs() / (2.0 * u);
            let sign = if self.im >= 0.0 { 1.0 } else { -1.0 };
            if self.re >= 0.0 {
                Self::new(u, sign * v)
            } else {
                Self::new(v, sign * u)
            }
        }
    }

    /// Complex exponential `e^z = e^re * (cos(im) + i*sin(im))`.
    pub fn exp(self) -> Self {
        let r = self.re.exp();
        Self::new(r * self.im.cos(), r * self.im.sin())
    }

    /// Principal complex natural logarithm `ln(z) = ln(|z|) + i*arg(z)`.
    pub fn ln(self) -> Self {
        Self::new(self.abs().ln(), self.arg())
    }

    /// Complex power with real scalar exponent `z^p = e^(p * ln(z))`.
    pub fn powf(self, p: f64) -> Self {
        if self.im == 0.0 && self.re >= 0.0 {
            Self::real(self.re.powf(p))
        } else {
            (self.ln() * Self::real(p)).exp()
        }
    }

    /// Complex power with complex exponent `z^w = e^(w * ln(z))`.
    pub fn powc(self, w: Complex64) -> Self {
        (self.ln() * w).exp()
    }

    /// Concise string representation e.g. `3 + 4i` or `-2i`
    pub fn format_concise(&self) -> String {
        if self.is_nan() {
            return "NaN + NaNi".to_string();
        }
        if self.im == 0.0 {
            format!("{}", self.re)
        } else if self.re == 0.0 {
            format!("{}i", self.im)
        } else if self.im < 0.0 {
            format!("{} - {}i", self.re, -self.im)
        } else {
            format!("{} + {}i", self.re, self.im)
        }
    }

    /// Tabular MATLAB 4-decimal formatting.
    pub fn format_matlab(&self) -> String {
        if self.is_nan() {
            return "       NaN".to_string();
        }
        if self.im == 0.0 {
            format!("{:10.4}", self.re)
        } else if self.im < 0.0 {
            format!("{:10.4} - {:10.4}i", self.re, -self.im)
        } else {
            format!("{:10.4} + {:10.4}i", self.re, self.im)
        }
    }
}

impl fmt::Display for Complex64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_concise())
    }
}

impl Add for Complex64 {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.re + rhs.re, self.im + rhs.im)
    }
}

impl AddAssign for Complex64 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

impl Sub for Complex64 {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.re - rhs.re, self.im - rhs.im)
    }
}

impl SubAssign for Complex64 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.re -= rhs.re;
        self.im -= rhs.im;
    }
}

impl Neg for Complex64 {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self::Output {
        Self::new(-self.re, -self.im)
    }
}

impl Mul for Complex64 {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(
            self.re * rhs.re - self.im * rhs.im,
            self.re * rhs.im + self.im * rhs.re,
        )
    }
}

impl MulAssign for Complex64 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Div for Complex64 {
    type Output = Self;
    /// Robust complex division via Smith's algorithm to prevent intermediate overflow/underflow.
    fn div(self, rhs: Self) -> Self::Output {
        if rhs.re.abs() >= rhs.im.abs() {
            if rhs.re == 0.0 {
                return Self::new(f64::NAN, f64::NAN);
            }
            let ratio = rhs.im / rhs.re;
            let denom = rhs.re + rhs.im * ratio;
            Self::new(
                (self.re + self.im * ratio) / denom,
                (self.im - self.re * ratio) / denom,
            )
        } else {
            let ratio = rhs.re / rhs.im;
            let denom = rhs.re * ratio + rhs.im;
            Self::new(
                (self.re * ratio + self.im) / denom,
                (self.im * ratio - self.re) / denom,
            )
        }
    }
}

impl DivAssign for Complex64 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

/// A 2D dense matrix of complex double-precision numbers.
#[derive(Debug, Clone, PartialEq)]
pub struct ComplexDenseMatrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<Complex64>,
}

impl ComplexDenseMatrix {
    pub fn empty() -> Self {
        Self {
            rows: 0,
            cols: 0,
            data: Vec::new(),
        }
    }

    pub fn scalar(val: Complex64) -> Self {
        Self {
            rows: 1,
            cols: 1,
            data: vec![val],
        }
    }

    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![Complex64::zero(); rows * cols],
        }
    }

    pub fn ones(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![Complex64::one(); rows * cols],
        }
    }

    pub fn eye(n: usize) -> Self {
        let mut data = vec![Complex64::zero(); n * n];
        for i in 0..n {
            data[i * n + i] = Complex64::one();
        }
        Self {
            rows: n,
            cols: n,
            data,
        }
    }

    /// Promote a real `DenseMatrix` to `ComplexDenseMatrix` with zero imaginary parts.
    pub fn from_real(m: &DenseMatrix) -> Self {
        let data = m.data.iter().map(|&x| Complex64::real(x)).collect();
        Self {
            rows: m.rows,
            cols: m.cols,
            data,
        }
    }

    /// Construct from real and imaginary matrices with broadcasting.
    pub fn from_parts(
        re: &DenseMatrix,
        im: &DenseMatrix,
        span: Option<Span>,
    ) -> Result<Self, Diagnostic> {
        if re.is_scalar() && !im.is_scalar() {
            let s_re = re.data[0];
            let data = im.data.iter().map(|&im_val| Complex64::new(s_re, im_val)).collect();
            return Ok(Self {
                rows: im.rows,
                cols: im.cols,
                data,
            });
        }
        if !re.is_scalar() && im.is_scalar() {
            let s_im = im.data[0];
            let data = re.data.iter().map(|&re_val| Complex64::new(re_val, s_im)).collect();
            return Ok(Self {
                rows: re.rows,
                cols: re.cols,
                data,
            });
        }
        if re.rows != im.rows || re.cols != im.cols {
            return Err(Diagnostic::error(
                format!(
                    "Arrays have incompatible sizes for complex constructor: {}x{} vs {}x{}",
                    re.rows, re.cols, im.rows, im.cols
                ),
                span,
            ));
        }
        let data = re
            .data
            .iter()
            .zip(im.data.iter())
            .map(|(&r, &i)| Complex64::new(r, i))
            .collect();
        Ok(Self {
            rows: re.rows,
            cols: re.cols,
            data,
        })
    }

    pub fn from_rows(
        rows_data: Vec<Vec<Complex64>>,
        span: Option<Span>,
    ) -> Result<Self, Diagnostic> {
        if rows_data.is_empty() {
            return Ok(Self::empty());
        }
        let cols = rows_data[0].len();
        let rows = rows_data.len();
        let mut data = Vec::with_capacity(rows * cols);
        for (r_idx, row) in rows_data.into_iter().enumerate() {
            if row.len() != cols {
                return Err(Diagnostic::error(
                    format!(
                        "Dimensions of arrays being concatenated are not consistent: row 1 has {} elements, but row {} has {}",
                        cols, r_idx + 1, row.len()
                    ),
                    span,
                ));
            }
            data.extend(row);
        }
        Ok(Self { rows, cols, data })
    }

    #[inline]
    pub fn is_scalar(&self) -> bool {
        self.rows == 1 && self.cols == 1
    }

    #[inline]
    pub fn as_scalar(&self) -> Option<Complex64> {
        if self.is_scalar() {
            self.data.first().copied()
        } else {
            None
        }
    }

    /// Extract real part as a real `DenseMatrix`.
    pub fn real_part(&self) -> DenseMatrix {
        DenseMatrix {
            rows: self.rows,
            cols: self.cols,
            data: self.data.iter().map(|z| z.re).collect(),
        }
    }

    /// Extract imaginary part as a real `DenseMatrix`.
    pub fn imag_part(&self) -> DenseMatrix {
        DenseMatrix {
            rows: self.rows,
            cols: self.cols,
            data: self.data.iter().map(|z| z.im).collect(),
        }
    }

    /// Check if all imaginary parts are zero.
    pub fn is_all_real(&self) -> bool {
        self.data.iter().all(|z| z.is_real())
    }

    /// Convert to real matrix by taking real parts.
    pub fn to_real_matrix(&self) -> DenseMatrix {
        self.real_part()
    }

    /// 1-based 2D indexing: get element at (row, col)
    pub fn get_2d(&self, r1: usize, c1: usize, span: Option<Span>) -> Result<Complex64, Diagnostic> {
        if r1 == 0 || r1 > self.rows || c1 == 0 || c1 > self.cols {
            return Err(Diagnostic::error(
                format!(
                    "Index exceeds matrix dimensions: index ({}, {}) on {}x{} matrix",
                    r1, c1, self.rows, self.cols
                ),
                span,
            ));
        }
        let r0 = r1 - 1;
        let c0 = c1 - 1;
        Ok(self.data[r0 * self.cols + c0])
    }

    /// 1-based 2D indexing: set element at (row, col)
    pub fn set_2d(
        &mut self,
        r1: usize,
        c1: usize,
        val: Complex64,
        span: Option<Span>,
    ) -> Result<(), Diagnostic> {
        if r1 == 0 || r1 > self.rows || c1 == 0 || c1 > self.cols {
            return Err(Diagnostic::error(
                format!(
                    "Index exceeds matrix dimensions: assignment ({}, {}) on {}x{} matrix",
                    r1, c1, self.rows, self.cols
                ),
                span,
            ));
        }
        let r0 = r1 - 1;
        let c0 = c1 - 1;
        self.data[r0 * self.cols + c0] = val;
        Ok(())
    }

    /// 1-based column-major linear indexing: get element
    pub fn get_linear(&self, idx1: usize, span: Option<Span>) -> Result<Complex64, Diagnostic> {
        let total = self.rows * self.cols;
        if idx1 == 0 || idx1 > total {
            return Err(Diagnostic::error(
                format!(
                    "Index exceeds matrix dimensions: linear index {} on {}x{} (numel={})",
                    idx1, self.rows, self.cols, total
                ),
                span,
            ));
        }
        let k = idx1 - 1;
        let r = k % self.rows;
        let c = k / self.rows;
        Ok(self.data[r * self.cols + c])
    }

    /// 1-based column-major linear indexing: set element
    pub fn set_linear(
        &mut self,
        idx1: usize,
        val: Complex64,
        span: Option<Span>,
    ) -> Result<(), Diagnostic> {
        let total = self.rows * self.cols;
        if idx1 == 0 || idx1 > total {
            return Err(Diagnostic::error(
                format!(
                    "Index exceeds matrix dimensions: linear assignment {} on {}x{} (numel={})",
                    idx1, self.rows, self.cols, total
                ),
                span,
            ));
        }
        let k = idx1 - 1;
        let r = k % self.rows;
        let c = k / self.rows;
        self.data[r * self.cols + c] = val;
        Ok(())
    }

    /// Format matching MATLAB display conventions.
    pub fn format_matlab(&self) -> String {
        if self.rows == 0 || self.cols == 0 {
            return format!("   [] ({}x{} complex double)\n", self.rows, self.cols);
        }
        if self.is_scalar() {
            return format!("   {}\n", self.data[0].format_concise());
        }

        let mut out = String::new();
        for r in 0..self.rows {
            out.push_str("  ");
            for c in 0..self.cols {
                let z = self.data[r * self.cols + c];
                out.push_str(&z.format_matlab());
                if c + 1 < self.cols {
                    out.push_str("   ");
                }
            }
            out.push('\n');
        }
        out
    }
}

impl fmt::Display for ComplexDenseMatrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_matlab())
    }
}
