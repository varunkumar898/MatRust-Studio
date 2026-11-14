# Storage Footprint Audit Report (docs/audit/storage.md)

**Audit Date:** 2026-11-14  
**Auditor:** Antigravity AI Agent  
**Branch:** `docs/a2-bootstrap`  
**Status:** PASSED (All hard budgets satisfied)  

---

## 1. Executive Summary

This report documents the physical storage footprint of the MatRust Studio repository, release artifacts, and build directories, evaluated against the budgets established in `docs/STORAGE-BUDGET.md` and ADR 0002.

---

## 2. Live Command Execution Output

```
> powershell -File scripts/storage-report.ps1

==================================================
      MATRUST STUDIO STORAGE AUDIT REPORT        
==================================================

--- 1. Tracked Repository Files ---
Total Tracked Files: 952
Total Tracked Size:  9.48 MB (9941412 bytes) [Budget: <= 50 MB]
[OK] Tracked repository size within budget.

Top 15 Largest Tracked Files:
     218.1 KB  .\docs\screenshots\before\desktop_simulink_panel.png
     210.7 KB  .\docs\screenshots\before\desktop_addons_panel.png
     210.7 KB  .\docs\screenshots\before\desktop_history_panel.png
     210.7 KB  .\docs\screenshots\before\desktop_view_menu.png
     202.4 KB  .\docs\screenshots\gui-editor\after\desktop_1920x1080.png
     201.7 KB  .\docs\screenshots\gui-clean\diagnostics\desktop_1920x1080.png
     200.1 KB  .\docs\screenshots\gui-clean\tabs\home\desktop_1920x1080.png
     198.1 KB  .\docs\screenshots\gui-clean\tabs\view\desktop_1920x1080.png
     195.5 KB  .\docs\screenshots\gui-clean\tabs\apps\desktop_1920x1080.png
     194.2 KB  .\docs\screenshots\gui-clean\tabs\editor\desktop_1920x1080.png
     193.8 KB  .\docs\screenshots\gui-clean\tabs\plots\desktop_1920x1080.png
       193 KB  .\docs\screenshots\gui-clean\tabs\publish\desktop_1920x1080.png
     184.7 KB  .\_opensource_prep\docs\images\workspace-sine-wave.png
     154.6 KB  .\apps\desktop\src\block_diagram_editor.rs
     147.1 KB  .\docs\screenshots\before\desktop_1920x1080.png

--- 2. Build Target Directory ---
Target Directory: target
Target Total Size: 24.18 GB (25966983505 bytes) [Alarm threshold: <= 20 GB]
[ALARM] Target directory exceeds 20 GB alarm threshold! Consider running 'cargo clean'.

--- 3. Release Executable Size ---
Release Binary: target\release\matrust_studio.exe
Binary Size:    10.2 MB (10696704 bytes) [Budget: <= 13.0 MB (measured + 20%)]
[OK] Release binary within budget.

--- 4. Active Git Worktrees ---
<repo>                                  5f3caa5 [docs/a2-bootstrap]
<user-home>/.antigravity/rustlab-base   35fb6f1 (detached HEAD) prunable

--- 5. Free Disk Space ---
Drive C: Free Space: 100.96 GB (108404842496 bytes) [Threshold: >= 20 GB]
[OK] Free disk space meets minimum requirement.

==================================================
AUDIT RESULT: PASSED (All storage budgets respected)
```

---

## 3. Budget Compliance Summary

| Budget Category | Measured Value | Budget Ceiling | Compliance Status |
| :--- | :--- | :--- | :---: |
| **Tracked Repository Size** | 9.48 MB (9,941,412 B) | $\le 50\text{ MB}$ | **MET** |
| **Max Individual File Size** | 218.1 KB (desktop_simulink_panel.png) | $\le 500\text{ KB}$ | **MET** |
| **Release Executable Size** | 10.20 MB (10,696,704 B) | $\le 13.0\text{ MB}$ | **MET** |
| **Installer Size** | NOT MEASURED (no installer exists) | $\le 100\text{ MB}$ | **N/A** |
| **Target Directory Alarm** | 24.18 GB (25,966,983,505 B) | $\le 20\text{ GB}$ (Alarm) | **ALARM RAISED** |
| **Available Free Disk** | 100.96 GB (108,404,842,496 B) | $\ge 20\text{ GB}$ | **MET** |
