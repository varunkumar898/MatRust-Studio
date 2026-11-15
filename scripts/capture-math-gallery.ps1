# S1/MG Capture script: launches MatRust Studio with Math Gallery open in the central dock
param(
    [string]$Binary = "target\debug\matrust_studio.exe",
    [string]$OutDir = "docs\screenshots\math-gallery"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$exePath = Join-Path $root $Binary
if (-not (Test-Path $exePath)) {
    throw "Missing $exePath. Run: cargo build -p matrust_studio --bin matrust_studio"
}
$outDirPath = Join-Path $root $OutDir
New-Item -ItemType Directory -Force -Path $outDirPath | Out-Null

$artifactDir = "C:\Users\varun\.gemini\antigravity\brain\6ef0c097-88ec-4cdd-aa79-fc2b17e3e0fa"
if (Test-Path $artifactDir) {
    New-Item -ItemType Directory -Force -Path $artifactDir | Out-Null
}

$definition = @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public class MatStudioCaptureMG {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc cb, IntPtr lParam);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr hWnd, StringBuilder sb, int max);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hWnd, IntPtr hdcBlt, uint flags);
    [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr hWnd, int x, int y, int width, int height, bool repaint);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr value);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int command);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hWnd, IntPtr after, int x, int y, int cx, int cy, uint flags);

    public static IntPtr FindForPid(uint ownerPid, string titlePrefix) {
        IntPtr found = IntPtr.Zero;
        long bestArea = 0;
        EnumWindows((h, l) => {
            var sb = new StringBuilder(512);
            GetWindowText(h, sb, 512);
            uint pid;
            GetWindowThreadProcessId(h, out pid);
            if (IsWindowVisible(h) && pid == ownerPid && sb.ToString().StartsWith(titlePrefix, StringComparison.Ordinal)) {
                RECT r;
                GetWindowRect(h, out r);
                long area = (long)(r.Right - r.Left) * (r.Bottom - r.Top);
                if (area > bestArea) {
                    bestArea = area;
                    found = h;
                }
            }
            return true;
        }, IntPtr.Zero);
        return found;
    }
}
"@
if (-not ([System.Management.Automation.PSTypeName]'MatStudioCaptureMG').Type) {
    Add-Type -TypeDefinition $definition
}
[void][MatStudioCaptureMG]::SetProcessDPIAware()
[void][MatStudioCaptureMG]::SetProcessDpiAwarenessContext([IntPtr](-4))
Add-Type -AssemblyName System.Drawing

function Capture-AppInstance([hashtable]$envVars, [string]$outName, [int]$w = 1564, [int]$h = 960) {
    foreach ($k in $envVars.Keys) {
        Set-Item -Path "env:$k" -Value $envVars[$k]
    }

    $proc = Start-Process -FilePath $exePath -WorkingDirectory $root -PassThru
    try {
        $deadline = (Get-Date).AddSeconds(12)
        $handle = [IntPtr]::Zero
        while ((Get-Date) -lt $deadline) {
            $handle = [MatStudioCaptureMG]::FindForPid($proc.Id, "MATSTUDIO")
            if ($handle -ne [IntPtr]::Zero) {
                $probe = New-Object MatStudioCaptureMG+RECT
                [void][MatStudioCaptureMG]::GetWindowRect($handle, [ref]$probe)
                $foundW = $probe.Right - $probe.Left
                $foundH = $probe.Bottom - $probe.Top
                if ($foundW -ge 800 -and $foundH -ge 500) { break }
            }
            Start-Sleep -Milliseconds 200
        }
        if ($handle -eq [IntPtr]::Zero) {
            throw "Window with title starting 'MATSTUDIO' was not found"
        }

        Start-Sleep -Milliseconds 2500

        [void][MatStudioCaptureMG]::ShowWindow($handle, 9)
        [void][MatStudioCaptureMG]::SetForegroundWindow($handle)
        Start-Sleep -Milliseconds 300
        $ok = [MatStudioCaptureMG]::MoveWindow($handle, 0, 0, $w, $h, $true)
        if (-not $ok) {
            $ok = [MatStudioCaptureMG]::SetWindowPos($handle, [IntPtr]::Zero, 0, 0, $w, $h, 0x0002 -bor 0x0001)
        }
        Start-Sleep -Milliseconds 2500

        $rect = New-Object MatStudioCaptureMG+RECT
        [void][MatStudioCaptureMG]::GetWindowRect($handle, [ref]$rect)
        $bw = $rect.Right - $rect.Left
        $bh = $rect.Bottom - $rect.Top

        $bitmap = New-Object System.Drawing.Bitmap($bw, $bh)
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $hdc = $graphics.GetHdc()
        try {
            [void][MatStudioCaptureMG]::PrintWindow($handle, $hdc, 2)
        }
        finally {
            $graphics.ReleaseHdc($hdc)
            $graphics.Dispose()
        }
        $pngPath = Join-Path $outDirPath $outName
        $bitmap.Save($pngPath, [System.Drawing.Imaging.ImageFormat]::Png)

        if (Test-Path $artifactDir) {
            $artPath = Join-Path $artifactDir $outName
            $bitmap.Save($artPath, [System.Drawing.Imaging.ImageFormat]::Png)
            Write-Output ("Saved artifact screenshot: {0}" -f $artPath)
        }

        $bitmap.Dispose()
        Write-Output ("captured={0} windowRect={1}x{2}" -f $pngPath, $bw, $bh)
    }
    finally {
        if (-not $proc.HasExited) {
            Stop-Process -Id $proc.Id -Force
        }
        foreach ($k in $envVars.Keys) {
            Remove-Item -Path "env:$k" -ErrorAction SilentlyContinue
        }
    }
}

# 1. Main Math Gallery central layout screenshot
Write-Output "Capturing Math Gallery central workspace layout..."
Capture-AppInstance @{
    MATSTUDIO_RESET_LAYOUT = "1"
    MATSTUDIO_START_TAB = "apps"
    MATSTUDIO_START_PANEL = "gallery"
} "math_gallery_1564x960.png" 1564 960

# 2. Math Gallery eigen_modes execution with Figures plot visible
Write-Output "Capturing Math Gallery eigen_modes execution with Figure plot..."
$eigenScript = Get-Content (Join-Path $root "examples\math\linear_algebra\eigen_modes.m") -Raw
Capture-AppInstance @{
    MATSTUDIO_RESET_LAYOUT = "1"
    MATSTUDIO_START_TAB = "apps"
    MATSTUDIO_START_PANEL = "gallery"
    MATSTUDIO_START_GALLERY_ID = "linalg-eigen"
    MATSTUDIO_START_RUN = $eigenScript
} "math_gallery_eigen_modes_run.png" 1564 960
