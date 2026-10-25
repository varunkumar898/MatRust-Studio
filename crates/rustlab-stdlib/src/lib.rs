//! # `rustlab-stdlib`
//!
//! Clean-room MATLAB-compatible Standard Library for RustLab.
//! Registers numerical generators, array inspectors, elementary math,
//! linear algebra decompositions, summary statistics, transforms, sorting,
//! formatting/IO, strings/regex, and diagnostics.

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![allow(clippy::needless_range_loop, clippy::type_complexity)]

pub mod containers;
pub mod diagnostics;
pub mod generators;
pub mod help;
pub mod helpers;
pub mod io_format;
pub mod linalg;
pub mod math_elem;
pub mod matio_builtins;
pub mod nd_ops;
pub mod numeric_types;
pub mod search_sort;
pub mod shape_inspect;
pub mod signal;
pub mod sparse_builtins;
pub mod stats;
pub mod strings;

use rustlab_eval::{EvalError, Interpreter};
use rustlab_math::Value;

/// Registers all standard library functions into the provided `Interpreter`.
pub fn register_all(interp: &mut Interpreter) {
    // Generators (Tier 1)
    interp.register_builtin("zeros", generators::zeros);
    interp.register_builtin("ones", generators::ones);
    interp.register_builtin("eye", generators::eye);
    interp.register_builtin("rand", generators::rand);
    interp.register_builtin("randn", generators::randn);
    interp.register_builtin("linspace", generators::linspace);
    interp.register_builtin("logspace", generators::logspace);
    interp.register_builtin("magic", generators::magic);
    interp.register_builtin("true", generators::true_fn);
    interp.register_builtin("false", generators::false_fn);

    // Shape & Array Inspection (Tier 2)
    interp.register_builtin("size", shape_inspect::size);
    interp.register_builtin("length", shape_inspect::length);
    interp.register_builtin("numel", shape_inspect::numel);
    interp.register_builtin("ndims", shape_inspect::ndims);
    interp.register_builtin("isempty", shape_inspect::isempty);
    interp.register_builtin("isscalar", shape_inspect::isscalar);
    interp.register_builtin("isvector", shape_inspect::isvector);
    interp.register_builtin("ismatrix", shape_inspect::ismatrix);
    interp.register_builtin("reshape", nd_ops::reshape);
    interp.register_builtin("repmat", nd_ops::repmat);
    interp.register_builtin("squeeze", shape_inspect::squeeze);
    interp.register_builtin("diag", shape_inspect::diag);
    interp.register_builtin("fliplr", shape_inspect::fliplr);
    interp.register_builtin("flipud", shape_inspect::flipud);
    interp.register_builtin("rot90", shape_inspect::rot90);
    interp.register_builtin("cat", nd_ops::cat);
    interp.register_builtin("horzcat", nd_ops::horzcat);
    interp.register_builtin("vertcat", nd_ops::vertcat);
    interp.register_builtin("permute", nd_ops::permute);
    interp.register_builtin("ipermute", nd_ops::ipermute);
    interp.register_builtin("circshift", nd_ops::circshift);
    interp.register_builtin("meshgrid", nd_ops::meshgrid);
    interp.register_builtin("ndgrid", nd_ops::ndgrid);

    // Elementary Math & Predicates (Tier 3)
    interp.register_builtin("abs", math_elem::abs_fn);
    interp.register_builtin("sign", math_elem::sign_fn);
    interp.register_builtin("sqrt", math_elem::sqrt_fn);
    interp.register_builtin("exp", math_elem::exp_fn);
    interp.register_builtin("log", math_elem::log_fn);
    interp.register_builtin("log10", math_elem::log10_fn);
    interp.register_builtin("log2", math_elem::log2_fn);
    interp.register_builtin("sin", math_elem::sin_fn);
    interp.register_builtin("cos", math_elem::cos_fn);
    interp.register_builtin("tan", math_elem::tan_fn);
    interp.register_builtin("asin", math_elem::asin_fn);
    interp.register_builtin("acos", math_elem::acos_fn);
    interp.register_builtin("atan", math_elem::atan_fn);
    interp.register_builtin("atan2", math_elem::atan2_fn);
    interp.register_builtin("round", math_elem::round_fn);
    interp.register_builtin("floor", math_elem::floor_fn);
    interp.register_builtin("ceil", math_elem::ceil_fn);
    interp.register_builtin("fix", math_elem::fix_fn);
    interp.register_builtin("mod", math_elem::mod_fn);
    interp.register_builtin("rem", math_elem::rem_fn);
    interp.register_builtin("isnan", math_elem::isnan_fn);
    interp.register_builtin("isinf", math_elem::isinf_fn);
    interp.register_builtin("isfinite", math_elem::isfinite_fn);

    // Reductions & Statistics (Tier 4)
    interp.register_builtin("sum", stats::sum_fn);
    interp.register_builtin("prod", stats::prod_fn);
    interp.register_builtin("cumsum", stats::cumsum_fn);
    interp.register_builtin("cumprod", stats::cumprod_fn);
    interp.register_builtin("diff", stats::diff_fn);
    interp.register_builtin("mean", stats::mean_fn);
    interp.register_builtin("var", stats::var_fn);
    interp.register_builtin("std", stats::std_fn);
    interp.register_builtin("min", stats::min_fn);
    interp.register_builtin("max", stats::max_fn);
    interp.register_builtin("any", stats::any_fn);
    interp.register_builtin("all", stats::all_fn);

    // Linear Algebra (Tier 6)
    interp.register_builtin("inv", linalg::inv_fn);
    interp.register_builtin("det", linalg::det_fn);
    interp.register_builtin("trace", linalg::trace_fn);
    interp.register_builtin("norm", linalg::norm_fn);
    interp.register_builtin("mldivide", linalg::mldivide_fn);
    interp.register_builtin("mrdivide", linalg::mrdivide_fn);
    interp.register_builtin("lu", linalg::lu_fn);
    interp.register_builtin("qr", linalg::qr_fn);
    interp.register_builtin("chol", linalg::chol_fn);
    interp.register_builtin("eig", linalg::eig_fn);
    interp.register_builtin("svd", linalg::svd_fn);
    interp.register_builtin("pinv", linalg::pinv_fn);
    interp.register_builtin("rank", linalg::rank_fn);
    interp.register_builtin("cond", linalg::cond_fn);
    interp.register_builtin("null", linalg::null_fn);
    interp.register_builtin("orth", linalg::orth_fn);
    interp.register_builtin("expm", linalg::expm_fn);
    interp.register_builtin("kron", linalg::kron_fn);

    // Transforms & Complex (Tier 7)
    interp.register_builtin("fft", signal::fft_fn);
    interp.register_builtin("ifft", signal::ifft_fn);
    interp.register_builtin("fft2", signal::fft2_fn);
    interp.register_builtin("ifft2", signal::ifft2_fn);
    interp.register_builtin("fftshift", signal::fftshift_fn);
    interp.register_builtin("conv", signal::conv_fn);
    interp.register_builtin("real", signal::real_fn);
    interp.register_builtin("imag", signal::imag_fn);
    interp.register_builtin("angle", signal::angle_fn);
    interp.register_builtin("conj", signal::conj_fn);

    // Sorting & Searching (Tier 5)
    interp.register_builtin("sort", search_sort::sort_fn);
    interp.register_builtin("find", search_sort::find_fn);
    interp.register_builtin("unique", search_sort::unique_fn);

    // Containers & Strings (Tier 8)
    interp.register_builtin("cell", containers::cell_fn);
    interp.register_builtin("struct", containers::struct_fn);
    interp.register_builtin("isfield", containers::isfield_fn);
    interp.register_builtin("fieldnames", containers::fieldnames_fn);
    interp.register_builtin("string", containers::string_fn);
    interp.register_builtin("char", containers::char_fn);
    interp.register_builtin("strlength", containers::strlength_fn);

    // String & Regular Expression Library
    interp.register_builtin("strsplit", strings::strsplit_fn);
    interp.register_builtin("strjoin", strings::strjoin_fn);
    interp.register_builtin("strrep", strings::strrep_fn);
    interp.register_builtin("strtrim", strings::strtrim_fn);
    interp.register_builtin("upper", strings::upper_fn);
    interp.register_builtin("lower", strings::lower_fn);
    interp.register_builtin("strcmp", strings::strcmp_fn);
    interp.register_builtin("strcmpi", strings::strcmpi_fn);
    interp.register_builtin("strncmp", strings::strncmp_fn);
    interp.register_builtin("contains", strings::contains_fn);
    interp.register_builtin("startsWith", strings::starts_with_fn);
    interp.register_builtin("endsWith", strings::ends_with_fn);
    interp.register_builtin("regexp", strings::regexp_fn);
    interp.register_builtin("regexprep", strings::regexprep_fn);

    // Formatting & I/O
    interp.register_builtin("sprintf", io_format::sprintf_fn);
    interp.register_builtin("fprintf", io_format::fprintf_fn);
    interp.register_builtin("disp", io_format::disp_fn);
    interp.register_builtin("num2str", io_format::num2str_fn);
    interp.register_builtin("mat2str", io_format::mat2str_fn);

    // Diagnostics & Assertions
    interp.register_builtin("error", diagnostics::error_fn);
    interp.register_builtin("warning", diagnostics::warning_fn);
    interp.register_builtin("assert", diagnostics::assert_fn);
    interp.register_builtin("help", help::help_fn);

    // MAT-file I/O
    interp.register_builtin("load", matio_builtins::load);
    interp.register_builtin("save", matio_builtins::save);

    // Numeric Types & Introspection (Milestone A3)
    interp.register_builtin("single", numeric_types::single);
    interp.register_builtin("double", numeric_types::double);
    interp.register_builtin("int8", numeric_types::int8);
    interp.register_builtin("int16", numeric_types::int16);
    interp.register_builtin("int32", numeric_types::int32);
    interp.register_builtin("int64", numeric_types::int64);
    interp.register_builtin("uint8", numeric_types::uint8);
    interp.register_builtin("uint16", numeric_types::uint16);
    interp.register_builtin("uint32", numeric_types::uint32);
    interp.register_builtin("uint64", numeric_types::uint64);
    interp.register_builtin("logical", numeric_types::logical);
    interp.register_builtin("cast", numeric_types::cast);
    interp.register_builtin("class", numeric_types::class);
    interp.register_builtin("isa", numeric_types::isa);
    interp.register_builtin("isinteger", numeric_types::isinteger);
    interp.register_builtin("isfloat", numeric_types::isfloat);
    interp.register_builtin("isnumeric", numeric_types::isnumeric);
    interp.register_builtin("issparse", numeric_types::issparse);
    interp.register_builtin("intmax", numeric_types::intmax);
    interp.register_builtin("intmin", numeric_types::intmin);
    interp.register_builtin("realmax", numeric_types::realmax);
    interp.register_builtin("realmin", numeric_types::realmin);
    interp.register_builtin("flintmax", numeric_types::flintmax);
    interp.register_builtin("eps", numeric_types::eps);
    interp.register_builtin("idivide", numeric_types::idivide);
    interp.register_builtin("bitand", numeric_types::bitand);
    interp.register_builtin("bitor", numeric_types::bitor);
    interp.register_builtin("bitxor", numeric_types::bitxor);
    interp.register_builtin("bitshift", numeric_types::bitshift);

    // Sparse Matrices CSC (Milestone A3)
    interp.register_builtin("sparse", sparse_builtins::sparse);
    interp.register_builtin("full", sparse_builtins::full);
    interp.register_builtin("speye", sparse_builtins::speye);
    interp.register_builtin("nnz", sparse_builtins::nnz);
    interp.register_builtin("nonzeros", sparse_builtins::nonzeros);
    interp.register_builtin("spones", sparse_builtins::spones);
}

/// Creates a new `Interpreter` pre-loaded with the complete MATLAB standard library.
pub fn new_interpreter() -> Interpreter {
    let mut interp = Interpreter::new();
    register_all(&mut interp);
    interp
}

/// Evaluates a MATLAB code snippet using the complete standard library.
pub fn eval_matlab(source: &str) -> Result<Option<Value>, EvalError> {
    let mut interp = new_interpreter();
    interp.eval_script(source)
}
