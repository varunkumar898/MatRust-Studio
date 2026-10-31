# REA GUI Investigation Report: MATLAB Desktop Target & Windowing Architecture

**Date:** 2026-10-30  
**Host Platform:** Windows 11 ARM64 (`aarch64-pc-windows-gnullvm`)  
**MCP Server:** `rea` (116 available reverse-engineering analysis tools)  
**Investigation Target:** `C:\Program Files\MATLAB\R2025a\bin\win64\MATLABWindow.exe`  
**Target Digest:** SHA-256 `69e2233fce9c243849e642a03af7c2dd898a9fe7e9859b26e502ea9651c8c0f9`, 798,304 bytes  

---

## 1. Executive Summary & Evidence Assessment

Per the Master Implementation Prompt (Phase One), this investigation deployed the **REA MCP server** to examine local host MATLAB installation artifacts and observable implementation characteristics without disassembling or decompiling protected proprietary code.

### Evidence Assessment Summary:

| Item | Result | Evidence | Limitation | Status |
| :--- | :--- | :--- | :--- | :--- |
| **REA MCP Availability** | Functional; 116 tools exposed in server schema | Direct tool discovery in MCP environment (`open_binary`, `inspect_pe_imports`, `inspect_pe_resources`, etc.) | Ghidra native decompilation provider unsupported on Windows ARM64 host | **VERIFIED** |
| **MATLAB Target Availability** | Located `R2025a` folder; `matlab.exe` absent; `MATLABWindow.exe` present | `C:\Program Files\MATLAB\R2025a\VersionInfo.xml` (v25.1.0.2973910 Update 1, Jul 03 2025); `bin\win64\MATLABWindow.exe` | Main MATLAB computational core (`matlab.exe`) not installed; only the CEF desktop shell wrapper exists | **VERIFIED** |
| **GUI-Related Imports** | Direct imports of Chromium Embedded Framework (CEF) and native IPC | REA `inspect_pe_imports` output: `libcef.dll`, `cefclientlib.dll`, `cefcommon.dll`, `matlabwindowlib.dll`, `libprotobuf3.dll`, `USER32.dll`, `SHELL32.dll` | PE imports reveal windowing container technology, but do not specify visual styling, dock layout, or color palette | **VERIFIED** |
| **Windowing Framework** | CEF / Webview hybrid shell with C++/Protobuf IPC bridge | APIs: `cef_browser_host_create_browser`, `cef_v8context_*`, `ClientApp`, `SetProcessDpiAwarenessContext`, `SetCurrentProcessExplicitAppUserModelID` | Exact internal JS/HTML UI bundles remain unobserved | **PARTIALLY VERIFIED** |
| **Runtime GUI Inspection** | Live UI inspection blocked | Attempted `observe_native_ui` / `capture_native_ui_scenario` | Cannot attach to an unlaunched/unlicensed commercial runtime; no live process | **BLOCKED** |
| **Desktop Layout Compatibility** | Independent clean-room reproduction required | MathWorks public documentation and user interface standards | PE inspection cannot generate or validate pixel-accurate UI layouts | **HYPOTHESIS** |

---

## 2. Observable MATLAB GUI Implementation Characteristics

Analysis of `MATLABWindow.exe` via REA `inspect_pe_imports` yielded concrete technical findings regarding MathWorks' desktop architecture in release R2025a:

1. **Chromium Embedded Framework (CEF) Container:**
   - The primary imported library is `libcef.dll` (97 imported functions) and associated support modules `cefclientlib.dll` and `cefcommon.dll`.
   - Core CEF functions include `cef_initialize`, `cef_browser_host_create_browser`, `cef_execute_process`, `cef_run_message_loop`, and V8 JavaScript engine integration (`cef_v8value_create_*`, `cef_v8context_*`).
   - This confirms that modern MATLAB desktop applications use a webview/HTML5-based frontend rendered inside a specialized native CEF container.

2. **Native Message Bridge & Protocol Buffers:**
   - The shell imports `libprotobuf3.dll` (46 functions), including `google::protobuf::internal::WireFormatLite`, `MessageLite`, and `TcParser`.
   - Native-to-UI communication is conducted via structured Protocol Buffer serialization between the host backend and the frontend renderer.

3. **High-DPI and Windows Shell Integration:**
   - From `USER32.dll`: `SetProcessDpiAwarenessContext` ensures crisp rendering across high-DPI displays.
   - From `SHELL32.dll`: `SetCurrentProcessExplicitAppUserModelID` manages taskbar grouping, window icon associations, and OS-level window management.

4. **Multi-Threaded Message Loop:**
   - From `matlabwindowlib.dll`: Imports `MainMessageLoopMultithreadedWin` and `MainMessageLoopStd`, demonstrating that UI event dispatching and background tasks operate on separate thread boundaries to prevent interface hangs.

---

## 3. Clean-Room Architectural Boundaries & Legal Compliance

1. **No Proprietary Decompilation:**
   - In accordance with the RustLab Clean-Room Protocol (Master Prompt Section 0 & 3), no MathWorks proprietary binaries, obfuscated P-code, or licensing algorithms were disassembled or reverse engineered.
   - REA analysis was strictly confined to PE header and import directory inspection (`inspect_pe_imports`).

2. **Independence from Heavy Web/CEF Runtimes:**
   - While MATLAB R2025a utilizes Chromium Embedded Framework (~150 MB of DLLs and heavy multi-process architecture), RustLab prioritizes:
     - Standalone native binary deployment (zero external runtime DLL dependencies).
     - Minimal memory footprint and immediate startup (< 100 ms vs multi-second CEF boot).
     - Native GPU acceleration and lightweight rendering.

---

## 4. Synthesis for RustLab Native Desktop GUI

The investigation yields three core design principles for RustLab's GUI:

1. **Separation of Concerns:** Maintain an explicit IPC/channel boundary between the rendering thread and the numerical computation worker (matching the multi-threaded pattern observed in `matlabwindowlib.dll`).
2. **Standard Scientific Computing Layout:** Faithfully reproduce the canonical 4-pane desktop layout:
   - **Current Folder** (tree/table of project files)
   - **Editor** (multi-tab script editing with syntax awareness)
   - **Workspace** (tabular variable inspector with data types, shapes, and bytes)
   - **Command Window** (interactive REPL console with command history and caret diagnostics)
3. **Rust Framework Selection:**
   - Rather than embedding a multi-hundred-megabyte CEF/Chromium webview, RustLab should leverage **`egui` / `eframe`** (immediate mode native Rust GUI) or **`Slint`**, delivering a responsive, self-contained desktop experience.
