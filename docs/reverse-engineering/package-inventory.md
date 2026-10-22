# Package Inventory and Installer Analysis: MATLAB R2026b Linux Package

## 1. Executive Summary

An exhaustive inspection of `/home/cherry/Downloads/matlab_R2026b_Linux.zip` and the extracted directory `/home/cherry/Downloads/matlab_R2026b_Linux` was performed.

**Primary Finding:**
The provided package is **NOT** a full offline MATLAB product installer. It is specifically the standalone **MathWorks Network License Manager (FLEXnet License Server) installer** for release R2026b. It contains only the MathWorks installer bootstrap framework and the FLEXnet license server daemons (`lmgrd`, MathWorks vendor daemon `MLM`, and licensing utilities).

Neither the MATLAB computational engine (`bin/glnxa64/MATLAB`), nor Simulink, nor any toolboxes, nor the MATLAB Runtime payload are present in this package.

---

## 2. Package Artifacts & Checksums

| File Path | Size (Bytes) | SHA-256 Digest |
| :--- | :--- | :--- |
| `matlab_R2026b_Linux.zip` | 283,371,454 (270.2 MiB) | `79db45365761ead7725982abc275a2f817ca202c0f555600b6618e7ed6639c39` |
| `archives/common/Network_License_Manager262_common.zip` | 4,222,903 (4.03 MiB) | `9ef308a6e7d9cf55379f5c23ea0e201a5c662512c2612cb46c251088531e45e2` |
| `archives/glnxa64/Network_License_Manager262_glnxa64.zip` | 10,230,677 (9.76 MiB) | `c07e1acedc7ba07a0e275247e1811b393571b0614b92c7f815a488e084510818` |

---

## 3. Detailed Structural Evidence

### 3.1 Metadata Inspection
- **`VersionInfo.xml`**:
  ```xml
  <MathWorks_version_info>
    <version>26.2.0.3386108</version>
    <release>R2026b</release>
    <date>Sep 08 2026</date>
    <checksum>1659818251</checksum>
  </MathWorks_version_info>
  ```
- **`archives/common/Network_License_Manager262_common.zip:productdata_Network_License_Manager262_common.xml`**:
  ```xml
  <productData>
    <productBaseCode>LICENSEMANAGER</productBaseCode>
    <productName>Network License Manager</productName>
    <productNumber>0</productNumber>
    <productType>licensedProduct</productType>
    <productVersion>26.2</productVersion>
    <releaseDescription>R2026b</releaseDescription>
  </productData>
  ```
- **`archives/glnxa64/Network_License_Manager262_glnxa64.zip:productdata_Network_License_Manager262_glnxa64.xml`**:
  Contains payloads exclusively for:
  - `3p/flexnet_license_server_glnxa64`
  - `bin_utils_glnxa64`
  - `gcc_runtime_glnxa64`
  - `license_manager_glnxa64`

### 3.2 Contents of `archives/`
In a standard offline MATLAB distribution, `archives/` contains tens of gigabytes of archives such as `MATLAB_common.zip`, `MATLAB_glnxa64.zip`, `Simulink_common.zip`, and individual toolboxes.
In this distribution, `archives/` contains **strictly** the two Network License Manager zip files listed above.

### 3.3 Absence of Executable Payload
Search commands across the installer directory confirm:
- No `bin/glnxa64/MATLAB` (the main computational engine binary)
- No `bin/matlab` startup wrapper
- The binaries present under `bin/glnxa64/` (`MathWorksProductInstaller`, `MATLABWindow`, `matlabconnector`) belong entirely to the Chromium/Qt installer framework used by MathWorks to guide installation and license setup.

---

## 4. Root Cause of Installer Crash (Segmentation Fault)

### 4.1 System & Environment Observation
- **CPU**: Intel Xeon E3-1225 v5 @ 3.30GHz (Skylake microarchitecture, Family 6, Model 94, Stepping 3)
- **OS**: Arch Linux (x86_64, Linux kernel 6.x)
- **Installer Entrypoint**: `/home/cherry/Downloads/matlab_R2026b_Linux/install`

### 4.2 Crash Stack Trace (`coredumpctl`)
```
Signal: 11 (SEGV) si_code: SEGV_MAPERR
Executable: /home/cherry/Downloads/matlab_R2026b_Linux/bin/glnxa64/MathWorksProductInstaller

Stack trace of thread 14499:
#0  0x0000760f272bac98 Ox0853564f26d75632 (libmwinstall_activationwsclientimpl.so + 0x120c98)
#1  0x0000760f272bb727 lc_init (libmwinstall_activationwsclientimpl.so + 0x121727)
#2  0x0000760f12c645a8 n/a (libgnutls.so.30 + 0x645a8)
#3  0x0000760f12c2663f n/a (libgnutls.so.30 + 0x2663f)
#4  0x0000760f2cf481ef n/a (ld-linux-x86-64.so.2 + 0x61ef)
...
#14 0x0000760f2c6945eb dlopen (libc.so.6 + 0x945eb)
```

### 4.3 Defect Mechanism (FLEXlm CPUID Pointer Dereference)
As documented on the official [ArchWiki MATLAB Troubleshooting Guide](https://wiki.archlinux.org/title/MATLAB#Installer_or_MATLAB_crashes_immediately_with_segmentation_fault_on_Intel_CPUs_(R2026+)):
1. In release R2026a and R2026b, the bundled licensing library (`libmwinstall_activationwsclientimpl.so`) executes the x86 `CPUID` instruction (`EAX=0`) to query the CPU vendor string.
2. On Intel CPUs, `CPUID` returns `"GenuineIntel"` in registers `EBX` (`0x756e6547`), `EDX` (`0x49656e69`), and `ECX` (`0x6c65746e`).
3. Due to a defect in `lc_init` / licensing initialization routines, the raw 32-bit register values are erroneously cast and dereferenced as 64-bit memory addresses (e.g. accessing `0x6c65746e`, which is ASCII `"ntel"`).
4. Modern Linux kernels do not map low user addresses by default (`vm.mmap_min_addr`), causing immediate `SEGV_MAPERR` signals upon execution.

### 4.4 Verified Upstream Workaround
The verified upstream shim (`matlab_fix.c` by Stefano Coniglio, linked from ArchWiki):
- Uses an ELF constructor (`__attribute__((constructor))`) to `mmap` anonymous read/write zero-pages at `0x75600000`, `0x49600000`, and `0x6c600000`.
- Installs a graceful `SIGSEGV` recovery handler for downstream offset reads.
- When built via `gcc -shared -fPIC -o ~/matlab_fix.so ~/matlab_fix.c -nostartfiles` and passed via `LD_PRELOAD=/home/cherry/matlab_fix.so ./install -help`, the installer initializes cleanly without crashing.

---

## 5. How to Obtain the Full MATLAB Linux Product

To obtain the complete MATLAB product package for Linux:
1. Log into your authorized account on the official MathWorks portal:
   [https://www.mathworks.com/downloads](https://www.mathworks.com/downloads)
2. Select **MATLAB R2026b** (or target release) and select platform **Linux (x86-64)**.
3. Download either:
   - The **MathWorks Product Installer** (`matlab_R2026b_Linux.zip` ~250MB online bootstrapper), which prompts for MathWorks credentials to stream the MATLAB engine and toolboxes over the network.
   - Or the **Full ISO / Offline Package** (typically distributed as a multi-gigabyte ISO or archive ~15–35 GB containing all product payloads under `archives/`).
4. Ensure you do NOT select the "Network License Manager" standalone download link, which produces only the 283 MB server package inspected here.

---

## 6. Re-verification addendum (2026-10-22)

All §2 SHA-256 digests were re-computed and matched. The crash analysis was
extended with post-mortem and static evidence:

1. **gdb on the stored core** (`/var/lib/systemd/coredump/...14499...zst`):
   `SIGSEGV (SEGV_MAPERR)` with **faulting address `si_addr = 0x6c65746e`** —
   the ASCII encoding of `"ntel"`, i.e. the low 32 bits of the `CPUID`
   vendor string `"GenuineIntel"` (ECX) being dereferenced as a pointer
   from `lc_init` (offset `0x121727`, called at fault site `+0x120c98`) in
   `bin/glnxa64/libmwinstall_activationwsclientimpl.so`
   (sha256 `2eee1d3f132be2c2200cd6ef371d8e1d1933a4376d7f41fceba5aade8be62c6a`).
   Invocation path: `MathWorksProductInstaller` main →
   `install::product_installer::LaunchHelper::launch` →
   `libmwcommonproductinstalldriver.so` → `dlopen` → `lc_init`.

2. **Static confirmation via REA** (evidence `ev_9f82dc265cd1…`,
   provider GNU objdump): the library's `.text` contains **exactly one
   `cpuid` instruction**; `lc_init` exports at `0x1216f0`. Machine-code
   fingerprint is consistent with the documented FLEXlm defect. (Tooling
   context: Ghidra and rizin are not installed; objdump/gdb-based REA
   providers are active and returned signed evidence.)

3. **Upstream shim provenance**: source fetched from
   `https://github.com/stefanoconiglio/MatlabR2026a-fix` (repo created
   2026-07-02; default branch `master`; **no LICENSE file** — local build
   kept for personal use, no redistribution). Reviewed: no network, exec,
   or file operations; maps three 1 MiB anonymous zero pages at the
   page-aligned vendor constants (`0x75600000`, `0x49600000`,
   `0x6c600000`) and installs a bounded SIGSEGV recovery handler. Rebuilt
   locally as `~/matlab_fix.so` and verified under `LD_PRELOAD` with a
   benign host process: all three pages appear in `/proc/self/maps`.

4. **Classification note**: §1 called this package "a Network License
   Manager installer". Precisely: it is the **standard MathWorks product
   installer bootstrap (~283 MB) that bundles the NLM payload archives**;
   no MATLAB/engine/toolbox archives are present — product payload must be
   fetched online (or via a licensed offline ISO). Commands to install:

   ```
   $ cd /home/cherry/Downloads/matlab_R2026b_Linux
   $ LD_PRELOAD=/home/cherry/matlab_fix.so ./install
   ```

   After installation, launch MATLAB the same way (or via the wrapper
   script from the ArchWiki section, which requires writing
   `/usr/local/bin/matlab` — a user-approved system change).
