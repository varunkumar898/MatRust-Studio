# Reverse Engineering Notes: MEX C API Interoperability Interface

This document specifies the foreign function interface (FFI) and runtime contracts of the MATLAB Executable (MEX) C API, enabling clean-room interoperability for native C, C++, and Rust plugin modules.

---

## 1. MEX Architecture & Lifecycle

A MEX file is a dynamically loaded shared library (`.mexw64` on Windows, `.mexa64` on Linux, `.mexmaca64` on macOS Apple Silicon).

When invoked from the interpreter:
1. The engine checks if a compiled MEX file matching the function name exists in the current search path.
2. The dynamic library is loaded (`dlopen` / `LoadLibraryW`).
3. The entry point symbol `mexFunction` is resolved and executed.
4. Input arguments are passed as pointers to read-only `mxArray` records; output arguments are returned in a caller-allocated pointer array.

---

## 2. Canonical Entry Point

Every MEX shared library must export the following C-ABI function:

```c
#include "mex.h"

void mexFunction(
    int nlhs,                   /* Number of expected output arguments */
    mxArray *plhs[],            /* Array of pointers to output mxArray pointers */
    int nrhs,                   /* Number of input arguments passed */
    const mxArray *prhs[]       /* Array of pointers to input mxArray pointers */
);
```

### Execution Contract:
- `prhs[0] .. prhs[nrhs-1]`: Managed by the caller runtime. Must not be modified or freed by the MEX function.
- `plhs[0] .. plhs[nlhs-1]`: Initially contain uninitialized/null pointers. The MEX function must allocate `mxArray` objects for each requested output.
- If an unhandled exception or `mexErrMsgIdAndTxt` occurs, execution aborts and the engine unwinds to the prompt.

---

## 3. The `mxArray` Memory Model

In memory, `mxArray` is an opaque handle encapsulating:
- **`class_id`**: Enumeration indicating data type (`mxDOUBLE_CLASS`, `mxINT32_CLASS`, etc.).
- **`ndims` & `dims`**: Array of dimension sizes (e.g. $[M, N]$).
- **`data_ptr` (`pr`)**: Contiguous buffer holding real data in column-major order.
- **`imag_ptr` (`pi`)**: Contiguous buffer holding imaginary data (in separate-complex mode).
- **`flags`**: Bitmask encoding `is_complex`, `is_logical`, `is_global`.

### Complex Data Representations
- **Separate Complex (Legacy / R2017b and earlier)**:
  `pr` points to array of doubles; `pi` points to a separate array of doubles.
- **Interleaved Complex (Modern / R2018a+)**:
  `pr` points to contiguous pairs of `[real0, imag0, real1, imag1, ...]`.
  `rustlab` will support both access patterns via compatibility shims.

---

## 4. Core C-ABI Export API Functions

To allow third-party MEX binaries to link against `rustlab`, the engine provides C exports with matching symbol names:

### 4.1 Array Creation
```c
/* Double matrices */
mxArray *mxCreateDoubleMatrix(size_t m, size_t n, mxComplexity ComplexFlag);
mxArray *mxCreateDoubleScalar(double value);

/* Numeric arrays */
mxArray *mxCreateNumericArray(size_t ndim, const size_t *dims, mxClassID classid, mxComplexity ComplexFlag);
mxArray *mxCreateNumericMatrix(size_t m, size_t n, mxClassID classid, mxComplexity ComplexFlag);

/* Strings & Characters */
mxArray *mxCreateString(const char *str);
mxArray *mxCreateCharArray(size_t ndim, const size_t *dims);

/* Containers */
mxArray *mxCreateCellMatrix(size_t m, size_t n);
mxArray *mxCreateCellArray(size_t ndim, const size_t *dims);
mxArray *mxCreateStructMatrix(size_t m, size_t n, int nfields, const char **fieldnames);
```

### 4.2 Array Inspection
```c
size_t mxGetM(const mxArray *pm);                       /* Row count */
size_t mxGetN(const mxArray *pm);                       /* Column count */
size_t mxGetNumberOfDimensions(const mxArray *pm);      /* Dimension count */
const size_t *mxGetDimensions(const mxArray *pm);       /* Pointer to dimensions array */
size_t mxGetNumberOfElements(const mxArray *pm);        /* Total elements (prod of dims) */
mxClassID mxGetClassID(const mxArray *pm);              /* Data class ID */

bool mxIsDouble(const mxArray *pm);
bool mxIsSingle(const mxArray *pm);
bool mxIsComplex(const mxArray *pm);
bool mxIsLogical(const mxArray *pm);
bool mxIsCell(const mxArray *pm);
bool mxIsStruct(const mxArray *pm);
bool mxIsEmpty(const mxArray *pm);
```

### 4.3 Data Buffers & Element Access
```c
/* Real & Imaginary pointers */
double *mxGetPr(const mxArray *pm);
void    mxSetPr(mxArray *pm, double *pr);
double *mxGetPi(const mxArray *pm);
void    mxSetPi(mxArray *pm, double *pi);
void   *mxGetData(const mxArray *pm);

/* Scalar convenience */
double  mxGetScalar(const mxArray *pm);

/* Cell access */
mxArray *mxGetCell(const mxArray *pm, size_t index);
void     mxSetCell(mxArray *pm, size_t index, mxArray *value);

/* Struct access */
mxArray *mxGetField(const mxArray *pm, size_t index, const char *fieldname);
void     mxSetField(mxArray *pm, size_t index, const char *fieldname, mxArray *value);
int      mxGetNumberOfFields(const mxArray *pm);
const char *mxGetFieldNameByNumber(const mxArray *pm, int fieldno);
```

### 4.4 Memory Lifecycle Management
```c
void *mxMalloc(size_t n);
void *mxCalloc(size_t n, size_t size);
void  mxFree(void *ptr);
void  mxDestroyArray(mxArray *pm);
void  mexMakeMemoryPersistent(void *ptr);
void  mexMakeArrayPersistent(mxArray *pm);
```
- **Garbage Collection Policy**: Any buffer allocated via `mxMalloc`/`mxCalloc` during a `mexFunction` call is registered in an ephemeral arena. When `mexFunction` exits, all non-persistent buffers are automatically reclaimed.

### 4.5 Runtime Interaction & Messaging
```c
int  mexPrintf(const char *format, ...);
void mexWarnMsgIdAndTxt(const char *identifier, const char *warning_msg);
void mexErrMsgIdAndTxt(const char *identifier, const char *error_msg);
int  mexCallMATLAB(int nlhs, mxArray *plhs[], int nrhs, mxArray *prhs[], const char *commandName);
int  mexEvalString(const char *command);
```

---

## 5. Rust Clean-Room Implementation Plan for MEX

In `rustlab`, the MEX compatibility layer is implemented in a dedicated crate (`rustlab-mex`):
1. Define a C-compatible `#[repr(C)] pub struct mxArray` backed by an `Arc<Value>`.
2. Implement `extern "C"` shims for `mxGetPr`, `mxCreateDoubleMatrix`, etc.
3. Use `libloading` in the engine to load external `.mex` dynamic libraries and invoke their `mexFunction` entry point.
