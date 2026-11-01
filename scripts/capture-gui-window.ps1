param(
    [string]$Package = "fixtures/sample_apps/r2021a_sample.mlapp",
    [string]$Output = "docs/assets/gui_window_live.png",
    [string]$Title = "FixtureApp_R2021a",
    [int]$AutoCloseMs = 6000
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root "target/debug/mlapp-cli.exe"
$packagePath = if ([System.IO.Path]::IsPathRooted($Package)) { $Package } else { Join-Path $root $Package }
$outputPath = if ([System.IO.Path]::IsPathRooted($Output)) { $Output } else { Join-Path $root $Output }

if (-not (Test-Path $exe)) {
    throw "Missing $exe. Run: cargo build -p mlapp-cli"
}

$capture = {
    param($windowTitle, $pngPath)

    Add-Type -AssemblyName System.Drawing
    $source = @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public class MlappWindowCapture {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc cb, IntPtr lParam);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr hWnd, StringBuilder sb, int max);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hWnd, IntPtr hdcBlt, uint flags);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);

    public static IntPtr Find(string title) {
        IntPtr found = IntPtr.Zero;
        EnumWindows((h, l) => {
            var sb = new StringBuilder(512);
            GetWindowText(h, sb, 512);
            if (IsWindowVisible(h) && sb.ToString() == title) {
                found = h;
                return false;
            }
            return true;
        }, IntPtr.Zero);
        return found;
    }

    public static int[] Bounds(IntPtr h) {
        RECT r;
        GetWindowRect(h, out r);
        return new int[] { r.Right - r.Left, r.Bottom - r.Top };
    }
}
"@
    Add-Type -TypeDefinition $source

    $deadline = (Get-Date).AddSeconds(5)
    $handle = [IntPtr]::Zero
    while ((Get-Date) -lt $deadline) {
        $handle = [MlappWindowCapture]::Find($windowTitle)
        if ($handle -ne [IntPtr]::Zero) { break }
        Start-Sleep -Milliseconds 100
    }
    if ($handle -eq [IntPtr]::Zero) {
        throw "Window '$windowTitle' was not found"
    }

    $size = [MlappWindowCapture]::Bounds($handle)
    $bitmap = New-Object System.Drawing.Bitmap($size[0], $size[1])
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $hdc = $graphics.GetHdc()
    try {
        $ok = [MlappWindowCapture]::PrintWindow($handle, $hdc, 2)
    }
    finally {
        $graphics.ReleaseHdc($hdc)
        $graphics.Dispose()
    }
    if (-not $ok) {
        $bitmap.Dispose()
        throw "PrintWindow failed for '$windowTitle'"
    }

    $directory = Split-Path -Parent $pngPath
    if ($directory) { New-Item -ItemType Directory -Force -Path $directory | Out-Null }
    $bitmap.Save($pngPath, [System.Drawing.Imaging.ImageFormat]::Png)
    $bitmap.Dispose()
    Write-Output "captured=$pngPath"
}

$job = Start-Job -ScriptBlock $capture -ArgumentList $Title, $outputPath
$oldAutoClose = $env:MLAPP_GUI_AUTOCLOSE_MS
$env:MLAPP_GUI_AUTOCLOSE_MS = "$AutoCloseMs"
try {
    & $exe run --gui $packagePath
    if ($LASTEXITCODE -ne 0) {
        throw "mlapp-cli exited with $LASTEXITCODE"
    }
    Wait-Job $job -Timeout 15 | Out-Null
    Receive-Job $job
    if (-not (Test-Path $outputPath)) {
        throw "Screenshot was not written to $outputPath"
    }
}
finally {
    Remove-Job $job -Force -ErrorAction SilentlyContinue
    if ($null -eq $oldAutoClose) {
        Remove-Item Env:MLAPP_GUI_AUTOCLOSE_MS -ErrorAction SilentlyContinue
    }
    else {
        $env:MLAPP_GUI_AUTOCLOSE_MS = $oldAutoClose
    }
}
