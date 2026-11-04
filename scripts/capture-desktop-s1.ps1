# S1 GUI audit capture: launches the debug desktop binary, resizes its window
# to the requested resolutions, and saves real PrintWindow screenshots into
# the requested out dir (default docs/screenshots/before/; use
# -OutDir docs/screenshots/after for post-change evidence). No code is modified by this script.
#
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File scripts\capture-desktop-s1.ps1
param(
    [string]$Binary = "target\debug\matrust_studio.exe",
    [int[]]$Widths = @(1280, 1920),
    [int[]]$Heights = @(720, 1080),
    [string]$OutDir = "docs\screenshots\before"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$exePath = Join-Path $root $Binary
if (-not (Test-Path $exePath)) {
    throw "Missing $exePath. Run: cargo build -p matrust_studio --bin matrust_studio"
}
$outDirPath = Join-Path $root $OutDir
New-Item -ItemType Directory -Force -Path $outDirPath | Out-Null

$definition = @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public class MatStudioCapture {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc cb, IntPtr lParam);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr hWnd, StringBuilder sb, int max);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hWnd, IntPtr hdcBlt, uint flags);
    [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr hWnd, int x, int y, int width, int height, bool repaint);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);

    // Title must come second so Find uses first-match-by-title-prefix.
    public static IntPtr Find(string titlePrefix) {
        IntPtr found = IntPtr.Zero;
        EnumWindows((h, l) => {
            var sb = new StringBuilder(512);
            GetWindowText(h, sb, 512);
            if (IsWindowVisible(h) && sb.ToString().StartsWith(titlePrefix, StringComparison.Ordinal)) {
                found = h;
                return false;
            }
            return true;
        }, IntPtr.Zero);
        return found;
    }

    // Same, but the window must BELONG to the given process id: several
    // unrelated windows (e.g. a terminal "MATSTUDIO — powershell.exe") can
    // share the title prefix, so the app pid disambiguates.
    public static IntPtr FindForPid(uint ownerPid, string titlePrefix) {
        IntPtr found = IntPtr.Zero;
        EnumWindows((h, l) => {
            var sb = new StringBuilder(512);
            GetWindowText(h, sb, 512);
            uint pid;
            GetWindowThreadProcessId(h, out pid);
            if (IsWindowVisible(h) && pid == ownerPid && sb.ToString().StartsWith(titlePrefix, StringComparison.Ordinal)) {
                found = h;
                return false;
            }
            return true;
        }, IntPtr.Zero);
        return found;
    }
}
"@
Add-Type -TypeDefinition $definition
Add-Type -AssemblyName System.Drawing

$process = Start-Process -FilePath $exePath -WorkingDirectory $root -PassThru
try {
    # Wait for the eframe window to appear (process name is matrust_studio,
    # window title comes from main.rs ("MATSTUDIO")).
    $deadline = (Get-Date).AddSeconds(12)
    $handle = [IntPtr]::Zero
    while ((Get-Date) -lt $deadline) {
        $handle = [MatStudioCapture]::FindForPid($process.Id, "MATSTUDIO")
        if ($handle -ne [IntPtr]::Zero) { break }
        Start-Sleep -Milliseconds 200
    }
    if ($handle -eq [IntPtr]::Zero) {
        throw "Window with title starting 'MATSTUDIO' was not found"
    }

    # Give the first UI frames time to layout.
    Start-Sleep -Milliseconds 1500

    for ($i = 0; $i -lt $Widths.Count; $i++) {
        $w = $Widths[$i]
        $h = $Heights[$i]
        $ok = [MatStudioCapture]::MoveWindow($handle, 0, 0, $w, $h, $true)
        if (-not $ok) { throw "MoveWindow failed for ${w}x${h}" }
        Start-Sleep -Milliseconds 1200

        $rect = New-Object MatStudioCapture+RECT
        [void][MatStudioCapture]::GetWindowRect($handle, [ref]$rect)
        $bw = $rect.Right - $rect.Left
        $bh = $rect.Bottom - $rect.Top

        $bitmap = New-Object System.Drawing.Bitmap($bw, $bh)
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $hdc = $graphics.GetHdc()
        try {
            [void][MatStudioCapture]::PrintWindow($handle, $hdc, 2)  # PW_RENDERFULLCONTENT
        }
        finally {
            $graphics.ReleaseHdc($hdc)
            $graphics.Dispose()
        }
        $pngPath = Join-Path $outDirPath ("desktop_{0}x{1}.png" -f $w, $h)
        $bitmap.Save($pngPath, [System.Drawing.Imaging.ImageFormat]::Png)
        $bitmap.Dispose()
        Write-Output ("captured={0} windowRect={1}x{2}" -f $pngPath, $bw, $bh)
    }
}
finally {
    if (-not $process.HasExited) {
        Stop-Process -Id $process.Id -Force
    }
}
