# REA GUI Investigation Call Log

**Date:** 2026-10-30  
**Host Environment:** Windows 11 ARM64 (`aarch64-pc-windows-gnullvm`)  
**MCP Server:** `rea` (Reverse Engineering Analysis MCP)  
**Investigation Purpose:** Investigate local MATLAB desktop installation, executable artifacts, imported GUI windowing libraries, and runtime architecture to inform clean-room GUI architecture design for RustLab.

---

## Verbatim Tool Call Log

| Timestamp (UTC) | Tool / Method | Target File & SHA-256 | Arguments / Parameters | Purpose & Outcome | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| 2026-10-30 15:39:15 | Shell (`Get-ChildItem`) | `C:\Program Files\MATLAB` | Directory scan for installation version | Discovered `C:\Program Files\MATLAB\R2025a` (Version 25.1.0.2973910 Update 1, dated Jul 03 2025). | VERIFIED |
| 2026-10-30 15:39:40 | Shell (`Get-ChildItem`) | `C:\Program Files\MATLAB\R2025a\bin\win64` | Executable search for `matlab.exe` | `matlab.exe` absent. Located `MATLABWindow.exe` (798,304 B) and `matlabwindowhelper.exe` (608,864 B). | VERIFIED |
| 2026-10-30 15:42:28 | REA `inspect_pe_imports` | `C:\Program Files\MATLAB\R2025a\bin\win64\MATLABWindow.exe`<br>`69e2233fce9c243849e642a03af7c2dd898a9fe7e9859b26e502ea9651c8c0f9` | `{"path": "C:\\Program Files\\MATLAB\\R2025a\\bin\\win64\\MATLABWindow.exe", "max_dlls": 256, "max_functions": 8192}` | Inspected PE32+ import directory: 12 DLLs, 198 imported APIs including `libcef.dll`, `cefclientlib.dll`, `matlabwindowlib.dll`, `libprotobuf3.dll`, `USER32.dll` (`SetProcessDpiAwarenessContext`), `SHELL32.dll`. | VERIFIED |
| 2026-10-30 15:43:10 | REA `inspect_pe_resources` | `C:\Program Files\MATLAB\R2025a\bin\win64\MATLABWindow.exe` | `{"path": "C:\\Program Files\\MATLAB\\R2025a\\bin\\win64\\MATLABWindow.exe"}` | Evaluated resource directory for UI manifests, icons, and dialog templates. | PARTIALLY VERIFIED |
| 2026-10-30 15:43:55 | REA `observe_native_ui` | `MATLABWindow.exe` | Runtime GUI observation probe | BLOCKED: Live target process not running; REA runtime observation requires active process attachment. | BLOCKED |
