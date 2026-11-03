# S1 extras: drives the real window with clicks to capture panels the default
# layout can't show. Click coordinates in pixels at 1920x1080 DPI-aware layout.
param(
    [string]$Binary = "target\debug\matrust_studio.exe"
)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$exePath = Join-Path $root $Binary
if (-not (Test-Path $exePath)) { throw "Missing $exePath" }
$outDir = Join-Path $root "docs\screenshots\before"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public class MsWin {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc cb, IntPtr lParam);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr hWnd, StringBuilder sb, int max);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hWnd, IntPtr hdcBlt, uint flags);
    [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr hWnd, int x, int y, int w, int h, bool repaint);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, int dx, int dy, uint data, IntPtr extra);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    public static IntPtr Find(string prefix) {
        IntPtr found = IntPtr.Zero;
        EnumWindows((h, l) => {
            var sb = new StringBuilder(512);
            GetWindowText(h, sb, 512);
            if (IsWindowVisible(h) && sb.ToString().StartsWith(prefix, StringComparison.Ordinal)) { found = h; return false; }
            return true;
        }, IntPtr.Zero);
        return found;
    }
    public static void Click(int x, int y) {
        SetCursorPos(x, y);
        System.Threading.Thread.Sleep(120);
        mouse_event(0x0002, 0, 0, 0, IntPtr.Zero); // LEFTDOWN
        System.Threading.Thread.Sleep(60);
        mouse_event(0x0004, 0, 0, 0, IntPtr.Zero); // LEFTUP
        System.Threading.Thread.Sleep(120);
    }
}
"@
Add-Type -AssemblyName System.Drawing

[void][MsWin]::SetProcessDPIAware()

function Save-Shot([IntPtr]$h, [string]$path) {
    $rect = New-Object MsWin+RECT
    [void][MsWin]::GetWindowRect($h, [ref]$rect)
    $w = $rect.Right - $rect.Left; $bh = $rect.Bottom - $rect.Top
    $bmp = New-Object System.Drawing.Bitmap($w, $bh)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $hdc = $g.GetHdc()
    try { [void][MsWin]::PrintWindow($h, $hdc, 2) } finally { $g.ReleaseHdc($hdc); $g.Dispose() }
    $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    Write-Output "captured=$path rect=${w}x${bh}"
}

$process = Start-Process -FilePath $exePath -WorkingDirectory $root -PassThru
try {
    $deadline = (Get-Date).AddSeconds(12)
    $handle = [IntPtr]::Zero
    while ((Get-Date) -lt $deadline) {
        $handle = [MsWin]::Find("MATSTUDIO")
        if ($handle -ne [IntPtr]::Zero) { break }
        Start-Sleep -Milliseconds 200
    }
    if ($handle -eq [IntPtr]::Zero) { throw "Window not found" }
    [void][MsWin]::SetForegroundWindow($handle)
    Start-Sleep -Milliseconds 1200
    [void][MsWin]::MoveWindow($handle, 0, 0, 1920, 1080, $true)
    Start-Sleep -Milliseconds 1200

    # 1. Block diagram panel: click the "Block Diagram" button on the HOME toolstrip.
    [MsWin]::Click(1568, 137)
    Start-Sleep -Milliseconds 1000
    Save-Shot $handle (Join-Path $outDir "desktop_block_diagram_panel.png")

    # 2. View menu open (screenshot menu itself).
    [MsWin]::Click(369, 88)
    Start-Sleep -Milliseconds 700
    Save-Shot $handle (Join-Path $outDir "desktop_view_menu.png")

    # 3. From the View menu, click the "Add-On Explorer" checkbox row.
    [MsWin]::Click(80, 425)
    Start-Sleep -Milliseconds 1000
    Save-Shot $handle (Join-Path $outDir "desktop_addons_panel.png")

    # 4. Command History panel: click its tab in the bottom-left leaf group.
    [MsWin]::Click(805, 938)
    Start-Sleep -Milliseconds 900
    Save-Shot $handle (Join-Path $outDir "desktop_history_panel.png")
}
finally {
    if (-not $process.HasExited) { Stop-Process -Id $process.Id -Force }
}
