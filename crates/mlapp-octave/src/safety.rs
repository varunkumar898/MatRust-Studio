//! Safety policy and allow-list verification for Octave bridge function execution.

/// Set of hazardous functions strictly blocked by default to prevent arbitrary system execution.
pub const BLOCKED_FUNCTIONS: &[&str] = &[
    "system",
    "dos",
    "unix",
    "unlink",
    "delete",
    "rmdir",
    "mkdir",
    "exit",
    "quit",
    "source",
    "run",
    "eval",
    "evalin",
    "assignin",
    "feval",
    "builtin",
    "javaMethod",
    "javaObject",
    "python_invoke",
    "popen",
    "popen2",
];

/// Known-safe scientific, mathematical, and numerical builtins.
pub const SAFE_BUILTINS: &[&str] = &[
    "sin",
    "cos",
    "tan",
    "asin",
    "acos",
    "atan",
    "atan2",
    "sinh",
    "cosh",
    "tanh",
    "exp",
    "log",
    "log10",
    "log2",
    "sqrt",
    "cbrt",
    "hypot",
    "abs",
    "sign",
    "round",
    "floor",
    "ceil",
    "fix",
    "mod",
    "rem",
    "sum",
    "prod",
    "mean",
    "median",
    "mode",
    "std",
    "var",
    "min",
    "max",
    "zeros",
    "ones",
    "eye",
    "rand",
    "randn",
    "linspace",
    "logspace",
    "magic",
    "size",
    "length",
    "numel",
    "ndims",
    "isempty",
    "isscalar",
    "isvector",
    "ismatrix",
    "reshape",
    "repmat",
    "squeeze",
    "diag",
    "fliplr",
    "flipud",
    "rot90",
    "cat",
    "inv",
    "det",
    "rank",
    "trace",
    "norm",
    "cond",
    "null",
    "orth",
    "rref",
    "lu",
    "qr",
    "chol",
    "svd",
    "eig",
    "pinv",
    "expm",
    "logm",
    "sqrtm",
    "kron",
    "fft",
    "ifft",
    "fft2",
    "ifft2",
    "fftn",
    "ifftn",
    "diff",
    "gradient",
    "trapz",
    "cumtrapz",
    "polyfit",
    "polyval",
    "roots",
    "conv",
    "fzero",
    "fminsearch",
    "fsolve",
];

/// Checks if a named function is allowed under the current execution policy.
pub fn check_call_safety(fn_name: &str, allow_unsafe: bool) -> Result<(), String> {
    if BLOCKED_FUNCTIONS
        .iter()
        .any(|b| b.eq_ignore_ascii_case(fn_name))
    {
        if allow_unsafe {
            return Ok(());
        }
        return Err(format!(
            "Security violation: execution of dangerous function '{fn_name}' is blocked by default Octave safety policy. Enable allow_unsafe to permit."
        ));
    }

    Ok(())
}

/// Returns true if the function is a verified safe numerical builtin.
pub fn is_recognized_safe(fn_name: &str) -> bool {
    SAFE_BUILTINS
        .iter()
        .any(|s| s.eq_ignore_ascii_case(fn_name))
}
