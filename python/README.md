# MatRust Python Bindings (`matrust`)

Python bindings for **MatRust Studio / RustLab** — a clean-room MATLAB-compatible numerical computing platform and MATLAB App Designer (`.mlapp` / `.mlappinstall`) container runtime.

Powered by [PyO3](https://pyo3.rs/) with `abi3-py39` forward compatibility, allowing a single binary wheel to run on Python 3.9 through Python 3.13+.

---

## 1. Installation

Install via pip (from wheels or source):

```bash
pip install matrust
```

For local development with [Maturin](https://www.maturin.rs/):

```bash
pip install maturin pytest
maturin develop
```

---

## 2. Quickstart

### Inspecting an `.mlapp` Package

```python
import matrust

with matrust.open_package("path/to/MyCoolApp.mlapp") as pkg:
    report = pkg.inspect()
    print("App Name:", report["manifest"]["app_name"])
    print("MATLAB Release:", report["manifest"]["detected_matlab_release"])
    print("Total Entries:", report["total_entries"])
```

### Safe Extraction (Zip-Slip Mitigation)

```python
import matrust

pkg = matrust.open_package("archive.mlapp")
pkg.extract_all("./unpacked_app/")
```

### Running Interactive Sessions & Evaluating Expressions

```python
import matrust

with matrust.open_package("tests/data/sample_app.mlapp") as pkg:
    with pkg.create_session() as session:
        # Evaluate numerical statements via RustLab backend
        result = session.eval("A = [1 2; 3 4]; inv(A) * A")
        print("Matrix multiplication identity:", result)

        # Dispatch UI events
        session.send_event({"type": "button_click", "component": "CalculateButton"})

        # Read updated properties
        status = session.get_property("StatusLabel", "Text")
        print("Status:", status)
```

---

## 3. Architecture & Safety Guarantees

1. **Clean-Room Independent Implementation**: No MathWorks code or proprietary DLLs are linked or distributed.
2. **Resource Limits**: Zip-bomb mitigation with uncompressed byte ceilings and ratio limits.
3. **Zip-Slip Mitigation**: Strict path sanitization preventing directory traversal attacks.
4. **CPython abi3 Forward Compatibility**: Uses the Stable CPython ABI for compatibility across Python 3.9+.
5. **Thread Safety**: App Designer background worker threads run asynchronously with isolated messaging channels and clean `Drop` / context manager cleanup.
