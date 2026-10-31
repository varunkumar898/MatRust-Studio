# RustLab Clean-Room Protocol & Reverse Engineering Notes

This directory contains functional specifications, binary layout specifications, interop validation notes, and toolchain logs for the **RustLab** numerical computing project.

---

## 1. Clean-Room Protocol & Role Boundaries

To ensure complete intellectual property integrity and maintain legal independence from proprietary numerical software suites (such as MATLAB®), RustLab adheres to a **Clean-Room Engineering Protocol**.

### Single-Agent Role Separation by Note-Passing
In this development workflow, **a single AI agent performs both Analyst and Implementer activities**. To maintain strict clean-room isolation under this model, separation is enforced **by note-passing only**:

```
+-------------------------------------------------------------+
|                     ANALYST PASS                            |
|  - Inspects file formats, public standards, and open tools. |
|  - Uses binary inspection tools (objdump, python validators)|
|    ONLY on rustlab's own binaries or open formats.          |
|  - Validates interoperability with Python / SciPy / NumPy.  |
|  - Writes functional specifications into docs/re-notes/.    |
|  - RULE: Notes contain ONLY plain English descriptions,     |
|    tables, and formulas. NO pasted disassembly and NO       |
|    decompiled pseudo-code are ever recorded.                |
+-------------------------------------------------------------+
                                |
             (Clean-Room Functional Specifications)
             (docs/re-notes/*.md & specifications)
                                v
+-------------------------------------------------------------+
|                   IMPLEMENTER PASS                          |
|  - Reads ONLY public specs and Analyst English notes.       |
|  - Writes production Rust code (rustlab-math, eval, etc.).  |
|  - Implements algorithms independently from mathematical    |
|    first principles and textbook definitions.               |
|  - NEVER consults proprietary binary disassembly.           |
+-------------------------------------------------------------+
```

### Absolute Rules:
1. **Zero Proprietary Binaries**: No MathWorks binary, P-code, or protection code has ever been disassembled, decompiled, or touched.
2. **Plain English Only**: All findings in `docs/re-notes/` are documented in human-readable plain English, mathematical specifications, or structured JSON/tables. No disassembly dumps or decompiler output are stored.
3. **Open Reference Interop**: Compatibility is verified solely against open-source implementations (`scipy.io`, `numpy`, GNU Octave documentation) and public specification standards.

---

## 2. Host Environment & Toolchain Audit

The host environment is **Windows ARM64** (`aarch64-pc-windows-gnullvm`).

### 2.1 Inspection Toolchain Status
1. **Ghidra MCP Server**:
   - Status: Unreachable / Not configured on host (`ghidra` server is not registered in MCP host configuration).
2. **REA MCP Server**:
   - Status: Available for native binary analysis, but Ghidra tools are not bundled in REA.
3. **Primary Authoritative Toolchain (Verified)**:
   - Tool: `llvm-objdump.exe`
   - Location: `C:\Users\varun\.rustup\toolchains\stable-aarch64-pc-windows-gnullvm\lib\rustlib\aarch64-pc-windows-gnullvm\bin\llvm-objdump.exe`
   - Version: `LLVM version 23.1.1-rust-1.99.1-stable`
   - Target: `coff-arm64` PE/COFF execution binaries.
4. **Secondary Toolchain (Verified)**:
   - Tool: GNU `objdump.exe`
   - Location: `C:\msys64\ucrt64\bin\objdump.exe`
   - Version: `GNU objdump (GNU Binutils) 2.45`
5. **Reference Validation Environment**:
   - Python: 3.14 (ARM64) at `C:\Users\varun\AppData\Local\Programs\Python\Python314-arm64\python.exe`
   - Packages: `scipy` 1.18.1, `numpy` 2.4.1.

---

## 3. Mandatory Call Logging Protocol

Every execution of native disassembly, symbol inspection, and binary analysis must be recorded in [`CALL-LOG.md`](CALL-LOG.md). Each entry must include:
- ISO 8601 Timestamp (UTC)
- Tool Executable & Path
- Target Binary / Object SHA-256
- Technical Purpose
- Summary of Findings / Outcome

---

## 4. Documentation Index

- [`CALL-LOG.md`](CALL-LOG.md): Full audit log of all toolchain executions with target SHA-256 checksums.
- [`gemm-codegen.md`](gemm-codegen.md): ARM64 NEON codegen inspection for GEMM and elementwise vector operations.
- [`mat-v7-spec.md`](mat-v7-spec.md): MAT-File Level 5 / Version 7 binary specification, SciPy interop validation results, and chunk padding analysis.
- [`PLAN-B8.md`](PLAN-B8.md): Inspection plan for using Ghidra on RustLab-generated C code (symbols, imports, stack usage).
- [`mat-format.md`](mat-format.md): Fundamental MAT-file layout reference.
- [`mex-api.md`](mex-api.md): Fundamental MEX C ABI specification.
