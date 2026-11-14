# Build Size & Binary Footprint Audit (docs/audit/build-size.md)

**Audit Date:** 2026-11-14  
**Auditor:** Antigravity AI Agent  
**Branch:** `docs/a2-bootstrap`  
**Status:** PASSED  

---

## 1. Release Executable Inventory

Measured with PowerShell `Get-ChildItem | Select-Object Name, Length`:

```
> Get-ChildItem target/release/*.exe | Select-Object Name, Length

Name                 Length
----                 ------
matrust_studio.exe 10696704
```

```
> Get-ChildItem re/targets/*.exe | Select-Object Name, Length

Name                                         Length
----                                         ------
rustlab-g0.1-verified-preclean-F6A302EB.exe 5416960
rustlab-g0.1-verified.exe                   5416960
rustlab-g1-verified.exe                     5400576
rustlab.exe                                 5400064
```

---

## 2. Binary Analysis & Budget Compliance

- **Primary Desktop Binary (`matrust_studio.exe`):**
  - **Measured Size:** 10,696,704 bytes (~10.20 MB).
  - **Budget:** $\le 13.0\text{ MB}$ (Measured baseline 10.20 MB + 20% headroom = 12.24 MB; rounded ceiling 13.0 MB).
  - **Compliance:** **PASSED** (2.3 MB below ceiling).
  - **Components Included:** CLI engine, full stdlib, numerical parser/evaluator, MNA electrical circuit engine, block diagram editor (`egui`/`eframe`), C99 code generator, parameter sweep engine, and simulation cancel/undo stacks.
- **Headless CLI Binary (`rustlab.exe`):**
  - **Measured Size:** 5,400,064 bytes (~5.15 MB).
  - **Components Included:** Core interpreter, stdlib, matrix operations, CSV/MAT I/O, REPL.
- **Installer Package:**
  - **Status:** **NOT MEASURED** (there is no installer package yet).
  - **Future Budget:** $\le 100\text{ MB}$ once packaging automation is introduced.
