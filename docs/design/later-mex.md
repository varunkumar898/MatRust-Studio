# Future Design: MEX C ABI Compatibility Layer (LATER)

> [!NOTE]
> **Status**: LATER (Deferred beyond Phase B core milestones).
> **Clean-Room Rule**: All headers and declarations implemented for this layer must be RustLab's own clean-room headers written from documented, publicly published C API signatures only. Proprietary header files (`mex.h`, `matrix.h` from commercial distributions) must never be copied or bundled into this repository.

---

## 1. Objectives & Clean-Room Boundaries

Milestone B8 (MEX Foreign Function Interface) is planned as an extension to allow RustLab to load external compiled MEX plugins (`.mexw64` / `.mexa64`) or compile C MEX plugins using a local C compiler.

### Clean-Room Rules:
1. **Own Headers**: RustLab provides its own `rustlab_mex.h` and `rustlab_matrix.h` generated from public function signatures.
2. **No Decompiled Code**: No disassembly or reverse-engineering of proprietary `libmex.dll` or `libmx.dll` is permitted.
3. **Linkage**: Emulate standard C calling conventions (`cdecl` on x86, AAPCS64 on ARM64, Microsoft x64 ABI on x86_64).

---

## 2. The MEX C ABI Specification

### 2.1 The Canonical Entry Point
Every MEX dynamic library exports a single canonical entry point:

```c
#include "rustlab_mex.h"

void mexFunction(
    int nlhs,                 /* Number of expected left-hand side (output) arguments */
    mxArray *plhs[],          /* Array of output pointers (allocated by MEX function) */
    int nrhs,                 /* Number of right-hand side (input) arguments */
    const mxArray *prhs[]     /* Array of input pointers (read-only, managed by engine) */
);
```

### 2.2 Calling Convention & Linkage Contract
- **Calling Convention**: Standard C calling convention.
- **Argument Ownership**:
  - `prhs`: Pointers belong to the engine runtime. MEX functions must not free or reassign them.
  - `plhs`: The MEX function allocates `mxArray` objects for each index $0 \le i < \text{nlhs}$. Unassigned outputs remain null.
- **Execution Lifecycle**:
  ```
  Interpreter Prompt -> Engine resolves "my_mex.mexw64"
                     -> LoadLibrary / dlopen
                     -> GetProcAddress("mexFunction")
                     -> Marshall Engine Values -> prhs[]
                     -> Invoke mexFunction(nlhs, plhs, nrhs, prhs)
                     -> Marshall plhs[] -> Engine Values
                     -> Arena Garbage Collection (free ephemeral allocations)
  ```

---

## 3. Core Exported C API Symbols (From Public Specifications)

### 3.1 Data Inspection & Extraction
```c
/* Shape and Dimensions */
size_t mxGetNumberOfDimensions(const mxArray *pm);
const size_t *mxGetDimensions(const mxArray *pm);
size_t mxGetM(const mxArray *pm);
size_t mxGetN(const mxArray *pm);
size_t mxGetNumberOfElements(const mxArray *pm);

/* Class and Type Predicates */
mxClassID mxGetClassID(const mxArray *pm);
bool mxIsDouble(const mxArray *pm);
bool mxIsSingle(const mxArray *pm);
bool mxIsComplex(const mxArray *pm);
bool mxIsLogical(const mxArray *pm);
bool mxIsChar(const mxArray *pm);
bool mxIsCell(const mxArray *pm);
bool mxIsStruct(const mxArray *pm);
bool mxIsEmpty(const mxArray *pm);

/* Buffer Pointers */
double *mxGetPr(const mxArray *pm);
double *mxGetPi(const mxArray *pm);
void   *mxGetData(const mxArray *pm);
double  mxGetScalar(const mxArray *pm);
int     mxGetString(const mxArray *pm, char *buf, size_t buflen);
```

### 3.2 Array Creation & Mutation
```c
mxArray *mxCreateDoubleMatrix(size_t m, size_t n, mxComplexity ComplexFlag);
mxArray *mxCreateDoubleScalar(double value);
mxArray *mxCreateNumericArray(size_t ndim, const size_t *dims, mxClassID classid, mxComplexity ComplexFlag);
mxArray *mxCreateNumericMatrix(size_t m, size_t n, mxClassID classid, mxComplexity ComplexFlag);
mxArray *mxCreateString(const char *str);
mxArray *mxCreateCharArray(size_t ndim, const size_t *dims);
mxArray *mxCreateCellMatrix(size_t m, size_t n);
mxArray *mxCreateStructMatrix(size_t m, size_t n, int nfields, const char **fieldnames);
void     mxSetPr(mxArray *pm, double *pr);
void     mxSetPi(mxArray *pm, double *pi);
```

### 3.3 Engine Interaction & Messaging
```c
int  mexPrintf(const char *format, ...);
void mexWarnMsgIdAndTxt(const char *identifier, const char *warning_msg);
void mexErrMsgIdAndTxt(const char *identifier, const char *error_msg);
int  mexCallMATLAB(int nlhs, mxArray *plhs[], int nrhs, mxArray *prhs[], const char *commandName);
int  mexEvalString(const char *command);
```

### 3.4 Memory Management & Arena Allocation
```c
void *mxMalloc(size_t n);
void *mxCalloc(size_t n, size_t size);
void  mxFree(void *ptr);
void  mxDestroyArray(mxArray *pm);
void  mexMakeMemoryPersistent(void *ptr);
void  mexMakeArrayPersistent(mxArray *pm);
```

---

## 4. Windows Dynamic Linking Architecture

### 4.1 Export Shims
When compiling MEX source code with RustLab's future `mex` command:
- RustLab will supply `rustlab_mex.h` and link against `rustlab_mex.lib` or provide lightweight shim DLLs `libmx.dll` / `libmex.dll` exporting the C symbols above.
