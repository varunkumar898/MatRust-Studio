//! Interactive help system for RustLab standard library functions.

use crate::helpers::user_err;
use rustlab_eval::EvalError;
use rustlab_math::Value;

/// Returns the help documentation string for a given function name.
pub fn get_help_text(name: &str) -> Option<&'static str> {
    match name {
        "eig" => Some(
            "EIG  Eigenvalues and eigenvectors.\n\n\
             Syntax:\n\
               d = eig(A)\n\
               [V, D] = eig(A)\n\n\
             Description:\n\
               d = eig(A) returns a column vector containing the eigenvalues of square matrix A.\n\
               [V, D] = eig(A) produces a diagonal matrix D of eigenvalues and a full matrix V\n\
               whose columns are the corresponding eigenvectors so that A*V = V*D.\n\n\
             RustLab Implementation:\n\
               - Symmetric/Hermitian matrices use native Householder tridiagonalization\n\
                 and implicit QL algorithm with Wilkinson shifts, ascending eigenvalue sort,\n\
                 and sign normalization (leading maximum absolute component positive).\n\
               - Non-symmetric matrices use faer Schur decomposition.\n"
        ),
        "svd" => Some(
            "SVD  Singular value decomposition.\n\n\
             Syntax:\n\
               s = svd(A)\n\
               [U, S, V] = svd(A)\n\
               [U, S, V] = svd(A, 'econ')\n\n\
             Description:\n\
               s = svd(A) returns the singular values of matrix A in descending order.\n\
               [U, S, V] = svd(A) produces a diagonal matrix S of the same dimension as A and\n\
               unitary matrices U and V so that A = U*S*V'.\n\
               [U, S, V] = svd(A, 'econ') computes the economy-sized decomposition.\n"
        ),
        "pinv" => Some(
            "PINV  Moore-Penrose pseudoinverse.\n\n\
             Syntax:\n\
               B = pinv(A)\n\
               B = pinv(A, tol)\n\n\
             Description:\n\
               B = pinv(A) returns the Moore-Penrose pseudoinverse of A such that\n\
               A*B*A = A, B*A*B = B, and (A*B)' = A*B, (B*A)' = B*A.\n\
               Singular values below tol = max(m,n) * eps(sigma_1) are treated as zero.\n"
        ),
        "rank" => Some(
            "RANK  Matrix rank.\n\n\
             Syntax:\n\
               k = rank(A)\n\
               k = rank(A, tol)\n\n\
             Description:\n\
               k = rank(A) provides an estimate of the number of linearly independent rows or columns of A.\n\
               k is the number of singular values of A that are larger than the tolerance tol.\n\
               Default tol is max(size(A)) * eps(max(svd(A))).\n"
        ),
        "cond" => Some(
            "COND  Condition number with respect to inversion.\n\n\
             Syntax:\n\
               c = cond(A)\n\n\
             Description:\n\
               c = cond(A) returns the 2-norm condition number of matrix A, which is the ratio\n\
               of the largest singular value to the smallest singular value.\n\
               Returns Inf only for exactly singular matrices.\n"
        ),
        "null" => Some(
            "NULL  Null space orthonormal basis.\n\n\
             Syntax:\n\
               Z = null(A)\n\n\
             Description:\n\
               Z = null(A) returns an orthonormal basis for the null space of A.\n\
               That is, A*Z has negligible elements, size(Z,1) == size(A,2), and Z'*Z == I.\n"
        ),
        "orth" => Some(
            "ORTH  Range space orthonormal basis.\n\n\
             Syntax:\n\
               Q = orth(A)\n\n\
             Description:\n\
               Q = orth(A) returns an orthonormal basis for the range (column space) of A.\n\
               size(Q,1) == size(A,1), size(Q,2) == rank(A), and Q'*Q == I.\n"
        ),
        "expm" => Some(
            "EXPM  Matrix exponential.\n\n\
             Syntax:\n\
               Y = expm(X)\n\n\
             Description:\n\
               Y = expm(X) computes the matrix exponential of X using the scaling and squaring\n\
               method with a [6/6] Padé approximation (Higham 2005).\n"
        ),
        "kron" => Some(
            "KRON  Kronecker tensor product.\n\n\
             Syntax:\n\
               K = kron(A, B)\n\n\
             Description:\n\
               K = kron(A, B) returns the Kronecker tensor product of matrices A and B.\n\
               The result is a large matrix formed by multiplying each element of A by the entire matrix B.\n"
        ),
        "fft" => Some(
            "FFT  1D Fast Fourier Transform.\n\n\
             Syntax:\n\
               Y = fft(X)\n\
               Y = fft(X, n)\n\n\
             Description:\n\
               Y = fft(X) computes the discrete Fourier transform (DFT) using high-performance\n\
               RustFFT backend, supporting prime and arbitrary composite non-power-of-2 lengths.\n"
        ),
        "ifft" => Some(
            "IFFT  1D Inverse Fast Fourier Transform.\n\n\
             Syntax:\n\
               Y = ifft(X)\n\
               Y = ifft(X, n)\n\n\
             Description:\n\
               Y = ifft(X) computes the inverse discrete Fourier transform, normalized by 1/N.\n"
        ),
        "fft2" => Some(
            "FFT2  2D Fast Fourier Transform.\n\n\
             Syntax:\n\
               Y = fft2(X)\n\n\
             Description:\n\
               Y = fft2(X) returns the 2-dimensional discrete Fourier transform of matrix X\n\
               computed by 1D transforms along rows and columns.\n"
        ),
        "ifft2" => Some(
            "IFFT2  2D Inverse Fast Fourier Transform.\n\n\
             Syntax:\n\
               Y = ifft2(X)\n\n\
             Description:\n\
               Y = ifft2(X) returns the 2-dimensional inverse discrete Fourier transform.\n"
        ),
        "single" => Some(
            "SINGLE  Convert to single precision float.\n\n\
             Syntax:\n\
               Y = single(X)\n\n\
             Description:\n\
               Converts numerical array X into 32-bit single precision.\n"
        ),
        "int8" | "int16" | "int32" | "int64" | "uint8" | "uint16" | "uint32" | "uint64" => Some(
            "INTEGER CONVERSION (int8, int16, int32, int64, uint8, uint16, uint32, uint64)  Saturating integer conversion.\n\n\
             Description:\n\
               Converts numeric arrays into integer types using round half away from zero\n\
               and saturating arithmetic without overflow.\n"
        ),
        "idivide" => Some(
            "IDIVIDE  Integer division with specified rounding mode.\n\n\
             Syntax:\n\
               C = idivide(A, B)\n\
               C = idivide(A, B, opt)\n\n\
             Description:\n\
               Divides integer arrays A and B using rounding mode opt:\n\
               'fix'   - Round toward zero (default)\n\
               'floor' - Round toward -inf\n\
               'ceil'  - Round toward +inf\n\
               'round' - Round to nearest, ties away from zero\n"
        ),
        "sparse" => Some(
            "SPARSE  Create Compressed Sparse Column (CSC) matrix.\n\n\
             Syntax:\n\
               S = sparse(A)\n\
               S = sparse(i, j, v, m, n)\n\n\
             Description:\n\
               Creates a CSC sparse matrix. When constructed from triplets (i, j, v),\n\
               duplicate coordinates are accumulated by summation.\n"
        ),
        "full" => Some(
            "FULL  Convert sparse matrix to full dense matrix.\n\n\
             Syntax:\n\
               A = full(S)\n"
        ),
        "speye" => Some(
            "SPEYE  Sparse identity matrix.\n\n\
             Syntax:\n\
               S = speye(m, n)\n"
        ),
        "cat" => Some(
            "CAT  Concatenate arrays.\n\n\
             Syntax:\n\
               C = cat(dim, A, B, ...)\n\n\
             Description:\n\
               Concatenates arrays along dimension dim (1-based).\n"
        ),
        "permute" => Some(
            "PERMUTE  Rearrange dimensions of N-D array.\n\n\
             Syntax:\n\
               B = permute(A, order)\n"
        ),
        "meshgrid" => Some(
            "MESHGRID  2-D and 3-D grids for plotting and evaluations.\n\n\
             Syntax:\n\
               [X, Y] = meshgrid(x, y)\n"
        ),
        "ndgrid" => Some(
            "NDGRID  Rectangular grid in N-D space.\n\n\
             Syntax:\n\
               [X1, X2, ...] = ndgrid(x1, x2, ...)\n"
        ),
        "cellfun" => Some("CELLFUN  Apply function to each cell in cell array.\n\nSyntax:\n  B = cellfun(func, C)\n  B = cellfun(func, C, \"UniformOutput\", false)\n"),
        "arrayfun" => Some("ARRAYFUN  Apply function to each element of array.\n\nSyntax:\n  B = arrayfun(func, A)\n"),
        "structfun" => Some("STRUCTFUN  Apply function to each field of scalar structure.\n\nSyntax:\n  B = structfun(func, S)\n"),
        "sprintf" => Some("SPRINTF  Format data into string.\n\nSyntax:\n  str = sprintf(formatSpec, A1, ...)\n"),
        "fprintf" => Some("FPRINTF  Write formatted data to file or screen.\n\nSyntax:\n  fprintf(formatSpec, A1, ...)\n"),
        "disp" => Some("DISP  Display value of variable.\n\nSyntax:\n  disp(X)\n"),
        "num2str" => Some("NUM2STR  Convert numbers to character array.\n\nSyntax:\n  s = num2str(A)\n"),
        "mat2str" => Some("MAT2STR  Convert matrix to string formatted for evaluation.\n\nSyntax:\n  s = mat2str(A)\n"),
        "strsplit" => Some("STRSPLIT  Split string or character vector at delimiter.\n\nSyntax:\n  C = strsplit(str, delimiter)\n"),
        "strjoin" => Some("STRJOIN  Join strings in cell array with delimiter.\n\nSyntax:\n  str = strjoin(C, delimiter)\n"),
        "strrep" => Some("STRREP  Find and replace substring.\n\nSyntax:\n  newStr = strrep(str, old, new)\n"),
        "strtrim" => Some("STRTRIM  Remove leading and trailing whitespace.\n\nSyntax:\n  trimmed = strtrim(str)\n"),
        "upper" => Some("UPPER  Convert string to uppercase.\n\nSyntax:\n  B = upper(A)\n"),
        "lower" => Some("LOWER  Convert string to lowercase.\n\nSyntax:\n  B = lower(A)\n"),
        "strcmp" => Some("STRCMP  Compare strings exactly.\n\nSyntax:\n  tf = strcmp(s1, s2)\n"),
        "strcmpi" => Some("STRCMPI  Compare strings case-insensitively.\n\nSyntax:\n  tf = strcmpi(s1, s2)\n"),
        "strncmp" => Some("STRNCMP  Compare first n characters of strings.\n\nSyntax:\n  tf = strncmp(s1, s2, n)\n"),
        "contains" => Some("CONTAINS  Determine if pattern is in string.\n\nSyntax:\n  tf = contains(str, pattern)\n"),
        "startsWith" => Some("STARTSWITH  Determine if string starts with pattern.\n\nSyntax:\n  tf = startsWith(str, pattern)\n"),
        "endsWith" => Some("ENDSWITH  Determine if string ends with pattern.\n\nSyntax:\n  tf = endsWith(str, pattern)\n"),
        "regexp" => Some("REGEXP  Match regular expression.\n\nSyntax:\n  [match, tokens] = regexp(str, expression)\n"),
        "regexprep" => Some("REGEXPREP  Replace text using regular expression.\n\nSyntax:\n  newStr = regexprep(str, expr, rep)\n"),
        "error" => Some("ERROR  Throw error and display message.\n\nSyntax:\n  error(msg)\n  error(id, msg)\n"),
        "warning" => Some("WARNING  Display warning message.\n\nSyntax:\n  warning(msg)\n"),
        "assert" => Some("ASSERT  Assert condition is true.\n\nSyntax:\n  assert(cond, msg)\n"),
        "which" => Some("WHICH  Locate functions and files.\n\nSyntax:\n  path = which(name)\n"),
        "exist" => Some("EXIST  Check existence of variable, script, or function.\n\nSyntax:\n  status = exist(name)\n"),
        "addpath" => Some("ADDPATH  Add folder to search path.\n\nSyntax:\n  addpath(folder)\n"),
        "rmpath" => Some("RMPATH  Remove folder from search path.\n\nSyntax:\n  rmpath(folder)\n"),
        "qr" => Some("QR  Orthogonal-triangular decomposition.\n\nSyntax:\n  [Q, R] = qr(A)\n  [Q, R, P] = qr(A)\n"),
        "double" => Some("DOUBLE  Create double-precision array.\n\nSyntax:\n  Y = double(X)\n"),
        "logical" => Some("LOGICAL  Convert numeric values to logical.\n\nSyntax:\n  Y = logical(X)\n"),
        "cast" => Some("CAST  Cast variable to different data type.\n\nSyntax:\n  B = cast(A, newclass)\n"),
        "class" => Some("CLASS  Name of variable class.\n\nSyntax:\n  c = class(obj)\n"),
        "isa" => Some("ISA  Determine if input is of specified class.\n\nSyntax:\n  tf = isa(obj, className)\n"),
        "isinteger" => Some("ISINTEGER  Determine if input is integer array.\n\nSyntax:\n  tf = isinteger(A)\n"),
        "isfloat" => Some("ISFLOAT  Determine if input is floating-point array.\n\nSyntax:\n  tf = isfloat(A)\n"),
        "isnumeric" => Some("ISNUMERIC  Determine if input is numeric array.\n\nSyntax:\n  tf = isnumeric(A)\n"),
        "issparse" => Some("ISSPARSE  Determine if input is sparse matrix.\n\nSyntax:\n  tf = issparse(S)\n"),
        "intmax" => Some("INTMAX  Largest value of specified integer type.\n\nSyntax:\n  val = intmax(classname)\n"),
        "intmin" => Some("INTMIN  Smallest value of specified integer type.\n\nSyntax:\n  val = intmin(classname)\n"),
        "realmax" => Some("REALMAX  Largest finite floating-point number.\n\nSyntax:\n  val = realmax(classname)\n"),
        "realmin" => Some("REALMIN  Smallest positive normalized floating-point number.\n\nSyntax:\n  val = realmin(classname)\n"),
        "flintmax" => Some("FLINTMAX  Largest consecutive integer in floating-point format.\n\nSyntax:\n  val = flintmax(classname)\n"),
        "eps" => Some("EPS  Floating-point relative accuracy.\n\nSyntax:\n  d = eps(X)\n"),
        "bitand" => Some("BITAND  Bitwise AND on integer arrays.\n\nSyntax:\n  C = bitand(A, B)\n"),
        "bitor" => Some("BITOR  Bitwise OR on integer arrays.\n\nSyntax:\n  C = bitor(A, B)\n"),
        "bitxor" => Some("BITXOR  Bitwise XOR on integer arrays.\n\nSyntax:\n  C = bitxor(A, B)\n"),
        "bitshift" => Some("BITSHIFT  Shift bits specified number of places.\n\nSyntax:\n  C = bitshift(A, k)\n"),
        "nnz" => Some("NNZ  Number of nonzero matrix elements.\n\nSyntax:\n  k = nnz(S)\n"),
        "nonzeros" => Some("NONZEROS  Nonzero matrix elements.\n\nSyntax:\n  v = nonzeros(S)\n"),
        "spones" => Some("SPONES  Replace nonzero sparse elements with ones.\n\nSyntax:\n  R = spones(S)\n"),
        "ipermute" => Some("IPERMUTE  Inverse permute array dimensions.\n\nSyntax:\n  A = ipermute(B, order)\n"),
        "squeeze" => Some("SQUEEZE  Remove singleton dimensions.\n\nSyntax:\n  B = squeeze(A)\n"),
        "circshift" => Some("CIRCSHIFT  Shift array circularly.\n\nSyntax:\n  B = circshift(A, k)\n"),
        "reshape" => Some("RESHAPE  Reshape array.\n\nSyntax:\n  B = reshape(A, sz)\n"),
        "abs" => Some(
            "ABS  Absolute value and complex magnitude.\n\n\
             Syntax:\n\
               Y = abs(X)\n\n\
             Description:\n\
               Returns the absolute value of real elements or complex magnitude sqrt(re^2 + im^2).\n"
        ),
        "acos" => Some(
            "ACOS  Inverse cosine.\n\n\
             Syntax:\n\
               Y = acos(X)\n\n\
             Description:\n\
               Computes the inverse cosine (in radians) for each element of X.\n"
        ),
        "all" => Some(
            "ALL  Determine if all array elements are nonzero or true.\n\n\
             Syntax:\n\
               B = all(A)\n\
               B = all(A, dim)\n\n\
             Description:\n\
               Tests along dimension dim whether all elements are nonzero or logical true.\n"
        ),
        "angle" => Some(
            "ANGLE  Phase angle of complex number.\n\n\
             Syntax:\n\
               P = angle(Z)\n\n\
             Description:\n\
               Returns the phase angle in radians in the interval [-pi, pi] for each element of Z.\n"
        ),
        "any" => Some(
            "ANY  Determine if any array elements are nonzero or true.\n\n\
             Syntax:\n\
               B = any(A)\n\
               B = any(A, dim)\n\n\
             Description:\n\
               Tests along dimension dim whether any element is nonzero or logical true.\n"
        ),
        "asin" => Some(
            "ASIN  Inverse sine.\n\n\
             Syntax:\n\
               Y = asin(X)\n\n\
             Description:\n\
               Computes the inverse sine (in radians) for each element of X.\n"
        ),
        "atan" => Some(
            "ATAN  Inverse tangent.\n\n\
             Syntax:\n\
               Y = atan(X)\n\n\
             Description:\n\
               Computes the inverse tangent (in radians) for each element of X.\n"
        ),
        "atan2" => Some(
            "ATAN2  Four-quadrant inverse tangent.\n\n\
             Syntax:\n\
               P = atan2(Y, X)\n\n\
             Description:\n\
               Computes four-quadrant inverse tangent of Y and X such that -pi <= P <= pi.\n"
        ),
        "ceil" => Some(
            "CEIL  Round toward positive infinity.\n\n\
             Syntax:\n\
               Y = ceil(X)\n\n\
             Description:\n\
               Rounds each element of X to the nearest integer greater than or equal to that element.\n"
        ),
        "cell" => Some(
            "CELL  Create cell array.\n\n\
             Syntax:\n\
               C = cell(n)\n\
               C = cell(sz1, ..., szN)\n\n\
             Description:\n\
               Constructs an empty cell array of the specified dimensions with each element initialized to [].\n"
        ),
        "char" => Some(
            "CHAR  Convert to character array.\n\n\
             Syntax:\n\
               S = char(X)\n\n\
             Description:\n\
               Converts numeric codes, strings, or cell arrays of character vectors to a char array.\n"
        ),
        "chol" => Some(
            "CHOL  Cholesky factorization.\n\n\
             Syntax:\n\
               R = chol(A)\n\n\
             Description:\n\
               Computes upper triangular R such that R'*R = A for symmetric positive definite matrix A.\n"
        ),
        "conj" => Some(
            "CONJ  Complex conjugate.\n\n\
             Syntax:\n\
               Y = conj(X)\n\n\
             Description:\n\
               Computes the complex conjugate of each element of X (re - 1i*im).\n"
        ),
        "conv" => Some(
            "CONV  Convolution and polynomial multiplication.\n\n\
             Syntax:\n\
               C = conv(A, B)\n\
               C = conv(A, B, shape)\n\n\
             Description:\n\
               Computes 1D discrete convolution of vectors A and B with full, same, or valid shape.\n"
        ),
        "cos" => Some(
            "COS  Cosine of argument in radians.\n\n\
             Syntax:\n\
               Y = cos(X)\n\n\
             Description:\n\
               Computes cosine of each element of X in radians.\n"
        ),
        "cumprod" => Some(
            "CUMPROD  Cumulative product.\n\n\
             Syntax:\n\
               B = cumprod(A)\n\
               B = cumprod(A, dim)\n\n\
             Description:\n\
               Computes the cumulative product of elements along dimension dim.\n"
        ),
        "cumsum" => Some(
            "CUMSUM  Cumulative sum.\n\n\
             Syntax:\n\
               B = cumsum(A)\n\
               B = cumsum(A, dim)\n\n\
             Description:\n\
               Computes the cumulative sum of elements along dimension dim.\n"
        ),
        "det" => Some(
            "DET  Matrix determinant.\n\n\
             Syntax:\n\
               d = det(A)\n\n\
             Description:\n\
               Computes the determinant of square matrix A using LU decomposition with partial pivoting.\n"
        ),
        "diag" => Some(
            "DIAG  Diagonal matrices and diagonals of matrix.\n\n\
             Syntax:\n\
               D = diag(v)\n\
               v = diag(A)\n\n\
             Description:\n\
               Extracts matrix diagonals or constructs diagonal matrices.\n"
        ),
        "diff" => Some(
            "DIFF  Differences and approximate derivatives.\n\n\
             Syntax:\n\
               Y = diff(X)\n\
               Y = diff(X, n, dim)\n\n\
             Description:\n\
               Computes differences between adjacent elements along dimension dim.\n"
        ),
        "exp" => Some(
            "EXP  Exponential.\n\n\
             Syntax:\n\
               Y = exp(X)\n\n\
             Description:\n\
               Computes the natural exponential e^x for each element of X.\n"
        ),
        "eye" => Some(
            "EYE  Identity matrix.\n\n\
             Syntax:\n\
               I = eye(n)\n\
               I = eye(m, n)\n\n\
             Description:\n\
               Constructs an m-by-n matrix with ones on the main diagonal and zeros elsewhere.\n"
        ),
        "false" => Some(
            "FALSE  Logical false array.\n\n\
             Syntax:\n\
               L = false(n)\n\
               L = false(sz1, ..., szN)\n\n\
             Description:\n\
               Creates a logical array with all elements set to logical 0 (false).\n"
        ),
        "fftshift" => Some(
            "FFTSHIFT  Shift zero-frequency component to center of spectrum.\n\n\
             Syntax:\n\
               Y = fftshift(X)\n\
               Y = fftshift(X, dim)\n\n\
             Description:\n\
               Rearranges outputs of fft, fft2, or fftn by moving zero-frequency to the center.\n"
        ),
        "fieldnames" => Some(
            "FIELDNAMES  Field names of structure array.\n\n\
             Syntax:\n\
               names = fieldnames(s)\n\n\
             Description:\n\
               Returns a cell array of character vectors containing the field names of structure s.\n"
        ),
        "find" => Some(
            "FIND  Find indices and values of nonzero elements.\n\n\
             Syntax:\n\
               k = find(X)\n\
               k = find(X, n, direction)\n\n\
             Description:\n\
               Returns linear indices of nonzero elements, with optional count limit and 'first'/'last' direction.\n"
        ),
        "fix" => Some(
            "FIX  Round toward zero.\n\n\
             Syntax:\n\
               Y = fix(X)\n\n\
             Description:\n\
               Rounds each element of X toward zero to the nearest integer.\n"
        ),
        "fliplr" => Some(
            "FLIPLR  Flip array left to right.\n\n\
             Syntax:\n\
               B = fliplr(A)\n\n\
             Description:\n\
               Reverses the order of columns of matrix A.\n"
        ),
        "flipud" => Some(
            "FLIPUD  Flip array up to down.\n\n\
             Syntax:\n\
               B = flipud(A)\n\n\
             Description:\n\
               Reverses the order of rows of matrix A.\n"
        ),
        "floor" => Some(
            "FLOOR  Round toward negative infinity.\n\n\
             Syntax:\n\
               Y = floor(X)\n\n\
             Description:\n\
               Rounds each element of X to the nearest integer less than or equal to that element.\n"
        ),
        "help" => Some(
            "HELP  Display help documentation for builtin or M-file.\n\n\
             Syntax:\n\
               help name\n\n\
             Description:\n\
               Displays interactive help documentation for the specified function or topic.\n"
        ),
        "horzcat" => Some(
            "HORZCAT  Horizontal concatenation.\n\n\
             Syntax:\n\
               C = horzcat(A1, A2, ...)\n\n\
             Description:\n\
               Concatenates arrays horizontally along dimension 2. Equivalent to [A1, A2, ...].\n"
        ),
        "imag" => Some(
            "IMAG  Imaginary part of complex number.\n\n\
             Syntax:\n\
               Y = imag(X)\n\n\
             Description:\n\
               Returns the imaginary part of each element of X.\n"
        ),
        "inv" => Some(
            "INV  Matrix inverse.\n\n\
             Syntax:\n\
               Y = inv(X)\n\n\
             Description:\n\
               Computes the matrix inverse X^(-1) of square nonsingular matrix X using LU decomposition with partial pivoting.\n"
        ),
        "isempty" => Some(
            "ISEMPTY  Determine whether array is empty.\n\n\
             Syntax:\n\
               tf = isempty(A)\n\n\
             Description:\n\
               Returns true if array A has at least one dimension of size 0.\n"
        ),
        "isfield" => Some(
            "ISFIELD  Determine whether input is structure array field.\n\n\
             Syntax:\n\
               tf = isfield(s, field)\n\n\
             Description:\n\
               Returns true if field is the name of a field in structure array s.\n"
        ),
        "isfinite" => Some(
            "ISFINITE  Array elements that are finite.\n\n\
             Syntax:\n\
               tf = isfinite(A)\n\n\
             Description:\n\
               Returns true where elements of A are not NaN, Inf, or -Inf.\n"
        ),
        "isinf" => Some(
            "ISINF  Array elements that are infinite.\n\n\
             Syntax:\n\
               tf = isinf(A)\n\n\
             Description:\n\
               Returns true where elements of A are Inf or -Inf.\n"
        ),
        "ismatrix" => Some(
            "ISMATRIX  Determine whether input is matrix.\n\n\
             Syntax:\n\
               tf = ismatrix(A)\n\n\
             Description:\n\
               Returns true if size(A) has exactly 2 dimensions.\n"
        ),
        "isnan" => Some(
            "ISNAN  Array elements that are NaN.\n\n\
             Syntax:\n\
               tf = isnan(A)\n\n\
             Description:\n\
               Returns true where elements of A are IEEE 754 Not-a-Number (NaN).\n"
        ),
        "isscalar" => Some(
            "ISSCALAR  Determine whether input is scalar.\n\n\
             Syntax:\n\
               tf = isscalar(A)\n\n\
             Description:\n\
               Returns true if size(A) is [1, 1].\n"
        ),
        "isvector" => Some(
            "ISVECTOR  Determine whether input is vector.\n\n\
             Syntax:\n\
               tf = isvector(A)\n\n\
             Description:\n\
               Returns true if size(A) is 1-by-N or N-by-1.\n"
        ),
        "length" => Some(
            "LENGTH  Length of largest array dimension.\n\n\
             Syntax:\n\
               n = length(X)\n\n\
             Description:\n\
               Returns max(size(X)) for array X, or 0 if X is empty.\n"
        ),
        "linspace" => Some(
            "LINSPACE  Linearly spaced vector.\n\n\
             Syntax:\n\
               y = linspace(x1, x2)\n\
               y = linspace(x1, x2, n)\n\n\
             Description:\n\
               Generates a row vector of n equally spaced points between x1 and x2.\n"
        ),
        "load" => Some(
            "LOAD  Load variables from MAT-file into workspace.\n\n\
             Syntax:\n\
               load(filename)\n\n\
             Description:\n\
               Loads variables from Level 5 or v7 MAT-file into the caller workspace.\n"
        ),
        "log" => Some(
            "LOG  Natural logarithm.\n\n\
             Syntax:\n\
               Y = log(X)\n\n\
             Description:\n\
               Computes natural logarithm ln(x) elementwise for each element of X.\n"
        ),
        "log10" => Some(
            "LOG10  Common logarithm (base 10).\n\n\
             Syntax:\n\
               Y = log10(X)\n\n\
             Description:\n\
               Computes base-10 logarithm elementwise for each element of X.\n"
        ),
        "log2" => Some(
            "LOG2  Base 2 logarithm.\n\n\
             Syntax:\n\
               Y = log2(X)\n\n\
             Description:\n\
               Computes base-2 logarithm elementwise for each element of X.\n"
        ),
        "logspace" => Some(
            "LOGSPACE  Logarithmically spaced vector.\n\n\
             Syntax:\n\
               y = logspace(a, b)\n\
               y = logspace(a, b, n)\n\n\
             Description:\n\
               Generates a row vector of n logarithmically spaced points between 10^a and 10^b.\n"
        ),
        "lu" => Some(
            "LU  LU matrix factorization.\n\n\
             Syntax:\n\
               [L, U, P] = lu(A)\n\n\
             Description:\n\
               Computes permutation matrix P, lower triangular L, and upper triangular U such that P*A = L*U.\n"
        ),
        "magic" => Some(
            "MAGIC  Magic square.\n\n\
             Syntax:\n\
               M = magic(n)\n\n\
             Description:\n\
               Constructs an n-by-n magic square with sum of rows, columns, and diagonals equal to n*(n^2+1)/2.\n"
        ),
        "max" => Some(
            "MAX  Maximum elements of array.\n\n\
             Syntax:\n\
               M = max(A)\n\
               M = max(A, [], dim)\n\
               [M, I] = max(A)\n\n\
             Description:\n\
               Finds largest elements along dimension dim, with optional linear index output.\n"
        ),
        "mean" => Some(
            "MEAN  Average or mean value of array.\n\n\
             Syntax:\n\
               M = mean(A)\n\
               M = mean(A, dim)\n\n\
             Description:\n\
               Computes the arithmetic mean along dimension dim.\n"
        ),
        "min" => Some(
            "MIN  Minimum elements of array.\n\n\
             Syntax:\n\
               M = min(A)\n\
               M = min(A, [], dim)\n\
               [M, I] = min(A)\n\n\
             Description:\n\
               Finds smallest elements along dimension dim, with optional linear index output.\n"
        ),
        "mldivide" => Some(
            "MLDIVIDE  Matrix left divide or linear system solve.\n\n\
             Syntax:\n\
               x = mldivide(A, B)\n\
               x = A \\ B\n\n\
             Description:\n\
               Solves linear system A*x = B using LU factorization with partial pivoting.\n"
        ),
        "mod" => Some(
            "MOD  Modulus after division.\n\n\
             Syntax:\n\
               M = mod(X, Y)\n\n\
             Description:\n\
               Computes modulus remainder after division: X - floor(X./Y).*Y.\n"
        ),
        "mrdivide" => Some(
            "MRDIVIDE  Matrix right divide.\n\n\
             Syntax:\n\
               x = mrdivide(B, A)\n\
               x = B / A\n\n\
             Description:\n\
               Solves linear system x*A = B. Equivalent to (A' \\ B')'.\n"
        ),
        "ndims" => Some(
            "NDIMS  Number of array dimensions.\n\n\
             Syntax:\n\
               n = ndims(A)\n\n\
             Description:\n\
               Returns the number of dimensions of array A (minimum 2).\n"
        ),
        "norm" => Some(
            "NORM  Vector and matrix norms.\n\n\
             Syntax:\n\
               n = norm(A)\n\
               n = norm(A, p)\n\n\
             Description:\n\
               Computes 1-norm, 2-norm, Frobenius norm, or infinity norm.\n"
        ),
        "numel" => Some(
            "NUMEL  Number of elements in array.\n\n\
             Syntax:\n\
               n = numel(A)\n\n\
             Description:\n\
               Returns the total number of elements in array A: prod(size(A)).\n"
        ),
        "ones" => Some(
            "ONES  Array of all ones.\n\n\
             Syntax:\n\
               Y = ones(n)\n\
               Y = ones(sz1, ..., szN)\n\n\
             Description:\n\
               Constructs an array of specified dimensions with all elements equal to 1.0.\n"
        ),
        "prod" => Some(
            "PROD  Product of array elements.\n\n\
             Syntax:\n\
               B = prod(A)\n\
               B = prod(A, dim)\n\n\
             Description:\n\
               Computes product of array elements along dimension dim.\n"
        ),
        "rand" => Some(
            "RAND  Uniformly distributed random numbers.\n\n\
             Syntax:\n\
               R = rand(n)\n\
               R = rand(sz1, ..., szN)\n\n\
             Description:\n\
               Generates array of pseudorandom values uniformly distributed on interval (0, 1).\n"
        ),
        "randn" => Some(
            "RANDN  Normally distributed random numbers.\n\n\
             Syntax:\n\
               R = randn(n)\n\
               R = randn(sz1, ..., szN)\n\n\
             Description:\n\
               Generates array of pseudorandom numbers from standard normal distribution N(0, 1).\n"
        ),
        "real" => Some(
            "REAL  Real part of complex number.\n\n\
             Syntax:\n\
               Y = real(X)\n\n\
             Description:\n\
               Returns the real component of each element of X.\n"
        ),
        "rem" => Some(
            "REM  Remainder after division.\n\n\
             Syntax:\n\
               R = rem(X, Y)\n\n\
             Description:\n\
               Computes remainder after division: X - fix(X./Y).*Y.\n"
        ),
        "repmat" => Some(
            "REPMAT  Repeat copies of array.\n\n\
             Syntax:\n\
               B = repmat(A, n)\n\
               B = repmat(A, sz1, ..., szN)\n\n\
             Description:\n\
               Tiles array A across the specified dimensions.\n"
        ),
        "rot90" => Some(
            "ROT90  Rotate array 90 degrees.\n\n\
             Syntax:\n\
               B = rot90(A)\n\
               B = rot90(A, k)\n\n\
             Description:\n\
               Rotates array counterclockwise by 90*k degrees in the first two dimensions.\n"
        ),
        "round" => Some(
            "ROUND  Round to nearest integer.\n\n\
             Syntax:\n\
               Y = round(X)\n\n\
             Description:\n\
               Rounds each element of X to the nearest integer.\n"
        ),
        "save" => Some(
            "SAVE  Save workspace variables to MAT-file.\n\n\
             Syntax:\n\
               save(filename, 'var1', ...)\n\n\
             Description:\n\
               Serializes workspace variables into a Level 5 MAT-file with optional compression.\n"
        ),
        "sign" => Some(
            "SIGN  Signum function.\n\n\
             Syntax:\n\
               Y = sign(X)\n\n\
             Description:\n\
               Returns 1 for positive numbers, -1 for negative numbers, and 0 for zero.\n"
        ),
        "sin" => Some(
            "SIN  Sine of argument in radians.\n\n\
             Syntax:\n\
               Y = sin(X)\n\n\
             Description:\n\
               Computes sine of each element of X in radians.\n"
        ),
        "size" => Some(
            "SIZE  Array dimensions.\n\n\
             Syntax:\n\
               sz = size(A)\n\
               [m, n] = size(A)\n\
               d = size(A, dim)\n\n\
             Description:\n\
               Returns the sizes of dimensions of array A.\n"
        ),
        "sort" => Some(
            "SORT  Sort array elements.\n\n\
             Syntax:\n\
               B = sort(A)\n\
               [B, I] = sort(A, dim, direction)\n\n\
             Description:\n\
               Sorts elements along dimension dim in 'ascend' or 'descend' order.\n"
        ),
        "sqrt" => Some(
            "SQRT  Square root.\n\n\
             Syntax:\n\
               Y = sqrt(X)\n\n\
             Description:\n\
               Computes square root elementwise for each element of X.\n"
        ),
        "std" => Some(
            "STD  Standard deviation.\n\n\
             Syntax:\n\
               s = std(A)\n\
               s = std(A, w, dim)\n\n\
             Description:\n\
               Computes standard deviation along dimension dim with normalization weight w (default 0 for N-1).\n"
        ),
        "string" => Some(
            "STRING  Convert to string array.\n\n\
             Syntax:\n\
               S = string(X)\n\n\
             Description:\n\
               Converts numeric, character, or cell array to string array representation.\n"
        ),
        "strlength" => Some(
            "STRLENGTH  Length of string elements.\n\n\
             Syntax:\n\
               L = strlength(S)\n\n\
             Description:\n\
               Returns the number of characters in each string or character vector element.\n"
        ),
        "struct" => Some(
            "STRUCT  Create structure array.\n\n\
             Syntax:\n\
               s = struct(field1, val1, ...)\n\n\
             Description:\n\
               Constructs structure array with specified field names and field values.\n"
        ),
        "sum" => Some(
            "SUM  Sum of array elements.\n\n\
             Syntax:\n\
               S = sum(A)\n\
               S = sum(A, dim)\n\n\
             Description:\n\
               Computes the arithmetic sum of elements along dimension dim.\n"
        ),
        "tan" => Some(
            "TAN  Tangent of argument in radians.\n\n\
             Syntax:\n\
               Y = tan(X)\n\n\
             Description:\n\
               Computes tangent of each element of X in radians.\n"
        ),
        "trace" => Some(
            "TRACE  Sum of diagonal elements of matrix.\n\n\
             Syntax:\n\
               t = trace(A)\n\n\
             Description:\n\
               Computes the trace of square matrix A: sum(diag(A)).\n"
        ),
        "true" => Some(
            "TRUE  Logical true array.\n\n\
             Syntax:\n\
               L = true(n)\n\
               L = true(sz1, ..., szN)\n\n\
             Description:\n\
               Creates a logical array with all elements set to logical 1 (true).\n"
        ),
        "unique" => Some(
            "UNIQUE  Unique values in array.\n\n\
             Syntax:\n\
               C = unique(A)\n\
               [C, ia, ic] = unique(A)\n\n\
             Description:\n\
               Returns sorted unique values of array A with index vectors.\n"
        ),
        "var" => Some(
            "VAR  Variance.\n\n\
             Syntax:\n\
               v = var(A)\n\
               v = var(A, w, dim)\n\n\
             Description:\n\
               Computes variance along dimension dim with normalization weight w (default 0 for N-1).\n"
        ),
        "vertcat" => Some(
            "VERTCAT  Vertical concatenation.\n\n\
             Syntax:\n\
               C = vertcat(A1, A2, ...)\n\n\
             Description:\n\
               Concatenates arrays vertically along dimension 1. Equivalent to [A1; A2; ...].\n"
        ),
        "zeros" => Some(
            "ZEROS  Array of all zeros.\n\n\
             Syntax:\n\
               Y = zeros(n)\n\
               Y = zeros(sz1, ..., szN)\n\n\
             Description:\n\
               Constructs an array of specified dimensions with all elements equal to 0.0.\n"
        ),
        _ => None,
    }
}

/// Builtin `help` function: `help('name')` or `help`
pub fn help_fn(args: &[Value], nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() {
        let general = "RustLab Help System\n\n\
                       For help on a specific function, type: help('function_name')\n\n\
                       Linear Algebra: eig, svd, pinv, rank, cond, null, orth, expm, kron, qr, lu, chol, inv, det\n\
                       Transforms: fft, ifft, fft2, ifft2, fftshift, conv\n\
                       Generators: zeros, ones, eye, rand, randn, linspace, logspace, magic\n\
                       Inspectors: size, length, numel, ndims, isempty, isscalar, ismatrix\n";
        println!("{general}");
        if nargout > 0 {
            return Ok(vec![Value::char_vector(general)]);
        }
        return Ok(Vec::new());
    }

    let target_name = match &args[0] {
        Value::Char(c) => c.as_slice().iter().collect::<String>(),
        Value::String(sa) => sa.as_slice().first().cloned().unwrap_or_default(),
        _ => {
            return Err(user_err(
                "MATLAB:help:invalidArg",
                "help argument must be a function name string or char.",
            ))
        }
    };

    let text = if let Some(doc) = get_help_text(&target_name) {
        doc.to_string()
    } else {
        format!("{target_name} not found in RustLab documentation.\n")
    };

    println!("{text}");
    if nargout > 0 {
        Ok(vec![Value::char_vector(&text)])
    } else {
        Ok(Vec::new())
    }
}
