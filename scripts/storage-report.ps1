# Storage & Artifact Footprint Audit Report (PowerShell)
# Enforces budgets:
# - Tracked repo size <= 50 MB
# - Largest tracked file <= 500 KB
# - Release binary <= 13 MB (measured 10.20 MB + 20% headroom)
# - Shared target directory <= 20 GB
# - Free disk space >= 20 GB

$ErrorActionPreference = "Stop"
$budgetExceeded = $false

Write-Host "=================================================="
Write-Host "      MATRUST STUDIO STORAGE AUDIT REPORT        "
Write-Host "=================================================="

# 1. Tracked Files Size & Top 15
Write-Host "`n--- 1. Tracked Repository Files ---"
$trackedFiles = git ls-files | ForEach-Object {
    if (Test-Path $_) {
        Get-Item $_
    }
}

$trackedTotalBytes = ($trackedFiles | Measure-Object -Property Length -Sum).Sum
$trackedTotalMB = [math]::Round($trackedTotalBytes / 1MB, 2)
Write-Host "Total Tracked Files: $($trackedFiles.Count)"
Write-Host "Total Tracked Size:  $trackedTotalMB MB ($trackedTotalBytes bytes) [Budget: <= 50 MB]"

if ($trackedTotalBytes -gt 50MB) {
    Write-Host "[ERROR] Tracked repository size exceeds 50 MB budget!" -ForegroundColor Red
    $budgetExceeded = $true
} else {
    Write-Host "[OK] Tracked repository size within budget." -ForegroundColor Green
}

Write-Host "`nTop 15 Largest Tracked Files:"
$top15 = $trackedFiles | Sort-Object Length -Descending | Select-Object -First 15
$top15 | ForEach-Object {
    $kb = [math]::Round($_.Length / 1KB, 1)
    Write-Host ("  {0,8} KB  {1}" -f $kb, (Resolve-Path -Relative $_.FullName))
    if ($_.Length -gt 500KB) {
        Write-Host "    [ERROR] File exceeds 500 KB threshold!" -ForegroundColor Red
        $budgetExceeded = $true
    }
}

# 2. Target Directory Size (respecting CARGO_TARGET_DIR)
Write-Host "`n--- 2. Build Target Directory ---"
$targetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { "target" }
if (Test-Path $targetDir) {
    $targetBytes = (Get-ChildItem -Path $targetDir -Recurse -File -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum
    $targetGB = [math]::Round($targetBytes / 1GB, 2)
    Write-Host "Target Directory: $targetDir"
    Write-Host "Target Total Size: $targetGB GB ($targetBytes bytes) [Alarm threshold: <= 20 GB]"
    if ($targetBytes -gt 20GB) {
        Write-Host "[ALARM] Target directory exceeds 20 GB alarm threshold! Consider running 'cargo clean'." -ForegroundColor Yellow
    } else {
        Write-Host "[OK] Target directory within alarm threshold." -ForegroundColor Green
    }
} else {
    Write-Host "Target Directory ($targetDir) does not exist (clean state)."
}

# 3. Release Executable Size
Write-Host "`n--- 3. Release Executable Size ---"
$releaseExe = Join-Path $targetDir "release\matrust_studio.exe"
if (Test-Path $releaseExe) {
    $exeItem = Get-Item $releaseExe
    $exeMB = [math]::Round($exeItem.Length / 1MB, 2)
    Write-Host "Release Binary: $releaseExe"
    Write-Host "Binary Size:    $exeMB MB ($($exeItem.Length) bytes) [Budget: <= 13.0 MB (measured + 20%)]"
    if ($exeItem.Length -gt 13MB) {
        Write-Host "[ERROR] Release binary size exceeds budget!" -ForegroundColor Red
        $budgetExceeded = $true
    } else {
        Write-Host "[OK] Release binary within budget." -ForegroundColor Green
    }
} else {
    Write-Host "Release binary ($releaseExe) not found; run cargo build --release -p matrust_studio to measure."
}

# 4. Git Worktrees
Write-Host "`n--- 4. Active Git Worktrees ---"
git worktree list

# 5. Free Disk Space
Write-Host "`n--- 5. Free Disk Space ---"
$driveLetter = (Get-Location).Drive.Name
if (-not $driveLetter) { $driveLetter = "C" }
try {
    $driveInfo = [System.IO.DriveInfo]::new($driveLetter)
    $freeBytes = $driveInfo.AvailableFreeSpace
    $freeGB = [math]::Round($freeBytes / 1GB, 2)
    Write-Host "Drive ${driveLetter}: Free Space: $freeGB GB ($freeBytes bytes) [Threshold: >= 20 GB]"
    if ($freeBytes -lt 20GB) {
        Write-Host "[ERROR] Free disk space is below 20 GB minimum!" -ForegroundColor Red
        $budgetExceeded = $true
    } else {
        Write-Host "[OK] Free disk space meets minimum requirement." -ForegroundColor Green
    }
} catch {
    Write-Host "Could not query free disk space: $_"
}

Write-Host "`n=================================================="
if ($budgetExceeded) {
    Write-Host "AUDIT RESULT: FAILED (Budget thresholds violated)" -ForegroundColor Red
    exit 1
} else {
    Write-Host "AUDIT RESULT: PASSED (All storage budgets respected)" -ForegroundColor Green
    exit 0
}
