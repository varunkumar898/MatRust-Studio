# RustLab Conformance Test Runner
# Verifies test scripts in tests/conformance against real MATLAB oracle semantics or committed .expected files.

param(
    [switch]$RegenerateWithMatlab
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir
$ConfDir = Join-Path $RootDir "tests\conformance"

Write-Host "============================================================"
Write-Host "RustLab Conformance Test Suite Runner"
Write-Host "Directory: $ConfDir"
Write-Host "============================================================"

# Check MATLAB availability
$matlabCmd = Get-Command "matlab" -ErrorAction SilentlyContinue
if (-not $matlabCmd -and $env:MATLAB_ROOT) {
    $candidate = Join-Path $env:MATLAB_ROOT "bin\matlab.exe"
    if (Test-Path $candidate) {
        $matlabCmd = $candidate
    }
}

if ($RegenerateWithMatlab) {
    if ($matlabCmd) {
        Write-Host "[ORACLE] Using real MATLAB: $matlabCmd"
        $mFiles = Get-ChildItem -Path $ConfDir -Filter "*.m"
        foreach ($file in $mFiles) {
            Write-Host "Running oracle for $($file.Name)..."
            # Execute with MATLAB emitting numeric values
            $outPath = [System.IO.Path]::ChangeExtension($file.FullName, ".expected")
            $batchCmd = "try, run('$($file.FullName.Replace('\', '/'))'); catch ME, disp(ME.message); end; quit;"
            & $matlabCmd -batch $batchCmd | Out-File -FilePath $outPath -Encoding utf8
        }
    } else {
        Write-Warning "[SKIPPED REAL MATLAB RUNNER]: MATLAB binary not found on PATH or MATLAB_ROOT. Using verified .expected files."
    }
}

Write-Host "[RUNNER] Executing RustLab conformance tests..."
cargo test -p rustlab-stdlib --test conformance_tests -- --nocapture
if ($LASTEXITCODE -ne 0) {
    Write-Error "Conformance tests failed with exit code $LASTEXITCODE"
    exit $LASTEXITCODE
}
Write-Host "All conformance tests passed successfully."
