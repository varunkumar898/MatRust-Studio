//! The unified MATLAB `Value` enum and dynamic arithmetic dispatch.
//!
//! Implements strict MATLAB numerical type semantics:
//! - Full numeric hierarchy: single, double, complex, int8..int64, uint8..uint64, sparse CSC.
//! - Saturating integer arithmetic without overflow.
//! - Round-half-away-from-zero conversions.
//! - Exact 64-bit integers without float loss.
//! - Type combination rules (int + double -> int; intA + intB -> error; single + double -> single).

use crate::array::DenseArray;
use crate::error::MathError;
use crate::ops::{elementwise_binary, elementwise_unary, matmul_2d};
use crate::sparse::CscMatrix;
use num_complex::Complex64;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// Trait for saturating conversion from `f64` into integer types (rounding half away from zero).
pub trait SaturatingCast<T> {
    fn saturating_cast(self) -> T;
}

macro_rules! impl_saturating_float_to_signed {
    ($int_type:ident, $min:expr, $max:expr) => {
        impl SaturatingCast<$int_type> for f64 {
            #[inline]
            fn saturating_cast(self) -> $int_type {
                if self.is_nan() {
                    0
                } else if self >= ($max as f64) {
                    $max
                } else if self <= ($min as f64) {
                    $min
                } else {
                    let rounded = if self >= 0.0 {
                        (self + 0.5).floor()
                    } else {
                        (self - 0.5).ceil()
                    };
                    rounded as $int_type
                }
            }
        }
    };
}

macro_rules! impl_saturating_float_to_unsigned {
    ($int_type:ident, $max:expr) => {
        impl SaturatingCast<$int_type> for f64 {
            #[inline]
            fn saturating_cast(self) -> $int_type {
                if self.is_nan() || self <= 0.0 {
                    0
                } else if self >= ($max as f64) {
                    $max
                } else {
                    let rounded = (self + 0.5).floor();
                    rounded as $int_type
                }
            }
        }
    };
}

impl_saturating_float_to_signed!(i8, i8::MIN, i8::MAX);
impl_saturating_float_to_signed!(i16, i16::MIN, i16::MAX);
impl_saturating_float_to_signed!(i32, i32::MIN, i32::MAX);
impl_saturating_float_to_signed!(i64, i64::MIN, i64::MAX);

impl_saturating_float_to_unsigned!(u8, u8::MAX);
impl_saturating_float_to_unsigned!(u16, u16::MAX);
impl_saturating_float_to_unsigned!(u32, u32::MAX);
impl_saturating_float_to_unsigned!(u64, u64::MAX);

/// Exact integer division rounded half away from zero.
macro_rules! impl_signed_idiv_round {
    ($fn_name:ident, $t:ident) => {
        #[inline]
        pub fn $fn_name(a: $t, b: $t) -> $t {
            if b == 0 {
                return if a > 0 {
                    $t::MAX
                } else if a < 0 {
                    $t::MIN
                } else {
                    0
                };
            }
            if a == $t::MIN && b == -1 {
                return $t::MAX;
            }
            let q = a / b;
            let r = a % b;
            // Over-flow safe comparison: b.abs() - r.abs() <= r.abs()
            let b_abs = if b == $t::MIN {
                $t::MAX as u64 + 1
            } else {
                b.unsigned_abs() as u64
            };
            let r_abs = if r == $t::MIN {
                $t::MAX as u64 + 1
            } else {
                r.unsigned_abs() as u64
            };
            if b_abs - r_abs <= r_abs {
                if (a > 0 && b > 0) || (a < 0 && b < 0) {
                    q.saturating_add(1)
                } else {
                    q.saturating_sub(1)
                }
            } else {
                q
            }
        }
    };
}

macro_rules! impl_unsigned_idiv_round {
    ($fn_name:ident, $t:ident) => {
        #[inline]
        pub fn $fn_name(a: $t, b: $t) -> $t {
            if b == 0 {
                return if a > 0 { $t::MAX } else { 0 };
            }
            let q = a / b;
            let r = a % b;
            if b - r <= r {
                q.saturating_add(1)
            } else {
                q
            }
        }
    };
}

impl_signed_idiv_round!(idiv_round_i8, i8);
impl_signed_idiv_round!(idiv_round_i16, i16);
impl_signed_idiv_round!(idiv_round_i32, i32);
impl_signed_idiv_round!(idiv_round_i64, i64);

impl_unsigned_idiv_round!(idiv_round_u8, u8);
impl_unsigned_idiv_round!(idiv_round_u16, u16);
impl_unsigned_idiv_round!(idiv_round_u32, u32);
impl_unsigned_idiv_round!(idiv_round_u64, u64);

/// A first-class MATLAB function handle: `@sin` or anonymous `@(x, y) x + y + a`.
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionHandle {
    /// Named function handle, e.g. `@sin`, `@magic`.
    Named(String),
    /// Anonymous function with parameter names, body AST, and lexically captured closure variables.
    Anonymous {
        params: Vec<String>,
        body: rustlab_parse::Expr,
        closure_env: HashMap<String, Value>,
    },
}

/// An N-dimensional MATLAB structure array with named fields.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StructArray {
    pub shape: Vec<usize>,
    pub fields: Vec<String>,
    pub data: Vec<HashMap<String, Value>>,
}

impl StructArray {
    pub fn new(shape: Vec<usize>, fields: Vec<String>, data: Vec<HashMap<String, Value>>) -> Self {
        Self {
            shape,
            fields,
            data,
        }
    }

    pub fn scalar(map: HashMap<String, Value>) -> Self {
        let fields = map.keys().cloned().collect();
        Self {
            shape: vec![1, 1],
            fields,
            data: vec![map],
        }
    }

    pub fn empty() -> Self {
        Self {
            shape: vec![0, 0],
            fields: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn get_field(&self, field: &str) -> Option<&Value> {
        self.data.first().and_then(|m| m.get(field))
    }

    pub fn set_field(&mut self, field: &str, val: Value) {
        if self.data.is_empty() {
            let mut map = HashMap::new();
            map.insert(field.to_string(), val);
            self.data = vec![map];
            self.shape = vec![1, 1];
            self.fields = vec![field.to_string()];
        } else {
            if !self.fields.iter().any(|f| f == field) {
                self.fields.push(field.to_string());
            }
            self.data[0].insert(field.to_string(), val);
        }
    }
}

/// Unified representation of any MATLAB runtime array value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Double precision 64-bit float matrix (default).
    Double(DenseArray<f64>),
    /// Complex double precision matrix.
    ComplexDouble(DenseArray<Complex64>),
    /// Single precision 32-bit float matrix.
    Single(DenseArray<f32>),
    /// Logical boolean array.
    Logical(DenseArray<bool>),
    /// 8-bit signed integer array.
    Int8(DenseArray<i8>),
    /// 16-bit signed integer array.
    Int16(DenseArray<i16>),
    /// 32-bit signed integer array.
    Int32(DenseArray<i32>),
    /// 64-bit signed integer array.
    Int64(DenseArray<i64>),
    /// 8-bit unsigned integer array.
    UInt8(DenseArray<u8>),
    /// 16-bit unsigned integer array.
    UInt16(DenseArray<u16>),
    /// 32-bit unsigned integer array.
    UInt32(DenseArray<u32>),
    /// 64-bit unsigned integer array.
    UInt64(DenseArray<u64>),
    /// Character array (row vector of characters).
    Char(DenseArray<char>),
    /// String array (array of string objects).
    String(DenseArray<String>),
    /// Cell array holding heterogeneous Values.
    Cell(DenseArray<Value>),
    /// Structure array with named fields.
    Struct(StructArray),
    /// First-class function handle (`@sin` or anonymous `@(x) ...`).
    FunctionHandle(Arc<FunctionHandle>),
    /// Compressed Sparse Column matrix of doubles.
    Sparse(CscMatrix<f64>),
}

impl Default for Value {
    fn default() -> Self {
        Self::empty()
    }
}

impl Value {
    /// Constructs a scalar `double` value.
    pub fn scalar(val: f64) -> Self {
        Self::Double(DenseArray::scalar(val))
    }

    /// Constructs a scalar `logical` value.
    pub fn logical_scalar(val: bool) -> Self {
        Self::Logical(DenseArray::scalar(val))
    }

    /// Constructs an empty `0 x 0` double matrix.
    pub fn empty() -> Self {
        Self::Double(DenseArray::empty())
    }

    /// Constructs a character vector from a string slice: `'hello'`.
    pub fn char_vector(s: &str) -> Self {
        let chars: Vec<char> = s.chars().collect();
        Self::Char(DenseArray::row_vector(chars))
    }

    /// Constructs a $1 \times 1$ scalar string: `"hello"`.
    pub fn string_scalar(s: &str) -> Self {
        Self::String(DenseArray::scalar(s.to_string()))
    }

    /// Constructs a named function handle: `@sin`.
    pub fn function_handle_named(name: &str) -> Self {
        Self::FunctionHandle(Arc::new(FunctionHandle::Named(name.to_string())))
    }

    /// Constructs an anonymous function handle with captured closure variables.
    pub fn function_handle_anon(
        params: Vec<String>,
        body: rustlab_parse::Expr,
        closure_env: HashMap<String, Value>,
    ) -> Self {
        Self::FunctionHandle(Arc::new(FunctionHandle::Anonymous {
            params,
            body,
            closure_env,
        }))
    }

    /// Returns the shape slice of the underlying array.
    pub fn shape(&self) -> &[usize] {
        match self {
            Self::Double(a) => a.shape(),
            Self::ComplexDouble(a) => a.shape(),
            Self::Single(a) => a.shape(),
            Self::Logical(a) => a.shape(),
            Self::Int8(a) => a.shape(),
            Self::Int16(a) => a.shape(),
            Self::Int32(a) => a.shape(),
            Self::Int64(a) => a.shape(),
            Self::UInt8(a) => a.shape(),
            Self::UInt16(a) => a.shape(),
            Self::UInt32(a) => a.shape(),
            Self::UInt64(a) => a.shape(),
            Self::Char(a) => a.shape(),
            Self::String(a) => a.shape(),
            Self::Cell(a) => a.shape(),
            Self::Struct(s) => &s.shape,
            Self::FunctionHandle(_) => &[1, 1],
            Self::Sparse(s) => &s.shape,
        }
    }

    /// Returns the total number of elements.
    pub fn numel(&self) -> usize {
        match self {
            Self::Double(a) => a.numel(),
            Self::ComplexDouble(a) => a.numel(),
            Self::Single(a) => a.numel(),
            Self::Logical(a) => a.numel(),
            Self::Int8(a) => a.numel(),
            Self::Int16(a) => a.numel(),
            Self::Int32(a) => a.numel(),
            Self::Int64(a) => a.numel(),
            Self::UInt8(a) => a.numel(),
            Self::UInt16(a) => a.numel(),
            Self::UInt32(a) => a.numel(),
            Self::UInt64(a) => a.numel(),
            Self::Char(a) => a.numel(),
            Self::String(a) => a.numel(),
            Self::Cell(a) => a.numel(),
            Self::Struct(s) => s.shape.iter().product(),
            Self::FunctionHandle(_) => 1,
            Self::Sparse(s) => s.rows() * s.cols(),
        }
    }

    /// Returns true if the value is empty.
    pub fn is_empty(&self) -> bool {
        self.numel() == 0
    }

    /// Returns true if the value is a $1 \times 1$ scalar.
    pub fn is_scalar(&self) -> bool {
        self.shape() == [1, 1]
    }

    /// True if value is an integer type (int8..uint64).
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Self::Int8(_)
                | Self::Int16(_)
                | Self::Int32(_)
                | Self::Int64(_)
                | Self::UInt8(_)
                | Self::UInt16(_)
                | Self::UInt32(_)
                | Self::UInt64(_)
        )
    }

    /// True if value is a floating-point type (double, single, complex double, sparse).
    pub fn is_float(&self) -> bool {
        matches!(
            self,
            Self::Double(_) | Self::ComplexDouble(_) | Self::Single(_) | Self::Sparse(_)
        )
    }

    /// True if value is any numeric array.
    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_float()
    }

    /// True if value is a sparse matrix.
    pub fn is_sparse(&self) -> bool {
        matches!(self, Self::Sparse(_))
    }

    /// Attempts to extract a scalar `f64`.
    pub fn as_scalar_f64(&self) -> Result<f64, MathError> {
        match self {
            Self::Double(a) if a.is_scalar() => Ok(a.as_slice()[0]),
            Self::Single(a) if a.is_scalar() => Ok(a.as_slice()[0] as f64),
            Self::Logical(a) if a.is_scalar() => Ok(if a.as_slice()[0] { 1.0 } else { 0.0 }),
            Self::Int8(a) if a.is_scalar() => Ok(a.as_slice()[0] as f64),
            Self::Int16(a) if a.is_scalar() => Ok(a.as_slice()[0] as f64),
            Self::Int32(a) if a.is_scalar() => Ok(a.as_slice()[0] as f64),
            Self::Int64(a) if a.is_scalar() => Ok(a.as_slice()[0] as f64),
            Self::UInt8(a) if a.is_scalar() => Ok(a.as_slice()[0] as f64),
            Self::UInt16(a) if a.is_scalar() => Ok(a.as_slice()[0] as f64),
            Self::UInt32(a) if a.is_scalar() => Ok(a.as_slice()[0] as f64),
            Self::UInt64(a) if a.is_scalar() => Ok(a.as_slice()[0] as f64),
            Self::Char(a) if a.is_scalar() => Ok(a.as_slice()[0] as u32 as f64),
            Self::Sparse(s) if s.shape == [1, 1] => s.get_1based(1, 1),
            _ => Err(MathError::UnsupportedOperation {
                op: "as_scalar_f64",
                type_name: "non-scalar",
            }),
        }
    }

    /// Attempts to extract an exact scalar `i64` without float conversion.
    pub fn as_scalar_i64(&self) -> Result<i64, MathError> {
        match self {
            Self::Int64(a) if a.is_scalar() => Ok(a.as_slice()[0]),
            Self::Int32(a) if a.is_scalar() => Ok(a.as_slice()[0] as i64),
            Self::Int16(a) if a.is_scalar() => Ok(a.as_slice()[0] as i64),
            Self::Int8(a) if a.is_scalar() => Ok(a.as_slice()[0] as i64),
            Self::UInt32(a) if a.is_scalar() => Ok(a.as_slice()[0] as i64),
            Self::UInt16(a) if a.is_scalar() => Ok(a.as_slice()[0] as i64),
            Self::UInt8(a) if a.is_scalar() => Ok(a.as_slice()[0] as i64),
            Self::Double(a) if a.is_scalar() => Ok(a.as_slice()[0].saturating_cast()),
            _ => Err(MathError::UnsupportedOperation {
                op: "as_scalar_i64",
                type_name: "non-scalar",
            }),
        }
    }

    /// Converts array to a double array representation.
    pub fn to_double_array(&self) -> Result<DenseArray<f64>, MathError> {
        match self {
            Self::Double(a) => Ok(a.clone()),
            Self::Single(a) => {
                let data = a.as_slice().iter().map(|&x| x as f64).collect();
                DenseArray::from_vec(a.shape().to_vec(), data)
            }
            Self::Logical(a) => {
                let data = a
                    .as_slice()
                    .iter()
                    .map(|&b| if b { 1.0 } else { 0.0 })
                    .collect();
                DenseArray::from_vec(a.shape().to_vec(), data)
            }
            Self::Int8(a) => {
                let data = a.as_slice().iter().map(|&i| i as f64).collect();
                DenseArray::from_vec(a.shape().to_vec(), data)
            }
            Self::Int16(a) => {
                let data = a.as_slice().iter().map(|&i| i as f64).collect();
                DenseArray::from_vec(a.shape().to_vec(), data)
            }
            Self::Int32(a) => {
                let data = a.as_slice().iter().map(|&i| i as f64).collect();
                DenseArray::from_vec(a.shape().to_vec(), data)
            }
            Self::Int64(a) => {
                let data = a.as_slice().iter().map(|&i| i as f64).collect();
                DenseArray::from_vec(a.shape().to_vec(), data)
            }
            Self::UInt8(a) => {
                let data = a.as_slice().iter().map(|&i| i as f64).collect();
                DenseArray::from_vec(a.shape().to_vec(), data)
            }
            Self::UInt16(a) => {
                let data = a.as_slice().iter().map(|&i| i as f64).collect();
                DenseArray::from_vec(a.shape().to_vec(), data)
            }
            Self::UInt32(a) => {
                let data = a.as_slice().iter().map(|&i| i as f64).collect();
                DenseArray::from_vec(a.shape().to_vec(), data)
            }
            Self::UInt64(a) => {
                let data = a.as_slice().iter().map(|&i| i as f64).collect();
                DenseArray::from_vec(a.shape().to_vec(), data)
            }
            Self::Char(a) => {
                let data = a.as_slice().iter().map(|&c| c as u32 as f64).collect();
                DenseArray::from_vec(a.shape().to_vec(), data)
            }
            Self::Sparse(s) => Ok(s.to_dense()),
            other => Err(MathError::UnsupportedOperation {
                op: "to_double_array",
                type_name: other.type_name(),
            }),
        }
    }

    /// Converts array to a single (32-bit float) array representation.
    pub fn to_single_array(&self) -> Result<DenseArray<f32>, MathError> {
        match self {
            Self::Single(a) => Ok(a.clone()),
            Self::Double(a) => {
                let data = a.as_slice().iter().map(|&x| x as f32).collect();
                DenseArray::from_vec(a.shape().to_vec(), data)
            }
            _ => {
                let da = self.to_double_array()?;
                let data = da.as_slice().iter().map(|&x| x as f32).collect();
                DenseArray::from_vec(da.shape().to_vec(), data)
            }
        }
    }

    /// Converts numeric arrays to a Complex64 array representation.
    pub fn to_complex_array(&self) -> Result<DenseArray<Complex64>, MathError> {
        match self {
            Self::ComplexDouble(a) => Ok(a.clone()),
            _ => {
                let da = self.to_double_array()?;
                let data = da
                    .as_slice()
                    .iter()
                    .map(|&x| Complex64::new(x, 0.0))
                    .collect();
                DenseArray::from_vec(da.shape().to_vec(), data)
            }
        }
    }

    /// Converts array to an `int8` representation with saturating math.
    pub fn to_int8(&self) -> Result<Self, MathError> {
        if let Self::Int8(_) = self {
            return Ok(self.clone());
        }
        let da = self.to_double_array()?;
        let data: Vec<i8> = da.as_slice().iter().map(|&x| x.saturating_cast()).collect();
        DenseArray::from_vec(da.shape().to_vec(), data).map(Self::Int8)
    }

    /// Converts array to an `int16` representation with saturating math.
    pub fn to_int16(&self) -> Result<Self, MathError> {
        if let Self::Int16(_) = self {
            return Ok(self.clone());
        }
        let da = self.to_double_array()?;
        let data: Vec<i16> = da.as_slice().iter().map(|&x| x.saturating_cast()).collect();
        DenseArray::from_vec(da.shape().to_vec(), data).map(Self::Int16)
    }

    /// Converts array to an `int32` representation with saturating math.
    pub fn to_int32(&self) -> Result<Self, MathError> {
        if let Self::Int32(_) = self {
            return Ok(self.clone());
        }
        let da = self.to_double_array()?;
        let data: Vec<i32> = da.as_slice().iter().map(|&x| x.saturating_cast()).collect();
        DenseArray::from_vec(da.shape().to_vec(), data).map(Self::Int32)
    }

    /// Converts array to an `int64` representation with saturating math.
    pub fn to_int64(&self) -> Result<Self, MathError> {
        if let Self::Int64(_) = self {
            return Ok(self.clone());
        }
        if let Self::UInt64(ua) = self {
            let data: Vec<i64> = ua
                .as_slice()
                .iter()
                .map(|&x| {
                    if x > i64::MAX as u64 {
                        i64::MAX
                    } else {
                        x as i64
                    }
                })
                .collect();
            return DenseArray::from_vec(ua.shape().to_vec(), data).map(Self::Int64);
        }
        let da = self.to_double_array()?;
        let data: Vec<i64> = da.as_slice().iter().map(|&x| x.saturating_cast()).collect();
        DenseArray::from_vec(da.shape().to_vec(), data).map(Self::Int64)
    }

    /// Converts array to a `uint8` representation with saturating math.
    pub fn to_uint8(&self) -> Result<Self, MathError> {
        if let Self::UInt8(_) = self {
            return Ok(self.clone());
        }
        let da = self.to_double_array()?;
        let data: Vec<u8> = da.as_slice().iter().map(|&x| x.saturating_cast()).collect();
        DenseArray::from_vec(da.shape().to_vec(), data).map(Self::UInt8)
    }

    /// Converts array to a `uint16` representation with saturating math.
    pub fn to_uint16(&self) -> Result<Self, MathError> {
        if let Self::UInt16(_) = self {
            return Ok(self.clone());
        }
        let da = self.to_double_array()?;
        let data: Vec<u16> = da.as_slice().iter().map(|&x| x.saturating_cast()).collect();
        DenseArray::from_vec(da.shape().to_vec(), data).map(Self::UInt16)
    }

    /// Converts array to a `uint32` representation with saturating math.
    pub fn to_uint32(&self) -> Result<Self, MathError> {
        if let Self::UInt32(_) = self {
            return Ok(self.clone());
        }
        let da = self.to_double_array()?;
        let data: Vec<u32> = da.as_slice().iter().map(|&x| x.saturating_cast()).collect();
        DenseArray::from_vec(da.shape().to_vec(), data).map(Self::UInt32)
    }

    /// Converts array to a `uint64` representation with saturating math.
    pub fn to_uint64(&self) -> Result<Self, MathError> {
        if let Self::UInt64(_) = self {
            return Ok(self.clone());
        }
        if let Self::Int64(ia) = self {
            let data: Vec<u64> = ia
                .as_slice()
                .iter()
                .map(|&x| if x < 0 { 0 } else { x as u64 })
                .collect();
            return DenseArray::from_vec(ia.shape().to_vec(), data).map(Self::UInt64);
        }
        let da = self.to_double_array()?;
        let data: Vec<u64> = da.as_slice().iter().map(|&x| x.saturating_cast()).collect();
        DenseArray::from_vec(da.shape().to_vec(), data).map(Self::UInt64)
    }

    /// Casts array to the target MATLAB type name.
    pub fn cast_to(&self, target_class: &str) -> Result<Self, MathError> {
        match target_class {
            "double" => self.to_double_array().map(Self::Double),
            "single" => self.to_single_array().map(Self::Single),
            "int8" => self.to_int8(),
            "int16" => self.to_int16(),
            "int32" => self.to_int32(),
            "int64" => self.to_int64(),
            "uint8" => self.to_uint8(),
            "uint16" => self.to_uint16(),
            "uint32" => self.to_uint32(),
            "uint64" => self.to_uint64(),
            "logical" => {
                let da = self.to_double_array()?;
                let data: Vec<bool> = da.as_slice().iter().map(|&x| x != 0.0).collect();
                DenseArray::from_vec(da.shape().to_vec(), data).map(Self::Logical)
            }
            _ => Err(MathError::UnsupportedOperation {
                op: "cast",
                type_name: "unknown class",
            }),
        }
    }

    /// Elementwise bitwise AND on integers.
    pub fn bitand(&self, other: &Self) -> Result<Self, MathError> {
        match (self, other) {
            (Self::Int8(a), Self::Int8(b)) => {
                elementwise_binary(a, b, |x, y| x & y).map(Self::Int8)
            }
            (Self::Int16(a), Self::Int16(b)) => {
                elementwise_binary(a, b, |x, y| x & y).map(Self::Int16)
            }
            (Self::Int32(a), Self::Int32(b)) => {
                elementwise_binary(a, b, |x, y| x & y).map(Self::Int32)
            }
            (Self::Int64(a), Self::Int64(b)) => {
                elementwise_binary(a, b, |x, y| x & y).map(Self::Int64)
            }
            (Self::UInt8(a), Self::UInt8(b)) => {
                elementwise_binary(a, b, |x, y| x & y).map(Self::UInt8)
            }
            (Self::UInt16(a), Self::UInt16(b)) => {
                elementwise_binary(a, b, |x, y| x & y).map(Self::UInt16)
            }
            (Self::UInt32(a), Self::UInt32(b)) => {
                elementwise_binary(a, b, |x, y| x & y).map(Self::UInt32)
            }
            (Self::UInt64(a), Self::UInt64(b)) => {
                elementwise_binary(a, b, |x, y| x & y).map(Self::UInt64)
            }
            (a, Self::Double(b)) if a.is_integer() && b.is_scalar() => {
                let b_cast = other.cast_to(a.type_name())?;
                a.bitand(&b_cast)
            }
            (Self::Double(a), b) if b.is_integer() && a.is_scalar() => {
                let a_cast = self.cast_to(b.type_name())?;
                a_cast.bitand(b)
            }
            _ => {
                if self.is_integer() && other.is_integer() {
                    Err(MathError::integer_class_mismatch())
                } else {
                    Err(MathError::UnsupportedOperation {
                        op: "bitand",
                        type_name: self.type_name(),
                    })
                }
            }
        }
    }

    /// Elementwise bitwise OR on integers.
    pub fn bitor(&self, other: &Self) -> Result<Self, MathError> {
        match (self, other) {
            (Self::Int8(a), Self::Int8(b)) => {
                elementwise_binary(a, b, |x, y| x | y).map(Self::Int8)
            }
            (Self::Int16(a), Self::Int16(b)) => {
                elementwise_binary(a, b, |x, y| x | y).map(Self::Int16)
            }
            (Self::Int32(a), Self::Int32(b)) => {
                elementwise_binary(a, b, |x, y| x | y).map(Self::Int32)
            }
            (Self::Int64(a), Self::Int64(b)) => {
                elementwise_binary(a, b, |x, y| x | y).map(Self::Int64)
            }
            (Self::UInt8(a), Self::UInt8(b)) => {
                elementwise_binary(a, b, |x, y| x | y).map(Self::UInt8)
            }
            (Self::UInt16(a), Self::UInt16(b)) => {
                elementwise_binary(a, b, |x, y| x | y).map(Self::UInt16)
            }
            (Self::UInt32(a), Self::UInt32(b)) => {
                elementwise_binary(a, b, |x, y| x | y).map(Self::UInt32)
            }
            (Self::UInt64(a), Self::UInt64(b)) => {
                elementwise_binary(a, b, |x, y| x | y).map(Self::UInt64)
            }
            (a, Self::Double(b)) if a.is_integer() && b.is_scalar() => {
                let b_cast = other.cast_to(a.type_name())?;
                a.bitor(&b_cast)
            }
            (Self::Double(a), b) if b.is_integer() && a.is_scalar() => {
                let a_cast = self.cast_to(b.type_name())?;
                a_cast.bitor(b)
            }
            _ => {
                if self.is_integer() && other.is_integer() {
                    Err(MathError::integer_class_mismatch())
                } else {
                    Err(MathError::UnsupportedOperation {
                        op: "bitor",
                        type_name: self.type_name(),
                    })
                }
            }
        }
    }

    /// Elementwise bitwise XOR on integers.
    pub fn bitxor(&self, other: &Self) -> Result<Self, MathError> {
        match (self, other) {
            (Self::Int8(a), Self::Int8(b)) => {
                elementwise_binary(a, b, |x, y| x ^ y).map(Self::Int8)
            }
            (Self::Int16(a), Self::Int16(b)) => {
                elementwise_binary(a, b, |x, y| x ^ y).map(Self::Int16)
            }
            (Self::Int32(a), Self::Int32(b)) => {
                elementwise_binary(a, b, |x, y| x ^ y).map(Self::Int32)
            }
            (Self::Int64(a), Self::Int64(b)) => {
                elementwise_binary(a, b, |x, y| x ^ y).map(Self::Int64)
            }
            (Self::UInt8(a), Self::UInt8(b)) => {
                elementwise_binary(a, b, |x, y| x ^ y).map(Self::UInt8)
            }
            (Self::UInt16(a), Self::UInt16(b)) => {
                elementwise_binary(a, b, |x, y| x ^ y).map(Self::UInt16)
            }
            (Self::UInt32(a), Self::UInt32(b)) => {
                elementwise_binary(a, b, |x, y| x ^ y).map(Self::UInt32)
            }
            (Self::UInt64(a), Self::UInt64(b)) => {
                elementwise_binary(a, b, |x, y| x ^ y).map(Self::UInt64)
            }
            (a, Self::Double(b)) if a.is_integer() && b.is_scalar() => {
                let b_cast = other.cast_to(a.type_name())?;
                a.bitxor(&b_cast)
            }
            (Self::Double(a), b) if b.is_integer() && a.is_scalar() => {
                let a_cast = self.cast_to(b.type_name())?;
                a_cast.bitxor(b)
            }
            _ => {
                if self.is_integer() && other.is_integer() {
                    Err(MathError::integer_class_mismatch())
                } else {
                    Err(MathError::UnsupportedOperation {
                        op: "bitxor",
                        type_name: self.type_name(),
                    })
                }
            }
        }
    }

    /// Bitwise shift of an integer array by `k` bits (positive left, negative right).
    pub fn bitshift(&self, shift_val: &Self) -> Result<Self, MathError> {
        let k = shift_val.as_scalar_i64()?;
        macro_rules! shift_signed {
            ($arr:expr, $variant:ident, $t:ident, $bits:expr) => {{
                let out = elementwise_unary($arr, |&x| {
                    if k >= $bits {
                        0
                    } else if k > 0 {
                        x << (k as u32)
                    } else if (-k) >= $bits {
                        x >> ($bits - 1)
                    } else if k < 0 {
                        x >> ((-k) as u32)
                    } else {
                        x
                    }
                });
                Ok(Self::$variant(out))
            }};
        }
        macro_rules! shift_unsigned {
            ($arr:expr, $variant:ident, $t:ident, $bits:expr) => {{
                let out = elementwise_unary($arr, |&x| {
                    if k >= $bits || (-k) >= $bits {
                        0
                    } else if k > 0 {
                        x << (k as u32)
                    } else if k < 0 {
                        x >> ((-k) as u32)
                    } else {
                        x
                    }
                });
                Ok(Self::$variant(out))
            }};
        }
        match self {
            Self::Int8(a) => shift_signed!(a, Int8, i8, 8),
            Self::Int16(a) => shift_signed!(a, Int16, i16, 16),
            Self::Int32(a) => shift_signed!(a, Int32, i32, 32),
            Self::Int64(a) => shift_signed!(a, Int64, i64, 64),
            Self::UInt8(a) => shift_unsigned!(a, UInt8, u8, 8),
            Self::UInt16(a) => shift_unsigned!(a, UInt16, u16, 16),
            Self::UInt32(a) => shift_unsigned!(a, UInt32, u32, 32),
            Self::UInt64(a) => shift_unsigned!(a, UInt64, u64, 64),
            _ => Err(MathError::UnsupportedOperation {
                op: "bitshift",
                type_name: self.type_name(),
            }),
        }
    }

    /// Integer division with specified rounding mode: `'fix'`, `'floor'`, `'ceil'`, or `'round'`.
    pub fn idivide(&self, other: &Self, mode: &str) -> Result<Self, MathError> {
        macro_rules! idiv_signed_op {
            ($a:expr, $b:expr, $t:ident) => {{
                let a = $a;
                let b = $b;
                if b == 0 {
                    if a > 0 {
                        $t::MAX
                    } else if a < 0 {
                        $t::MIN
                    } else {
                        0
                    }
                } else if a == $t::MIN && b == -1 {
                    $t::MAX
                } else {
                    let q = a / b;
                    let r = a % b;
                    match mode {
                        "fix" => q,
                        "floor" => {
                            if r != 0 && ((a > 0) != (b > 0)) {
                                q.saturating_sub(1)
                            } else {
                                q
                            }
                        }
                        "ceil" => {
                            if r != 0 && ((a > 0) == (b > 0)) {
                                q.saturating_add(1)
                            } else {
                                q
                            }
                        }
                        "round" => {
                            let b_abs = if b == $t::MIN {
                                $t::MAX as u64 + 1
                            } else {
                                b.unsigned_abs() as u64
                            };
                            let r_abs = if r == $t::MIN {
                                $t::MAX as u64 + 1
                            } else {
                                r.unsigned_abs() as u64
                            };
                            if b_abs - r_abs <= r_abs {
                                if (a > 0 && b > 0) || (a < 0 && b < 0) {
                                    q.saturating_add(1)
                                } else {
                                    q.saturating_sub(1)
                                }
                            } else {
                                q
                            }
                        }
                        _ => q,
                    }
                }
            }};
        }

        macro_rules! idiv_unsigned_op {
            ($a:expr, $b:expr, $t:ident) => {{
                let a = $a;
                let b = $b;
                if b == 0 {
                    if a > 0 {
                        $t::MAX
                    } else {
                        0
                    }
                } else {
                    let q = a / b;
                    let r = a % b;
                    match mode {
                        "fix" | "floor" => q,
                        "ceil" => {
                            if r != 0 {
                                q.saturating_add(1)
                            } else {
                                q
                            }
                        }
                        "round" => {
                            if b - r <= r {
                                q.saturating_add(1)
                            } else {
                                q
                            }
                        }
                        _ => q,
                    }
                }
            }};
        }

        match (self, other) {
            (Self::Int8(a), Self::Int8(b)) => {
                elementwise_binary(a, b, |&x, &y| idiv_signed_op!(x, y, i8)).map(Self::Int8)
            }
            (Self::Int16(a), Self::Int16(b)) => {
                elementwise_binary(a, b, |&x, &y| idiv_signed_op!(x, y, i16)).map(Self::Int16)
            }
            (Self::Int32(a), Self::Int32(b)) => {
                elementwise_binary(a, b, |&x, &y| idiv_signed_op!(x, y, i32)).map(Self::Int32)
            }
            (Self::Int64(a), Self::Int64(b)) => {
                elementwise_binary(a, b, |&x, &y| idiv_signed_op!(x, y, i64)).map(Self::Int64)
            }
            (Self::UInt8(a), Self::UInt8(b)) => {
                elementwise_binary(a, b, |&x, &y| idiv_unsigned_op!(x, y, u8)).map(Self::UInt8)
            }
            (Self::UInt16(a), Self::UInt16(b)) => {
                elementwise_binary(a, b, |&x, &y| idiv_unsigned_op!(x, y, u16)).map(Self::UInt16)
            }
            (Self::UInt32(a), Self::UInt32(b)) => {
                elementwise_binary(a, b, |&x, &y| idiv_unsigned_op!(x, y, u32)).map(Self::UInt32)
            }
            (Self::UInt64(a), Self::UInt64(b)) => {
                elementwise_binary(a, b, |&x, &y| idiv_unsigned_op!(x, y, u64)).map(Self::UInt64)
            }
            (a, Self::Double(b)) if a.is_integer() && b.is_scalar() => {
                let b_cast = other.cast_to(a.type_name())?;
                a.idivide(&b_cast, mode)
            }
            (Self::Double(a), b) if b.is_integer() && a.is_scalar() => {
                let a_cast = self.cast_to(b.type_name())?;
                a_cast.idivide(b, mode)
            }
            _ => {
                if self.is_integer() && other.is_integer() {
                    Err(MathError::integer_class_mismatch())
                } else {
                    Err(MathError::UnsupportedOperation {
                        op: "idivide",
                        type_name: self.type_name(),
                    })
                }
            }
        }
    }

    /// Returns the MATLAB type name string (`'double'`, `'logical'`, etc.).
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Double(_) => "double",
            Self::ComplexDouble(_) => "complex double",
            Self::Single(_) => "single",
            Self::Logical(_) => "logical",
            Self::Int8(_) => "int8",
            Self::Int16(_) => "int16",
            Self::Int32(_) => "int32",
            Self::Int64(_) => "int64",
            Self::UInt8(_) => "uint8",
            Self::UInt16(_) => "uint16",
            Self::UInt32(_) => "uint32",
            Self::UInt64(_) => "uint64",
            Self::Char(_) => "char",
            Self::String(_) => "string",
            Self::Cell(_) => "cell",
            Self::Struct(_) => "struct",
            Self::FunctionHandle(_) => "function_handle",
            Self::Sparse(_) => "double",
        }
    }

    // --- Core Elementwise Arithmetic with Mixed-Type Dispatch ---

    #[allow(clippy::chunks_exact_to_as_chunks)]
    pub fn add(&self, other: &Self) -> Result<Self, MathError> {
        // Fast-path: Double + Double (zero cloning, vectorizable)
        if let (Self::Double(da), Self::Double(db)) = (self, other) {
            if da.shape() == db.shape() {
                let a_s = da.as_slice();
                let b_s = db.as_slice();
                let mut out_data = Vec::with_capacity(a_s.len());
                let chunks_a = a_s.chunks_exact(8);
                let chunks_b = b_s.chunks_exact(8);
                let rem_a = chunks_a.remainder();
                let rem_b = chunks_b.remainder();
                for (ca, cb) in chunks_a.zip(chunks_b) {
                    out_data.extend_from_slice(&[
                        ca[0] + cb[0],
                        ca[1] + cb[1],
                        ca[2] + cb[2],
                        ca[3] + cb[3],
                        ca[4] + cb[4],
                        ca[5] + cb[5],
                        ca[6] + cb[6],
                        ca[7] + cb[7],
                    ]);
                }
                for (&ra, &rb) in rem_a.iter().zip(rem_b.iter()) {
                    out_data.push(ra + rb);
                }
                return Ok(Self::Double(DenseArray::from_vec_unchecked(
                    da.shape().to_vec(),
                    out_data,
                )));
            }
            let out = elementwise_binary(da, db, |x, y| x + y)?;
            return Ok(Self::Double(out));
        }

        // String concatenation
        if let (Self::String(sa), Self::String(sb)) = (self, other) {
            let out = elementwise_binary(sa, sb, |a, b| format!("{}{}", a, b))?;
            return Ok(Self::String(out));
        }

        // Sparse addition
        if let (Self::Sparse(sa), Self::Sparse(sb)) = (self, other) {
            return sa.add_sparse(sb).map(Self::Sparse);
        }
        if let (Self::Sparse(sa), db) = (self, other) {
            let d = db.to_double_array()?;
            let dense_a = sa.to_dense();
            let out = elementwise_binary(&dense_a, &d, |x, y| x + y)?;
            return Ok(Self::Double(out));
        }
        if let (da, Self::Sparse(sb)) = (self, other) {
            let d = da.to_double_array()?;
            let dense_b = sb.to_dense();
            let out = elementwise_binary(&d, &dense_b, |x, y| x + y)?;
            return Ok(Self::Double(out));
        }

        // Integer arithmetic (saturating, mixed-type error or int+double -> int)
        if self.is_integer() || other.is_integer() {
            return self.add_integer(other);
        }

        // Fast-path: Single + Single (zero cloning)
        if let (Self::Single(sa), Self::Single(sb)) = (self, other) {
            let out = elementwise_binary(sa, sb, |x, y| x + y)?;
            return Ok(Self::Single(out));
        }

        // Fast-path: Complex + Complex (zero cloning)
        if let (Self::ComplexDouble(ca), Self::ComplexDouble(cb)) = (self, other) {
            let out = elementwise_binary(ca, cb, |x, y| x + y)?;
            return Ok(Self::ComplexDouble(out));
        }

        // Single precision arithmetic with mixed promotion
        if matches!(self, Self::Single(_)) || matches!(other, Self::Single(_)) {
            let sa = self.to_single_array()?;
            let sb = other.to_single_array()?;
            let out = elementwise_binary(&sa, &sb, |x, y| x + y)?;
            return Ok(Self::Single(out));
        }

        // Complex arithmetic with mixed promotion
        if matches!(self, Self::ComplexDouble(_)) || matches!(other, Self::ComplexDouble(_)) {
            let ca = self.to_complex_array()?;
            let cb = other.to_complex_array()?;
            let out = elementwise_binary(&ca, &cb, |x, y| x + y)?;
            return Ok(Self::ComplexDouble(out));
        }

        // Default double arithmetic
        let da = self.to_double_array()?;
        let db = other.to_double_array()?;
        let out = elementwise_binary(&da, &db, |x, y| x + y)?;
        Ok(Self::Double(out))
    }

    pub fn sub(&self, other: &Self) -> Result<Self, MathError> {
        // Sparse subtraction
        if let (Self::Sparse(sa), Self::Sparse(sb)) = (self, other) {
            let neg_b = CscMatrix {
                shape: sb.shape,
                col_ptr: sb.col_ptr.clone(),
                row_ind: sb.row_ind.clone(),
                values: sb.values.iter().map(|&x| -x).collect(),
            };
            return sa.add_sparse(&neg_b).map(Self::Sparse);
        }
        if let (Self::Sparse(sa), db) = (self, other) {
            let d = db.to_double_array()?;
            let dense_a = sa.to_dense();
            let out = elementwise_binary(&dense_a, &d, |x, y| x - y)?;
            return Ok(Self::Double(out));
        }
        if let (da, Self::Sparse(sb)) = (self, other) {
            let d = da.to_double_array()?;
            let dense_b = sb.to_dense();
            let out = elementwise_binary(&d, &dense_b, |x, y| x - y)?;
            return Ok(Self::Double(out));
        }

        if self.is_integer() || other.is_integer() {
            return self.sub_integer(other);
        }

        // Fast-path: Double - Double (zero cloning)
        if let (Self::Double(da), Self::Double(db)) = (self, other) {
            let out = elementwise_binary(da, db, |x, y| x - y)?;
            return Ok(Self::Double(out));
        }

        // Fast-path: Single - Single (zero cloning)
        if let (Self::Single(sa), Self::Single(sb)) = (self, other) {
            let out = elementwise_binary(sa, sb, |x, y| x - y)?;
            return Ok(Self::Single(out));
        }

        // Fast-path: Complex - Complex (zero cloning)
        if let (Self::ComplexDouble(ca), Self::ComplexDouble(cb)) = (self, other) {
            let out = elementwise_binary(ca, cb, |x, y| x - y)?;
            return Ok(Self::ComplexDouble(out));
        }

        if matches!(self, Self::Single(_)) || matches!(other, Self::Single(_)) {
            let sa = self.to_single_array()?;
            let sb = other.to_single_array()?;
            let out = elementwise_binary(&sa, &sb, |x, y| x - y)?;
            return Ok(Self::Single(out));
        }

        if matches!(self, Self::ComplexDouble(_)) || matches!(other, Self::ComplexDouble(_)) {
            let ca = self.to_complex_array()?;
            let cb = other.to_complex_array()?;
            let out = elementwise_binary(&ca, &cb, |x, y| x - y)?;
            return Ok(Self::ComplexDouble(out));
        }

        let da = self.to_double_array()?;
        let db = other.to_double_array()?;
        let out = elementwise_binary(&da, &db, |x, y| x - y)?;
        Ok(Self::Double(out))
    }

    #[allow(clippy::chunks_exact_to_as_chunks)]
    pub fn dot_mul(&self, other: &Self) -> Result<Self, MathError> {
        if self.is_integer() || other.is_integer() {
            return self.mul_integer(other);
        }

        // Fast-path: Double .* Double (zero cloning, vectorizable)
        if let (Self::Double(da), Self::Double(db)) = (self, other) {
            if da.shape() == db.shape() {
                let a_s = da.as_slice();
                let b_s = db.as_slice();
                let mut out_data = Vec::with_capacity(a_s.len());
                let chunks_a = a_s.chunks_exact(8);
                let chunks_b = b_s.chunks_exact(8);
                let rem_a = chunks_a.remainder();
                let rem_b = chunks_b.remainder();
                for (ca, cb) in chunks_a.zip(chunks_b) {
                    out_data.extend_from_slice(&[
                        ca[0] * cb[0],
                        ca[1] * cb[1],
                        ca[2] * cb[2],
                        ca[3] * cb[3],
                        ca[4] * cb[4],
                        ca[5] * cb[5],
                        ca[6] * cb[6],
                        ca[7] * cb[7],
                    ]);
                }
                for (&ra, &rb) in rem_a.iter().zip(rem_b.iter()) {
                    out_data.push(ra * rb);
                }
                return Ok(Self::Double(DenseArray::from_vec_unchecked(
                    da.shape().to_vec(),
                    out_data,
                )));
            }
            let out = elementwise_binary(da, db, |x, y| x * y)?;
            return Ok(Self::Double(out));
        }

        // Fast-path: Single .* Single (zero cloning)
        if let (Self::Single(sa), Self::Single(sb)) = (self, other) {
            let out = elementwise_binary(sa, sb, |x, y| x * y)?;
            return Ok(Self::Single(out));
        }

        // Fast-path: Complex .* Complex (zero cloning)
        if let (Self::ComplexDouble(ca), Self::ComplexDouble(cb)) = (self, other) {
            let out = elementwise_binary(ca, cb, |x, y| x * y)?;
            return Ok(Self::ComplexDouble(out));
        }

        if matches!(self, Self::Single(_)) || matches!(other, Self::Single(_)) {
            let sa = self.to_single_array()?;
            let sb = other.to_single_array()?;
            let out = elementwise_binary(&sa, &sb, |x, y| x * y)?;
            return Ok(Self::Single(out));
        }

        if matches!(self, Self::ComplexDouble(_)) || matches!(other, Self::ComplexDouble(_)) {
            let ca = self.to_complex_array()?;
            let cb = other.to_complex_array()?;
            let out = elementwise_binary(&ca, &cb, |x, y| x * y)?;
            return Ok(Self::ComplexDouble(out));
        }

        let da = self.to_double_array()?;
        let db = other.to_double_array()?;
        let out = elementwise_binary(&da, &db, |x, y| x * y)?;
        Ok(Self::Double(out))
    }

    pub fn dot_div(&self, other: &Self) -> Result<Self, MathError> {
        if self.is_integer() || other.is_integer() {
            return self.div_integer(other);
        }

        // Fast-path: Double ./ Double (zero cloning)
        if let (Self::Double(da), Self::Double(db)) = (self, other) {
            let out = elementwise_binary(da, db, |x, y| x / y)?;
            return Ok(Self::Double(out));
        }

        // Fast-path: Single ./ Single (zero cloning)
        if let (Self::Single(sa), Self::Single(sb)) = (self, other) {
            let out = elementwise_binary(sa, sb, |x, y| x / y)?;
            return Ok(Self::Single(out));
        }

        // Fast-path: Complex ./ Complex (zero cloning)
        if let (Self::ComplexDouble(ca), Self::ComplexDouble(cb)) = (self, other) {
            let out = elementwise_binary(ca, cb, |x, y| x / y)?;
            return Ok(Self::ComplexDouble(out));
        }

        if matches!(self, Self::Single(_)) || matches!(other, Self::Single(_)) {
            let sa = self.to_single_array()?;
            let sb = other.to_single_array()?;
            let out = elementwise_binary(&sa, &sb, |x, y| x / y)?;
            return Ok(Self::Single(out));
        }

        if matches!(self, Self::ComplexDouble(_)) || matches!(other, Self::ComplexDouble(_)) {
            let ca = self.to_complex_array()?;
            let cb = other.to_complex_array()?;
            let out = elementwise_binary(&ca, &cb, |x, y| x / y)?;
            return Ok(Self::ComplexDouble(out));
        }

        let da = self.to_double_array()?;
        let db = other.to_double_array()?;
        let out = elementwise_binary(&da, &db, |x, y| x / y)?;
        Ok(Self::Double(out))
    }

    pub fn dot_left_div(&self, other: &Self) -> Result<Self, MathError> {
        other.dot_div(self)
    }

    pub fn dot_pow(&self, other: &Self) -> Result<Self, MathError> {
        let da = self.to_double_array()?;
        let db = other.to_double_array()?;
        let out = elementwise_binary(&da, &db, |x, y| x.powf(*y))?;
        Ok(Self::Double(out))
    }

    /// Matrix multiplication: $A * B$.
    pub fn mul(&self, other: &Self) -> Result<Self, MathError> {
        // Sparse multiplication
        if let (Self::Sparse(sa), Self::Sparse(sb)) = (self, other) {
            return sa.matmul_sparse(sb).map(Self::Sparse);
        }
        if let (Self::Sparse(sa), db) = (self, other) {
            let d = db.to_double_array()?;
            let out = sa.matmul_dense(&d)?;
            return Ok(Self::Double(out));
        }
        if let (da, Self::Sparse(sb)) = (self, other) {
            let d = da.to_double_array()?;
            let trans_b = sb.transpose();
            let trans_d = d.transpose_2d()?;
            let trans_c = trans_b.matmul_dense(&trans_d)?;
            let c = trans_c.transpose_2d()?;
            return Ok(Self::Double(c));
        }

        if self.is_integer() || other.is_integer() {
            // In MATLAB, matrix multiplication between integer and scalar performs elementwise scalar mul
            if self.is_scalar() || other.is_scalar() {
                return self.dot_mul(other);
            }
            return Err(MathError::UnsupportedOperation {
                op: "matrix multiplication",
                type_name: self.type_name(),
            });
        }

        if matches!(self, Self::ComplexDouble(_)) || matches!(other, Self::ComplexDouble(_)) {
            let ca = self.to_complex_array()?;
            let cb = other.to_complex_array()?;
            if ca.is_scalar() {
                let s = ca.as_slice()[0];
                let out = elementwise_unary(&cb, |x| x * s);
                return Ok(Self::ComplexDouble(out));
            }
            if cb.is_scalar() {
                let s = cb.as_slice()[0];
                let out = elementwise_unary(&ca, |x| x * s);
                return Ok(Self::ComplexDouble(out));
            }
            return Err(MathError::UnsupportedOperation {
                op: "complex matrix multiplication",
                type_name: "complex double",
            });
        }

        let da = self.to_double_array()?;
        let db = other.to_double_array()?;
        if da.is_scalar() {
            let s = da.as_slice()[0];
            let out = elementwise_unary(&db, |x| x * s);
            return Ok(Self::Double(out));
        }
        if db.is_scalar() {
            let s = db.as_slice()[0];
            let out = elementwise_unary(&da, |x| x * s);
            return Ok(Self::Double(out));
        }

        let out = matmul_2d(&da, &db)?;
        Ok(Self::Double(out))
    }

    /// Unary negation: `-A`.
    pub fn neg(&self) -> Result<Self, MathError> {
        match self {
            Self::Double(a) => Ok(Self::Double(elementwise_unary(a, |x| -x))),
            Self::Single(a) => Ok(Self::Single(elementwise_unary(a, |x| -x))),
            Self::ComplexDouble(a) => Ok(Self::ComplexDouble(elementwise_unary(a, |x| -x))),
            Self::Int8(a) => Ok(Self::Int8(elementwise_unary(a, |x| x.saturating_neg()))),
            Self::Int16(a) => Ok(Self::Int16(elementwise_unary(a, |x| x.saturating_neg()))),
            Self::Int32(a) => Ok(Self::Int32(elementwise_unary(a, |x| x.saturating_neg()))),
            Self::Int64(a) => Ok(Self::Int64(elementwise_unary(a, |x| x.saturating_neg()))),
            Self::UInt8(a) => Ok(Self::UInt8(elementwise_unary(a, |x| {
                0u8.saturating_sub(*x)
            }))),
            Self::UInt16(a) => Ok(Self::UInt16(elementwise_unary(a, |x| {
                0u16.saturating_sub(*x)
            }))),
            Self::UInt32(a) => Ok(Self::UInt32(elementwise_unary(a, |x| {
                0u32.saturating_sub(*x)
            }))),
            Self::UInt64(a) => Ok(Self::UInt64(elementwise_unary(a, |x| {
                0u64.saturating_sub(*x)
            }))),
            Self::Sparse(s) => {
                let neg_vals = s.values.iter().map(|&x| -x).collect();
                Ok(Self::Sparse(CscMatrix {
                    shape: s.shape,
                    col_ptr: s.col_ptr.clone(),
                    row_ind: s.row_ind.clone(),
                    values: neg_vals,
                }))
            }
            _ => {
                let da = self.to_double_array()?;
                Ok(Self::Double(elementwise_unary(&da, |x| -x)))
            }
        }
    }

    /// Absolute value: `abs(A)`.
    pub fn abs(&self) -> Result<Self, MathError> {
        match self {
            Self::Double(a) => Ok(Self::Double(elementwise_unary(a, |x| x.abs()))),
            Self::Single(a) => Ok(Self::Single(elementwise_unary(a, |x| x.abs()))),
            Self::ComplexDouble(a) => Ok(Self::Double(elementwise_unary(a, |x| x.norm()))),
            Self::Int8(a) => Ok(Self::Int8(elementwise_unary(a, |x| x.saturating_abs()))),
            Self::Int16(a) => Ok(Self::Int16(elementwise_unary(a, |x| x.saturating_abs()))),
            Self::Int32(a) => Ok(Self::Int32(elementwise_unary(a, |x| x.saturating_abs()))),
            Self::Int64(a) => Ok(Self::Int64(elementwise_unary(a, |x| x.saturating_abs()))),
            Self::UInt8(a) => Ok(Self::UInt8(a.clone())),
            Self::UInt16(a) => Ok(Self::UInt16(a.clone())),
            Self::UInt32(a) => Ok(Self::UInt32(a.clone())),
            Self::UInt64(a) => Ok(Self::UInt64(a.clone())),
            _ => {
                let da = self.to_double_array()?;
                Ok(Self::Double(elementwise_unary(&da, |x| x.abs())))
            }
        }
    }

    /// Non-conjugate array transpose: `A.'`.
    pub fn transpose(&self) -> Result<Self, MathError> {
        match self {
            Self::Double(a) => Ok(Self::Double(a.transpose_2d()?)),
            Self::Single(a) => Ok(Self::Single(a.transpose_2d()?)),
            Self::ComplexDouble(a) => Ok(Self::ComplexDouble(a.transpose_2d()?)),
            Self::Logical(a) => Ok(Self::Logical(a.transpose_2d()?)),
            Self::Int8(a) => Ok(Self::Int8(a.transpose_2d()?)),
            Self::Int16(a) => Ok(Self::Int16(a.transpose_2d()?)),
            Self::Int32(a) => Ok(Self::Int32(a.transpose_2d()?)),
            Self::Int64(a) => Ok(Self::Int64(a.transpose_2d()?)),
            Self::UInt8(a) => Ok(Self::UInt8(a.transpose_2d()?)),
            Self::UInt16(a) => Ok(Self::UInt16(a.transpose_2d()?)),
            Self::UInt32(a) => Ok(Self::UInt32(a.transpose_2d()?)),
            Self::UInt64(a) => Ok(Self::UInt64(a.transpose_2d()?)),
            Self::Char(a) => Ok(Self::Char(a.transpose_2d()?)),
            Self::Cell(a) => Ok(Self::Cell(a.transpose_2d()?)),
            Self::Sparse(s) => Ok(Self::Sparse(s.transpose())),
            other => Err(MathError::UnsupportedOperation {
                op: "transpose",
                type_name: other.type_name(),
            }),
        }
    }

    /// Complex conjugate transpose: `A'`.
    pub fn ctranspose(&self) -> Result<Self, MathError> {
        match self {
            Self::ComplexDouble(a) => {
                let trans = a.transpose_2d()?;
                let conj = elementwise_unary(&trans, |x| x.conj());
                Ok(Self::ComplexDouble(conj))
            }
            _ => self.transpose(),
        }
    }

    // --- Helper Dispatchers for Integer Saturating Arithmetic ---

    fn add_integer(&self, other: &Self) -> Result<Self, MathError> {
        macro_rules! match_int_add {
            ($self_arr:expr, $variant:ident, $t:ty, $cast_method:ident) => {
                match other {
                    Self::$variant(b) => {
                        let out = elementwise_binary($self_arr, b, |x, y| x.saturating_add(*y))?;
                        Ok(Self::$variant(out))
                    }
                    Self::Double(b) => {
                        if !b.is_scalar() {
                            return Err(MathError::integer_class_mismatch());
                        }
                        let b_cast = b.map(|&x| x.saturating_cast());
                        let out =
                            elementwise_binary($self_arr, &b_cast, |x, y| x.saturating_add(*y))?;
                        Ok(Self::$variant(out))
                    }
                    Self::Logical(b) => {
                        let b_cast = b.map(|&x| if x { 1 as $t } else { 0 as $t });
                        let out =
                            elementwise_binary($self_arr, &b_cast, |x, y| x.saturating_add(*y))?;
                        Ok(Self::$variant(out))
                    }
                    Self::Char(b) => {
                        let b_cast = b.map(|&c| c as u32 as $t);
                        let out =
                            elementwise_binary($self_arr, &b_cast, |x, y| x.saturating_add(*y))?;
                        Ok(Self::$variant(out))
                    }
                    _ => Err(MathError::integer_class_mismatch()),
                }
            };
        }

        match self {
            Self::Int8(a) => match_int_add!(a, Int8, i8, saturating_cast),
            Self::Int16(a) => match_int_add!(a, Int16, i16, saturating_cast),
            Self::Int32(a) => match_int_add!(a, Int32, i32, saturating_cast),
            Self::Int64(a) => match_int_add!(a, Int64, i64, saturating_cast),
            Self::UInt8(a) => match_int_add!(a, UInt8, u8, saturating_cast),
            Self::UInt16(a) => match_int_add!(a, UInt16, u16, saturating_cast),
            Self::UInt32(a) => match_int_add!(a, UInt32, u32, saturating_cast),
            Self::UInt64(a) => match_int_add!(a, UInt64, u64, saturating_cast),
            _ => {
                // other is the integer
                other.add_integer(self)
            }
        }
    }

    fn sub_integer(&self, other: &Self) -> Result<Self, MathError> {
        macro_rules! match_int_sub {
            ($self_arr:expr, $variant:ident, $t:ty) => {
                match other {
                    Self::$variant(b) => {
                        let out = elementwise_binary($self_arr, b, |x, y| x.saturating_sub(*y))?;
                        Ok(Self::$variant(out))
                    }
                    Self::Double(b) => {
                        if !b.is_scalar() {
                            return Err(MathError::integer_class_mismatch());
                        }
                        let b_cast = b.map(|&x| x.saturating_cast());
                        let out =
                            elementwise_binary($self_arr, &b_cast, |x, y| x.saturating_sub(*y))?;
                        Ok(Self::$variant(out))
                    }
                    Self::Logical(b) => {
                        let b_cast = b.map(|&x| if x { 1 as $t } else { 0 as $t });
                        let out =
                            elementwise_binary($self_arr, &b_cast, |x, y| x.saturating_sub(*y))?;
                        Ok(Self::$variant(out))
                    }
                    Self::Char(b) => {
                        let b_cast = b.map(|&c| c as u32 as $t);
                        let out =
                            elementwise_binary($self_arr, &b_cast, |x, y| x.saturating_sub(*y))?;
                        Ok(Self::$variant(out))
                    }
                    _ => Err(MathError::integer_class_mismatch()),
                }
            };
        }

        match self {
            Self::Int8(a) => match_int_sub!(a, Int8, i8),
            Self::Int16(a) => match_int_sub!(a, Int16, i16),
            Self::Int32(a) => match_int_sub!(a, Int32, i32),
            Self::Int64(a) => match_int_sub!(a, Int64, i64),
            Self::UInt8(a) => match_int_sub!(a, UInt8, u8),
            Self::UInt16(a) => match_int_sub!(a, UInt16, u16),
            Self::UInt32(a) => match_int_sub!(a, UInt32, u32),
            Self::UInt64(a) => match_int_sub!(a, UInt64, u64),
            _ => {
                // self is double or logical/char, other is integer
                if let Self::Double(da) = self {
                    if !da.is_scalar() {
                        return Err(MathError::integer_class_mismatch());
                    }
                }
                let self_da = self.to_double_array()?;
                match other {
                    Self::Int8(b) => {
                        let a_cast: DenseArray<i8> = self_da.map(|&x| x.saturating_cast());
                        elementwise_binary(&a_cast, b, |x, y| x.saturating_sub(*y)).map(Self::Int8)
                    }
                    Self::Int16(b) => {
                        let a_cast: DenseArray<i16> = self_da.map(|&x| x.saturating_cast());
                        elementwise_binary(&a_cast, b, |x, y| x.saturating_sub(*y)).map(Self::Int16)
                    }
                    Self::Int32(b) => {
                        let a_cast: DenseArray<i32> = self_da.map(|&x| x.saturating_cast());
                        elementwise_binary(&a_cast, b, |x, y| x.saturating_sub(*y)).map(Self::Int32)
                    }
                    Self::Int64(b) => {
                        let a_cast: DenseArray<i64> = self_da.map(|&x| x.saturating_cast());
                        elementwise_binary(&a_cast, b, |x, y| x.saturating_sub(*y)).map(Self::Int64)
                    }
                    Self::UInt8(b) => {
                        let a_cast: DenseArray<u8> = self_da.map(|&x| x.saturating_cast());
                        elementwise_binary(&a_cast, b, |x, y| x.saturating_sub(*y)).map(Self::UInt8)
                    }
                    Self::UInt16(b) => {
                        let a_cast: DenseArray<u16> = self_da.map(|&x| x.saturating_cast());
                        elementwise_binary(&a_cast, b, |x, y| x.saturating_sub(*y))
                            .map(Self::UInt16)
                    }
                    Self::UInt32(b) => {
                        let a_cast: DenseArray<u32> = self_da.map(|&x| x.saturating_cast());
                        elementwise_binary(&a_cast, b, |x, y| x.saturating_sub(*y))
                            .map(Self::UInt32)
                    }
                    Self::UInt64(b) => {
                        let a_cast: DenseArray<u64> = self_da.map(|&x| x.saturating_cast());
                        elementwise_binary(&a_cast, b, |x, y| x.saturating_sub(*y))
                            .map(Self::UInt64)
                    }
                    _ => Err(MathError::integer_class_mismatch()),
                }
            }
        }
    }

    fn mul_integer(&self, other: &Self) -> Result<Self, MathError> {
        macro_rules! match_int_mul {
            ($self_arr:expr, $variant:ident, $t:ty) => {
                match other {
                    Self::$variant(b) => {
                        let out = elementwise_binary($self_arr, b, |x, y| x.saturating_mul(*y))?;
                        Ok(Self::$variant(out))
                    }
                    Self::Double(b) => {
                        if !b.is_scalar() {
                            return Err(MathError::integer_class_mismatch());
                        }
                        let b_cast = b.map(|&x| x.saturating_cast());
                        let out =
                            elementwise_binary($self_arr, &b_cast, |x, y| x.saturating_mul(*y))?;
                        Ok(Self::$variant(out))
                    }
                    Self::Logical(b) => {
                        let b_cast = b.map(|&x| if x { 1 as $t } else { 0 as $t });
                        let out =
                            elementwise_binary($self_arr, &b_cast, |x, y| x.saturating_mul(*y))?;
                        Ok(Self::$variant(out))
                    }
                    Self::Char(b) => {
                        let b_cast = b.map(|&c| c as u32 as $t);
                        let out =
                            elementwise_binary($self_arr, &b_cast, |x, y| x.saturating_mul(*y))?;
                        Ok(Self::$variant(out))
                    }
                    _ => Err(MathError::integer_class_mismatch()),
                }
            };
        }

        match self {
            Self::Int8(a) => match_int_mul!(a, Int8, i8),
            Self::Int16(a) => match_int_mul!(a, Int16, i16),
            Self::Int32(a) => match_int_mul!(a, Int32, i32),
            Self::Int64(a) => match_int_mul!(a, Int64, i64),
            Self::UInt8(a) => match_int_mul!(a, UInt8, u8),
            Self::UInt16(a) => match_int_mul!(a, UInt16, u16),
            Self::UInt32(a) => match_int_mul!(a, UInt32, u32),
            Self::UInt64(a) => match_int_mul!(a, UInt64, u64),
            _ => {
                if let Self::Double(da) = self {
                    if !da.is_scalar() {
                        return Err(MathError::integer_class_mismatch());
                    }
                }
                other.mul_integer(self)
            }
        }
    }

    fn div_integer(&self, other: &Self) -> Result<Self, MathError> {
        macro_rules! match_int_div {
            ($self_arr:expr, $variant:ident, $div_fn:ident, $t:ty) => {
                match other {
                    Self::$variant(b) => {
                        let out = elementwise_binary($self_arr, b, |x, y| $div_fn(*x, *y))?;
                        Ok(Self::$variant(out))
                    }
                    Self::Double(b) => {
                        if !b.is_scalar() {
                            return Err(MathError::integer_class_mismatch());
                        }
                        let b_cast = b.map(|&x| x.saturating_cast());
                        let out = elementwise_binary($self_arr, &b_cast, |x, y| $div_fn(*x, *y))?;
                        Ok(Self::$variant(out))
                    }
                    Self::Logical(b) => {
                        let b_cast = b.map(|&x| if x { 1 as $t } else { 0 as $t });
                        let out = elementwise_binary($self_arr, &b_cast, |x, y| $div_fn(*x, *y))?;
                        Ok(Self::$variant(out))
                    }
                    Self::Char(b) => {
                        let b_cast = b.map(|&c| c as u32 as $t);
                        let out = elementwise_binary($self_arr, &b_cast, |x, y| $div_fn(*x, *y))?;
                        Ok(Self::$variant(out))
                    }
                    _ => Err(MathError::integer_class_mismatch()),
                }
            };
        }

        match self {
            Self::Int8(a) => match_int_div!(a, Int8, idiv_round_i8, i8),
            Self::Int16(a) => match_int_div!(a, Int16, idiv_round_i16, i16),
            Self::Int32(a) => match_int_div!(a, Int32, idiv_round_i32, i32),
            Self::Int64(a) => match_int_div!(a, Int64, idiv_round_i64, i64),
            Self::UInt8(a) => match_int_div!(a, UInt8, idiv_round_u8, u8),
            Self::UInt16(a) => match_int_div!(a, UInt16, idiv_round_u16, u16),
            Self::UInt32(a) => match_int_div!(a, UInt32, idiv_round_u32, u32),
            Self::UInt64(a) => match_int_div!(a, UInt64, idiv_round_u64, u64),
            _ => {
                if let Self::Double(da) = self {
                    if !da.is_scalar() {
                        return Err(MathError::integer_class_mismatch());
                    }
                }
                let self_da = self.to_double_array()?;
                match other {
                    Self::Int8(b) => {
                        let a_cast: DenseArray<i8> = self_da.map(|&x| x.saturating_cast());
                        elementwise_binary(&a_cast, b, |x, y| idiv_round_i8(*x, *y)).map(Self::Int8)
                    }
                    Self::Int16(b) => {
                        let a_cast: DenseArray<i16> = self_da.map(|&x| x.saturating_cast());
                        elementwise_binary(&a_cast, b, |x, y| idiv_round_i16(*x, *y))
                            .map(Self::Int16)
                    }
                    Self::Int32(b) => {
                        let a_cast: DenseArray<i32> = self_da.map(|&x| x.saturating_cast());
                        elementwise_binary(&a_cast, b, |x, y| idiv_round_i32(*x, *y))
                            .map(Self::Int32)
                    }
                    Self::Int64(b) => {
                        let a_cast: DenseArray<i64> = self_da.map(|&x| x.saturating_cast());
                        elementwise_binary(&a_cast, b, |x, y| idiv_round_i64(*x, *y))
                            .map(Self::Int64)
                    }
                    Self::UInt8(b) => {
                        let a_cast: DenseArray<u8> = self_da.map(|&x| x.saturating_cast());
                        elementwise_binary(&a_cast, b, |x, y| idiv_round_u8(*x, *y))
                            .map(Self::UInt8)
                    }
                    Self::UInt16(b) => {
                        let a_cast: DenseArray<u16> = self_da.map(|&x| x.saturating_cast());
                        elementwise_binary(&a_cast, b, |x, y| idiv_round_u16(*x, *y))
                            .map(Self::UInt16)
                    }
                    Self::UInt32(b) => {
                        let a_cast: DenseArray<u32> = self_da.map(|&x| x.saturating_cast());
                        elementwise_binary(&a_cast, b, |x, y| idiv_round_u32(*x, *y))
                            .map(Self::UInt32)
                    }
                    Self::UInt64(b) => {
                        let a_cast: DenseArray<u64> = self_da.map(|&x| x.saturating_cast());
                        elementwise_binary(&a_cast, b, |x, y| idiv_round_u64(*x, *y))
                            .map(Self::UInt64)
                    }
                    _ => Err(MathError::integer_class_mismatch()),
                }
            }
        }
    }

    // --- Relational and Logical Operations ---

    pub fn eq(&self, other: &Self) -> Result<Self, MathError> {
        let da = self.to_double_array()?;
        let db = other.to_double_array()?;
        let out = elementwise_binary(&da, &db, |x, y| {
            if x.is_nan() || y.is_nan() {
                false
            } else {
                x == y
            }
        })?;
        Ok(Self::Logical(out))
    }

    pub fn not_eq(&self, other: &Self) -> Result<Self, MathError> {
        let da = self.to_double_array()?;
        let db = other.to_double_array()?;
        let out = elementwise_binary(&da, &db, |x, y| {
            if x.is_nan() || y.is_nan() {
                true
            } else {
                x != y
            }
        })?;
        Ok(Self::Logical(out))
    }

    pub fn lt(&self, other: &Self) -> Result<Self, MathError> {
        let da = self.to_double_array()?;
        let db = other.to_double_array()?;
        let out = elementwise_binary(&da, &db, |x, y| x < y)?;
        Ok(Self::Logical(out))
    }

    pub fn lt_eq(&self, other: &Self) -> Result<Self, MathError> {
        let da = self.to_double_array()?;
        let db = other.to_double_array()?;
        let out = elementwise_binary(&da, &db, |x, y| x <= y)?;
        Ok(Self::Logical(out))
    }

    pub fn gt(&self, other: &Self) -> Result<Self, MathError> {
        let da = self.to_double_array()?;
        let db = other.to_double_array()?;
        let out = elementwise_binary(&da, &db, |x, y| x > y)?;
        Ok(Self::Logical(out))
    }

    pub fn gt_eq(&self, other: &Self) -> Result<Self, MathError> {
        let da = self.to_double_array()?;
        let db = other.to_double_array()?;
        let out = elementwise_binary(&da, &db, |x, y| x >= y)?;
        Ok(Self::Logical(out))
    }

    pub fn and(&self, other: &Self) -> Result<Self, MathError> {
        let da = self.to_double_array()?;
        let db = other.to_double_array()?;
        let out = elementwise_binary(&da, &db, |x, y| *x != 0.0 && *y != 0.0)?;
        Ok(Self::Logical(out))
    }

    pub fn or(&self, other: &Self) -> Result<Self, MathError> {
        let da = self.to_double_array()?;
        let db = other.to_double_array()?;
        let out = elementwise_binary(&da, &db, |x, y| *x != 0.0 || *y != 0.0)?;
        Ok(Self::Logical(out))
    }

    pub fn not(&self) -> Result<Self, MathError> {
        let da = self.to_double_array()?;
        let out = elementwise_unary(&da, |x| *x == 0.0);
        Ok(Self::Logical(out))
    }

    /// Extracts elements matching a logical mask array into a vector.
    pub fn get_mask(&self, mask: &DenseArray<bool>) -> Result<Self, MathError> {
        match self {
            Self::Double(a) => Ok(Self::Double(a.get_mask(mask)?)),
            Self::Single(a) => Ok(Self::Single(a.get_mask(mask)?)),
            Self::ComplexDouble(a) => Ok(Self::ComplexDouble(a.get_mask(mask)?)),
            Self::Logical(a) => Ok(Self::Logical(a.get_mask(mask)?)),
            Self::Int8(a) => Ok(Self::Int8(a.get_mask(mask)?)),
            Self::Int16(a) => Ok(Self::Int16(a.get_mask(mask)?)),
            Self::Int32(a) => Ok(Self::Int32(a.get_mask(mask)?)),
            Self::Int64(a) => Ok(Self::Int64(a.get_mask(mask)?)),
            Self::UInt8(a) => Ok(Self::UInt8(a.get_mask(mask)?)),
            Self::UInt16(a) => Ok(Self::UInt16(a.get_mask(mask)?)),
            Self::UInt32(a) => Ok(Self::UInt32(a.get_mask(mask)?)),
            Self::UInt64(a) => Ok(Self::UInt64(a.get_mask(mask)?)),
            Self::Char(a) => Ok(Self::Char(a.get_mask(mask)?)),
            Self::String(a) => Ok(Self::String(a.get_mask(mask)?)),
            Self::Cell(a) => Ok(Self::Cell(a.get_mask(mask)?)),
            other => Err(MathError::UnsupportedOperation {
                op: "get_mask",
                type_name: other.type_name(),
            }),
        }
    }

    /// Sets elements matching a logical mask to assigned values.
    pub fn set_mask(&mut self, mask: &DenseArray<bool>, val: &Value) -> Result<(), MathError> {
        match (self, val) {
            (Self::Double(a), Self::Double(v)) => a.set_mask(mask, v.as_slice()),
            (Self::Double(a), other) => {
                if let Ok(da) = other.to_double_array() {
                    a.set_mask(mask, da.as_slice())
                } else {
                    Err(MathError::dim_agree(a.shape(), other.shape()))
                }
            }
            (Self::Single(a), Self::Single(v)) => a.set_mask(mask, v.as_slice()),
            (Self::ComplexDouble(a), Self::ComplexDouble(v)) => a.set_mask(mask, v.as_slice()),
            (Self::Logical(a), Self::Logical(v)) => a.set_mask(mask, v.as_slice()),
            (Self::Int8(a), Self::Int8(v)) => a.set_mask(mask, v.as_slice()),
            (Self::Int16(a), Self::Int16(v)) => a.set_mask(mask, v.as_slice()),
            (Self::Int32(a), Self::Int32(v)) => a.set_mask(mask, v.as_slice()),
            (Self::Int64(a), Self::Int64(v)) => a.set_mask(mask, v.as_slice()),
            (Self::UInt8(a), Self::UInt8(v)) => a.set_mask(mask, v.as_slice()),
            (Self::UInt16(a), Self::UInt16(v)) => a.set_mask(mask, v.as_slice()),
            (Self::UInt32(a), Self::UInt32(v)) => a.set_mask(mask, v.as_slice()),
            (Self::UInt64(a), Self::UInt64(v)) => a.set_mask(mask, v.as_slice()),
            (Self::Cell(a), Self::Cell(v)) => a.set_mask(mask, v.as_slice()),
            (target, _) => Err(MathError::UnsupportedOperation {
                op: "set_mask",
                type_name: target.type_name(),
            }),
        }
    }
}

/// Formats a finite double in MATLAB `format short` scientific notation:
/// four decimal places, an explicit exponent sign, and a 2-digit zero-padded
/// exponent (e.g. `1.0000e-16`, `5.0000e-04`, `1.2346e+04`, `1.0000e+10`).
pub(crate) fn format_short_scientific(x: f64) -> String {
    if !(x.is_finite()) {
        return if x.is_nan() {
            "NaN".to_string()
        } else {
            "Inf".to_string()
        };
    }
    let sign = if x < 0.0 { "-" } else { "" };
    let s = format!("{:.4e}", x.abs());
    match s.split_once('e') {
        Some((mantissa, exponent)) => match exponent.parse::<i32>() {
            Ok(exp) => format!("{sign}{mantissa}e{exp:+03}"),
            Err(_) => s,
        },
        None => s,
    }
}

fn format_short_f64(x: f64, is_all_int: bool, scale: f64) -> String {
    if x.is_nan() {
        "NaN".to_string()
    } else if x == f64::INFINITY {
        "Inf".to_string()
    } else if x == f64::NEG_INFINITY {
        "-Inf".to_string()
    } else {
        let v = x / scale;
        if is_all_int || (v.fract() == 0.0 && v.abs() < 1e12) {
            format!("{}", v.round() as i64)
        } else {
            format!("{:.4}", v)
        }
    }
}

fn format_integer_dense_array<T: fmt::Display + Copy>(
    f: &mut fmt::Formatter<'_>,
    arr: &DenseArray<T>,
) -> fmt::Result {
    let shp = arr.shape();
    if arr.is_empty() {
        return write!(f, "   []");
    }
    if arr.is_scalar() {
        return write!(f, "    {}", arr.as_slice()[0]);
    }
    if shp.len() != 2 {
        return write!(f, "<{}D integer array>", shp.len());
    }

    let rows = shp[0];
    let cols = shp[1];
    let mut cell_matrix = Vec::with_capacity(rows);
    let mut col_widths = vec![4; cols];

    for r in 1..=rows {
        let mut row_strs = Vec::with_capacity(cols);
        for c in 1..=cols {
            let val = arr.as_slice()[(r - 1) + (c - 1) * rows];
            let s = format!("{}", val);
            col_widths[c - 1] = col_widths[c - 1].max(s.len());
            row_strs.push(s);
        }
        cell_matrix.push(row_strs);
    }

    let uniform_width = col_widths.into_iter().max().unwrap_or(4);

    for (r, row) in cell_matrix.iter().enumerate() {
        write!(f, "  ")?;
        for cell in row.iter() {
            write!(f, "  {:>width$}", cell, width = uniform_width)?;
        }
        if r + 1 < rows {
            writeln!(f)?;
        }
    }
    Ok(())
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Double(arr) => {
                if arr.is_empty() {
                    return write!(f, "   []");
                }
                if arr.is_scalar() {
                    let x = arr.as_slice()[0];
                    if x.is_nan() {
                        return write!(f, "   NaN");
                    }
                    if x == f64::INFINITY {
                        return write!(f, "   Inf");
                    }
                    if x == f64::NEG_INFINITY {
                        return write!(f, "  -Inf");
                    }
                    // Integral values below the fixed-point display limit print with
                    // full digits (MATLAB `format short` keeps e.g. 100000 verbatim),
                    // while 1e10 and beyond switch to scientific notation.
                    if x.fract() == 0.0 && x.abs() < 1e10 {
                        return write!(f, "    {}", x as i64);
                    }
                    // Unverified scalar threshold rule (A3.1.5): non-integer
                    // scalars at or above 1e3 use scientific notation. The
                    // MathWorks `format` page only documents the matrix rule.
                    if x != 0.0 && (x.abs() < 1e-3 || x.abs() >= 1e3) {
                        return write!(f, "   {}", format_short_scientific(x));
                    }
                    return write!(f, "    {:.4}", x);
                }
                let rows = arr.shape()[0];
                let cols = arr.shape()[1];

                let is_all_int = arr.as_slice().iter().all(|&x| {
                    x.is_nan() || x.is_infinite() || (x.fract() == 0.0 && x.abs() < 1e12)
                });

                let max_finite = arr
                    .as_slice()
                    .iter()
                    .filter(|x| x.is_finite())
                    .map(|x| x.abs())
                    .fold(0.0f64, f64::max);

                // MathWorks `format` rule: "If the largest element of a matrix
                // is larger than 1000 or smaller than 0.001, MATLAB applies a
                // common scale factor for the short and long formats."
                // Strict inequalities are deliberate (boundary 1000/0.001 does
                // not scale); the boundary golden is marked unverified.
                let (scale, has_scale) = if !is_all_int
                    && (max_finite > 1e3 || (max_finite < 1e-3 && max_finite > 0.0))
                {
                    let exp = max_finite.log10().floor() as i32;
                    (10.0f64.powi(exp), Some(exp))
                } else {
                    (1.0, None)
                };

                if let Some(exp) = has_scale {
                    writeln!(f, "  1.0e{:+03} *\n", exp)?;
                }

                let mut cell_matrix = Vec::with_capacity(rows);
                for r in 1..=rows {
                    let mut row_strs = Vec::with_capacity(cols);
                    for c in 1..=cols {
                        let val = *arr.get_subscript(&[r, c]).unwrap_or(&0.0);
                        row_strs.push(format_short_f64(val, is_all_int, scale));
                    }
                    cell_matrix.push(row_strs);
                }

                let mut col_widths = vec![4; cols];
                for row in &cell_matrix {
                    for (c, cell) in row.iter().enumerate() {
                        col_widths[c] = col_widths[c].max(cell.len());
                    }
                }

                let uniform_col_width = col_widths.into_iter().max().unwrap_or(4);

                for (r, row) in cell_matrix.iter().enumerate() {
                    write!(f, "  ")?;
                    for cell in row.iter() {
                        write!(f, "  {:>width$}", cell, width = uniform_col_width)?;
                    }
                    if r + 1 < rows {
                        writeln!(f)?;
                    }
                }
                Ok(())
            }
            Self::Single(arr) => {
                let da = arr.map(|&x| x as f64);
                write!(f, "{}", Value::Double(da))
            }
            Self::ComplexDouble(arr) => {
                if arr.is_empty() {
                    return write!(f, "   []");
                }
                if arr.is_scalar() {
                    let c = arr.as_slice()[0];
                    let sign = if c.im >= 0.0 { "+" } else { "-" };
                    if c.re.fract() == 0.0
                        && c.im.fract() == 0.0
                        && c.re.abs() < 1e12
                        && c.im.abs() < 1e12
                    {
                        return write!(f, "    {} {} {}i", c.re as i64, sign, c.im.abs() as i64);
                    }
                    return write!(f, "    {:.4} {} {:.4}i", c.re, sign, c.im.abs());
                }
                let rows = arr.shape()[0];
                let cols = arr.shape()[1];
                let default_c = Complex64::new(0.0, 0.0);

                let is_all_int = arr.as_slice().iter().all(|c| {
                    (c.re.is_nan()
                        || c.re.is_infinite()
                        || (c.re.fract() == 0.0 && c.re.abs() < 1e12))
                        && (c.im.is_nan()
                            || c.im.is_infinite()
                            || (c.im.fract() == 0.0 && c.im.abs() < 1e12))
                });

                let mut cell_matrix = Vec::with_capacity(rows);
                for r in 1..=rows {
                    let mut row_strs = Vec::with_capacity(cols);
                    for c in 1..=cols {
                        let val = *arr.get_subscript(&[r, c]).unwrap_or(&default_c);
                        let sign = if val.im >= 0.0 { "+" } else { "-" };
                        let s = if is_all_int {
                            format!("{} {} {}i", val.re as i64, sign, val.im.abs() as i64)
                        } else {
                            format!("{:.4} {} {:.4}i", val.re, sign, val.im.abs())
                        };
                        row_strs.push(s);
                    }
                    cell_matrix.push(row_strs);
                }

                let mut col_widths = vec![4; cols];
                for row in &cell_matrix {
                    for (c, cell) in row.iter().enumerate() {
                        col_widths[c] = col_widths[c].max(cell.len());
                    }
                }
                let uniform_col_width = col_widths.into_iter().max().unwrap_or(4);

                for (r, row) in cell_matrix.iter().enumerate() {
                    write!(f, "  ")?;
                    for cell in row.iter() {
                        write!(f, "  {:>width$}", cell, width = uniform_col_width)?;
                    }
                    if r + 1 < rows {
                        writeln!(f)?;
                    }
                }
                Ok(())
            }
            Self::Logical(arr) => {
                let rows = arr.shape()[0];
                let cols = arr.shape()[1];
                for r in 1..=rows {
                    write!(f, "  ")?;
                    for c in 1..=cols {
                        let val = arr.get_subscript(&[r, c]).unwrap_or(&false);
                        write!(f, "   {}", if *val { 1 } else { 0 })?;
                    }
                    if r < rows {
                        writeln!(f)?;
                    }
                }
                Ok(())
            }
            Self::Int8(arr) => format_integer_dense_array(f, arr),
            Self::Int16(arr) => format_integer_dense_array(f, arr),
            Self::Int32(arr) => format_integer_dense_array(f, arr),
            Self::Int64(arr) => format_integer_dense_array(f, arr),
            Self::UInt8(arr) => format_integer_dense_array(f, arr),
            Self::UInt16(arr) => format_integer_dense_array(f, arr),
            Self::UInt32(arr) => format_integer_dense_array(f, arr),
            Self::UInt64(arr) => format_integer_dense_array(f, arr),
            Self::Sparse(s) => write!(f, "{}", s.format_matlab_display()),
            Self::Char(arr) => {
                let text: String = arr.as_slice().iter().collect();
                write!(f, "'{}'", text)
            }
            Self::String(arr) => {
                if arr.is_scalar() {
                    write!(f, "\"{}\"", arr.as_slice()[0])
                } else {
                    write!(f, "<{}x{} string array>", arr.shape()[0], arr.shape()[1])
                }
            }
            Self::Cell(arr) => {
                if arr.is_empty() {
                    return write!(f, "   {{}}");
                }
                let rows = arr.shape()[0];
                let cols = arr.shape()[1];
                for r in 1..=rows {
                    write!(f, "  ")?;
                    for c in 1..=cols {
                        let item = &arr.as_slice()[(r - 1) + (c - 1) * rows];
                        write!(f, "  [{}]", item)?;
                    }
                    if r < rows {
                        writeln!(f)?;
                    }
                }
                Ok(())
            }
            Self::Struct(s) => {
                if s.shape == [1, 1] && !s.data.is_empty() {
                    writeln!(f, "  struct with fields:\n")?;
                    let m = &s.data[0];
                    let default_val = Value::empty();
                    for (i, field) in s.fields.iter().enumerate() {
                        let val = m.get(field).unwrap_or(&default_val);
                        write!(f, "    {}: {}", field, val)?;
                        if i + 1 < s.fields.len() {
                            writeln!(f)?;
                        }
                    }
                    Ok(())
                } else {
                    write!(f, "<{}x{} struct array>", s.shape[0], s.shape[1])
                }
            }
            Self::FunctionHandle(h) => match &**h {
                FunctionHandle::Named(name) => write!(f, "@{}", name),
                FunctionHandle::Anonymous { params, .. } => {
                    write!(f, "@({}) ...", params.join(", "))
                }
            },
        }
    }
}
