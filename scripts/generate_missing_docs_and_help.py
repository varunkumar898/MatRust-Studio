import os
import re

BUILTIN_INFO = {
    "abs": ("Absolute value and complex magnitude", "Y = abs(X)", "Returns the absolute value of real elements or complex magnitude sqrt(re^2 + im^2)."),
    "acos": ("Inverse cosine", "Y = acos(X)", "Computes the inverse cosine (in radians) for each element of X."),
    "all": ("Determine if all array elements are nonzero or true", "B = all(A)\nB = all(A, dim)", "Tests along dimension dim whether all elements are nonzero or logical true."),
    "angle": ("Phase angle of complex number", "P = angle(Z)", "Returns the phase angle in radians in the interval [-pi, pi] for each element of Z."),
    "any": ("Determine if any array elements are nonzero or true", "B = any(A)\nB = any(A, dim)", "Tests along dimension dim whether any element is nonzero or logical true."),
    "asin": ("Inverse sine", "Y = asin(X)", "Computes the inverse sine (in radians) for each element of X."),
    "atan": ("Inverse tangent", "Y = atan(X)", "Computes the inverse tangent (in radians) for each element of X."),
    "atan2": ("Four-quadrant inverse tangent", "P = atan2(Y, X)", "Computes four-quadrant inverse tangent of Y and X such that -pi <= P <= pi."),
    "ceil": ("Round toward positive infinity", "Y = ceil(X)", "Rounds each element of X to the nearest integer greater than or equal to that element."),
    "cell": ("Create cell array", "C = cell(n)\nC = cell(sz1, ..., szN)", "Constructs an empty cell array of the specified dimensions with each element initialized to []."),
    "char": ("Convert to character array", "S = char(X)", "Converts numeric codes, strings, or cell arrays of character vectors to a char array."),
    "chol": ("Cholesky factorization", "R = chol(A)", "Computes upper triangular R such that R'*R = A for symmetric positive definite matrix A."),
    "conj": ("Complex conjugate", "Y = conj(X)", "Computes the complex conjugate of each element of X (re - 1i*im)."),
    "conv": ("Convolution and polynomial multiplication", "C = conv(A, B)\nC = conv(A, B, shape)", "Computes 1D discrete convolution of vectors A and B with full, same, or valid shape."),
    "cos": ("Cosine of argument in radians", "Y = cos(X)", "Computes cosine of each element of X in radians."),
    "cumprod": ("Cumulative product", "B = cumprod(A)\nB = cumprod(A, dim)", "Computes the cumulative product of elements along dimension dim."),
    "cumsum": ("Cumulative sum", "B = cumsum(A)\nB = cumsum(A, dim)", "Computes the cumulative sum of elements along dimension dim."),
    "det": ("Matrix determinant", "d = det(A)", "Computes the determinant of square matrix A using LU decomposition with partial pivoting."),
    "diag": ("Diagonal matrices and diagonals of matrix", "D = diag(v)\nv = diag(A)", "Extracts matrix diagonals or constructs diagonal matrices."),
    "diff": ("Differences and approximate derivatives", "Y = diff(X)\nY = diff(X, n, dim)", "Computes differences between adjacent elements along dimension dim."),
    "exp": ("Exponential", "Y = exp(X)", "Computes the natural exponential e^x for each element of X."),
    "eye": ("Identity matrix", "I = eye(n)\nI = eye(m, n)", "Constructs an m-by-n matrix with ones on the main diagonal and zeros elsewhere."),
    "false": ("Logical false array", "L = false(n)\nL = false(sz1, ..., szN)", "Creates a logical array with all elements set to logical 0 (false)."),
    "fftshift": ("Shift zero-frequency component to center of spectrum", "Y = fftshift(X)\nY = fftshift(X, dim)", "Rearranges outputs of fft, fft2, or fftn by moving zero-frequency to the center."),
    "fieldnames": ("Field names of structure array", "names = fieldnames(s)", "Returns a cell array of character vectors containing the field names of structure s."),
    "find": ("Find indices and values of nonzero elements", "k = find(X)\nk = find(X, n, direction)", "Returns linear indices of nonzero elements, with optional count limit and 'first'/'last' direction."),
    "fix": ("Round toward zero", "Y = fix(X)", "Rounds each element of X toward zero to the nearest integer."),
    "fliplr": ("Flip array left to right", "B = fliplr(A)", "Reverses the order of columns of matrix A."),
    "flipud": ("Flip array up to down", "B = flipud(A)", "Reverses the order of rows of matrix A."),
    "floor": ("Round toward negative infinity", "Y = floor(X)", "Rounds each element of X to the nearest integer less than or equal to that element."),
    "help": ("Display help documentation for builtin or M-file", "help name", "Displays interactive help documentation for the specified function or topic."),
    "horzcat": ("Horizontal concatenation", "C = horzcat(A1, A2, ...)", "Concatenates arrays horizontally along dimension 2. Equivalent to [A1, A2, ...]."),
    "imag": ("Imaginary part of complex number", "Y = imag(X)", "Returns the imaginary part of each element of X."),
    "int8": ("Convert to 8-bit signed integer", "Y = int8(X)", "Converts numeric array to 8-bit signed integer with saturating arithmetic [-128, 127]."),
    "int16": ("Convert to 16-bit signed integer", "Y = int16(X)", "Converts numeric array to 16-bit signed integer with saturating arithmetic [-32768, 32767]."),
    "int32": ("Convert to 32-bit signed integer", "Y = int32(X)", "Converts numeric array to 32-bit signed integer with saturating arithmetic [-2147483648, 2147483647]."),
    "int64": ("Convert to 64-bit signed integer", "Y = int64(X)", "Converts numeric array to 64-bit signed integer with exact arithmetic without f64 roundtrip."),
    "inv": ("Matrix inverse", "Y = inv(X)", "Computes the matrix inverse X^(-1) of square nonsingular matrix X using LU decomposition with partial pivoting."),
    "isempty": ("Determine whether array is empty", "tf = isempty(A)", "Returns true if array A has at least one dimension of size 0."),
    "isfield": ("Determine whether input is structure array field", "tf = isfield(s, field)", "Returns true if field is the name of a field in structure array s."),
    "isfinite": ("Array elements that are finite", "tf = isfinite(A)", "Returns true where elements of A are not NaN, Inf, or -Inf."),
    "isinf": ("Array elements that are infinite", "tf = isinf(A)", "Returns true where elements of A are Inf or -Inf."),
    "ismatrix": ("Determine whether input is matrix", "tf = ismatrix(A)", "Returns true if size(A) has exactly 2 dimensions."),
    "isnan": ("Array elements that are NaN", "tf = isnan(A)", "Returns true where elements of A are IEEE 754 Not-a-Number (NaN)."),
    "isscalar": ("Determine whether input is scalar", "tf = isscalar(A)", "Returns true if size(A) is [1, 1]."),
    "isvector": ("Determine whether input is vector", "tf = isvector(A)", "Returns true if size(A) is 1-by-N or N-by-1."),
    "length": ("Length of largest array dimension", "n = length(X)", "Returns max(size(X)) for array X, or 0 if X is empty."),
    "linspace": ("Linearly spaced vector", "y = linspace(x1, x2)\ny = linspace(x1, x2, n)", "Generates a row vector of n equally spaced points between x1 and x2."),
    "load": ("Load variables from MAT-file into workspace", "load(filename)", "Loads variables from Level 5 or v7 MAT-file into the caller workspace."),
    "log": ("Natural logarithm", "Y = log(X)", "Computes natural logarithm ln(x) elementwise for each element of X."),
    "log10": ("Common logarithm (base 10)", "Y = log10(X)", "Computes base-10 logarithm elementwise for each element of X."),
    "log2": ("Base 2 logarithm", "Y = log2(X)", "Computes base-2 logarithm elementwise for each element of X."),
    "logspace": ("Logarithmically spaced vector", "y = logspace(a, b)\ny = logspace(a, b, n)", "Generates a row vector of n logarithmically spaced points between 10^a and 10^b."),
    "lu": ("LU matrix factorization", "[L, U, P] = lu(A)", "Computes permutation matrix P, lower triangular L, and upper triangular U such that P*A = L*U."),
    "magic": ("Magic square", "M = magic(n)", "Constructs an n-by-n magic square with sum of rows, columns, and diagonals equal to n*(n^2+1)/2."),
    "max": ("Maximum elements of array", "M = max(A)\nM = max(A, [], dim)\n[M, I] = max(A)", "Finds largest elements along dimension dim, with optional linear index output."),
    "mean": ("Average or mean value of array", "M = mean(A)\nM = mean(A, dim)", "Computes the arithmetic mean along dimension dim."),
    "min": ("Minimum elements of array", "M = min(A)\nM = min(A, [], dim)\n[M, I] = min(A)", "Finds smallest elements along dimension dim, with optional linear index output."),
    "mldivide": ("Matrix left divide or linear system solve", "x = mldivide(A, B)\nx = A \\ B", "Solves linear system A*x = B using LU factorization with partial pivoting."),
    "mod": ("Modulus after division", "M = mod(X, Y)", "Computes modulus remainder after division: X - floor(X./Y).*Y."),
    "mrdivide": ("Matrix right divide", "x = mrdivide(B, A)\nx = B / A", "Solves linear system x*A = B. Equivalent to (A' \\ B')'."),
    "ndims": ("Number of array dimensions", "n = ndims(A)", "Returns the number of dimensions of array A (minimum 2)."),
    "norm": ("Vector and matrix norms", "n = norm(A)\nn = norm(A, p)", "Computes 1-norm, 2-norm, Frobenius norm, or infinity norm."),
    "numel": ("Number of elements in array", "n = numel(A)", "Returns the total number of elements in array A: prod(size(A))."),
    "ones": ("Array of all ones", "Y = ones(n)\nY = ones(sz1, ..., szN)", "Constructs an array of specified dimensions with all elements equal to 1.0."),
    "prod": ("Product of array elements", "B = prod(A)\nB = prod(A, dim)", "Computes product of array elements along dimension dim."),
    "rand": ("Uniformly distributed random numbers", "R = rand(n)\nR = rand(sz1, ..., szN)", "Generates array of pseudorandom values uniformly distributed on interval (0, 1)."),
    "randn": ("Normally distributed random numbers", "R = randn(n)\nR = randn(sz1, ..., szN)", "Generates array of pseudorandom numbers from standard normal distribution N(0, 1)."),
    "real": ("Real part of complex number", "Y = real(X)", "Returns the real component of each element of X."),
    "rem": ("Remainder after division", "R = rem(X, Y)", "Computes remainder after division: X - fix(X./Y).*Y."),
    "repmat": ("Repeat copies of array", "B = repmat(A, n)\nB = repmat(A, sz1, ..., szN)", "Tiles array A across the specified dimensions."),
    "rot90": ("Rotate array 90 degrees", "B = rot90(A)\nB = rot90(A, k)", "Rotates array counterclockwise by 90*k degrees in the first two dimensions."),
    "round": ("Round to nearest integer", "Y = round(X)", "Rounds each element of X to the nearest integer."),
    "save": ("Save workspace variables to MAT-file", "save(filename, 'var1', ...)", "Serializes workspace variables into a Level 5 MAT-file with optional compression."),
    "sign": ("Signum function", "Y = sign(X)", "Returns 1 for positive numbers, -1 for negative numbers, and 0 for zero."),
    "sin": ("Sine of argument in radians", "Y = sin(X)", "Computes sine of each element of X in radians."),
    "size": ("Array dimensions", "sz = size(A)\n[m, n] = size(A)\nd = size(A, dim)", "Returns the sizes of dimensions of array A."),
    "sort": ("Sort array elements", "B = sort(A)\n[B, I] = sort(A, dim, direction)", "Sorts elements along dimension dim in 'ascend' or 'descend' order."),
    "sqrt": ("Square root", "Y = sqrt(X)", "Computes square root elementwise for each element of X."),
    "std": ("Standard deviation", "s = std(A)\ns = std(A, w, dim)", "Computes standard deviation along dimension dim with normalization weight w (default 0 for N-1)."),
    "string": ("Convert to string array", "S = string(X)", "Converts numeric, character, or cell array to string array representation."),
    "strlength": ("Length of string elements", "L = strlength(S)", "Returns the number of characters in each string or character vector element."),
    "struct": ("Create structure array", "s = struct(field1, val1, ...)", "Constructs structure array with specified field names and field values."),
    "sum": ("Sum of array elements", "S = sum(A)\nS = sum(A, dim)", "Computes the arithmetic sum of elements along dimension dim."),
    "tan": ("Tangent of argument in radians", "Y = tan(X)", "Computes tangent of each element of X in radians."),
    "trace": ("Sum of diagonal elements of matrix", "t = trace(A)", "Computes the trace of square matrix A: sum(diag(A))."),
    "true": ("Logical true array", "L = true(n)\nL = true(sz1, ..., szN)", "Creates a logical array with all elements set to logical 1 (true)."),
    "uint8": ("Convert to 8-bit unsigned integer", "Y = uint8(X)", "Converts numeric array to 8-bit unsigned integer with saturating arithmetic [0, 255]."),
    "uint16": ("Convert to 16-bit unsigned integer", "Y = uint16(X)", "Converts numeric array to 16-bit unsigned integer with saturating arithmetic [0, 65535]."),
    "uint32": ("Convert to 32-bit unsigned integer", "Y = uint32(X)", "Converts numeric array to 32-bit unsigned integer with saturating arithmetic [0, 4294967295]."),
    "uint64": ("Convert to 64-bit unsigned integer", "Y = uint64(X)", "Converts numeric array to 64-bit unsigned integer with exact arithmetic without f64 roundtrip."),
    "unique": ("Unique values in array", "C = unique(A)\n[C, ia, ic] = unique(A)", "Returns sorted unique values of array A with index vectors."),
    "var": ("Variance", "v = var(A)\nv = var(A, w, dim)", "Computes variance along dimension dim with normalization weight w (default 0 for N-1)."),
    "vertcat": ("Vertical concatenation", "C = vertcat(A1, A2, ...)", "Concatenates arrays vertically along dimension 1. Equivalent to [A1; A2; ...]."),
    "zeros": ("Array of all zeros", "Y = zeros(n)\nY = zeros(sz1, ..., szN)", "Constructs an array of specified dimensions with all elements equal to 0.0."),
}

docs_dir = 'rustlab/docs/functions'
os.makedirs(docs_dir, exist_ok=True)

# 1. Create missing markdown documentation files
created_docs = 0
for name, (title, syntax, desc) in BUILTIN_INFO.items():
    doc_path = os.path.join(docs_dir, f"{name}.md")
    if not os.path.exists(doc_path):
        content = f"""# `{name}`

{title}.

## Syntax

```matlab
{syntax}
```

## Description

- `{syntax.splitlines()[0]}`: {desc}
- Implemented in RustLab following clean-room MATLAB-compatible specifications.

## References

1. MATLAB Documentation: `{name}`
2. GNU Octave Documentation: `{name}`
"""
        with open(doc_path, 'w', encoding='utf-8') as f:
            f.write(content)
        created_docs += 1

print(f"Generated {created_docs} missing markdown documentation files.")

# 2. Add missing help entries into help.rs
help_rs_path = 'rustlab/crates/rustlab-stdlib/src/help.rs'
with open(help_rs_path, 'r', encoding='utf-8') as f:
    help_content = f.read()

existing_help = set(re.findall(r'"([^"]+)"\s*=>\s*Some\(', help_content))

new_entries = []
for name, (title, syntax, desc) in BUILTIN_INFO.items():
    if name not in existing_help:
        esc_syntax = syntax.replace("\n", "\\n\\\n               ")
        esc_desc = desc.replace("\n", " ")
        entry = f'''        "{name}" => Some(
            "{name.upper()}  {title}.\\n\\n\\
             Syntax:\\n\\
               {esc_syntax}\\n\\n\\
             Description:\\n\\
               {esc_desc}\\n"
        ),'''
        new_entries.append(entry)

if new_entries:
    # Insert new entries right before `_ => None`
    target = '        _ => None,'
    replacement = '\n'.join(new_entries) + '\n' + target
    if target in help_content:
        updated_help = help_content.replace(target, replacement, 1)
        with open(help_rs_path, 'w', encoding='utf-8') as f:
            f.write(updated_help)
        print(f"Inserted {len(new_entries)} missing help entries into help.rs.")
    else:
        print("Could not find insertion target in help.rs")
else:
    print("All help entries already present.")
