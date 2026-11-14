# MatRust Studio Storage Budget Ledger (docs/STORAGE-BUDGET.md)

This ledger defines and tracks all machine-enforced repository and artifact storage budgets for MatRust Studio. Every measured number derives directly from automated command executions recorded in this session. Anything not measured is explicitly designated as `NOT MEASURED`.

> **Installer note:** There is currently no installer package for MatRust Studio. The installer budget applies only once an installer packaging mechanism is introduced.

---

## 1. Owner Storage Budgets & Current Measured Status

| Metric / Artifact | Budget Ceiling | Measured Value | Status | Measurement Source |
| :--- | :---: | :---: | :---: | :--- |
| **Tracked Repository Size** | $\le 50\text{ MB}$ | **9.48 MB** (9,941,412 B) | **PASS** | `powershell -File scripts/storage-report.ps1` |
| **New Tracked File Limit** | $\le 500\text{ KB}$ | **218.1 KB** (largest file) | **PASS** | `scripts/storage-report.ps1` (top 15 tracked files) |
| **Release Desktop Executable** | $\le 12.84\text{ MB}$ (13.0 MB) | **10.20 MB** (10,696,704 B) | **PASS** | `Get-Item target/release/matrust_studio.exe` |
| **Installer Package** | $\le 100\text{ MB}$ | **NOT MEASURED** | **N/A** | There is no installer |
| **Shared Target Directory Alarm** | $\le 20\text{ GB}$ (Alarm) | **24.18 GB** (25,966,983,505 B) | **ALARM** | `scripts/storage-report.ps1` (cache cleanup advised) |
| **Free Disk Space Floor** | $\ge 20\text{ GB}$ | **100.96 GB** (108,404,842,496 B) | **PASS** | `[System.IO.DriveInfo]::new('C').AvailableFreeSpace` |

---

## 2. Release Executable Measurement

- **Target Build Command:** `cargo build --release -p matrust_studio`
- **Binary Location:** `target/release/matrust_studio.exe`
- **Measured Size:** Exactly `10,696,704` bytes (~10.20 MB).
- **Headroom Ceiling Calculation:** $10{,}696{,}704\text{ B} \times 1.20 = 12{,}836{,}044\text{ B}$ (~12.24 MB; enforced ceiling $\le 13.0\text{ MB}$).

---

## 3. Enforcement & Quality Gates

Storage budgets are automated by `scripts/storage-report.ps1` and `scripts/storage-report.sh`. These checks exit with a non-zero error code if any hard budget threshold is breached:
1. Tracked repository size exceeding $50\text{ MB}$.
2. Any individual tracked file exceeding $500\text{ KB}$.
3. Release executable exceeding $13.0\text{ MB}$.
4. Free filesystem disk space falling below $20\text{ GB}$.

An alarm is emitted when the shared build target directory exceeds $20\text{ GB}$.
