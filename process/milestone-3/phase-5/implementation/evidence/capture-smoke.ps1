# T6 assistant pre-baseline capture (DPI-aware).
#
# IMPORTANT: the gallery host is DPI-UNAWARE (GetDpiForWindow reports 96 even on a
# 125%-scaled monitor); Windows bitmap-stretches its logical 800x600 surface to
# physical 1000x750. A DPI-UNAWARE capturing process reads virtualized 96-dpi window
# rects but CopyFromScreen samples PHYSICAL pixels, so it would grab only the
# top-left logical-sized sub-rect and clip the right/bottom of the real window. We
# therefore make THIS process per-monitor-DPI-aware (V2) before any HWND/GDI call,
# read the true physical window rect, and crop the capture to it.
#
# The interop class is named WinCap (distinct from resize-test.ps1's WinResize) and
# guarded so both scripts can run in one PowerShell session and either can be re-run.
param([string]$OutPath = "$PSScriptRoot\t6-gallery-grid-launch.png")

if (-not ('WinCap' -as [type])) {
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class WinCap {
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr ctx);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
}
'@
}
[WinCap]::SetProcessDpiAwarenessContext([IntPtr](-4)) | Out-Null  # PER_MONITOR_AWARE_V2
Add-Type -AssemblyName System.Drawing

$exe = Join-Path (Resolve-Path "$PSScriptRoot\..\..\..\..\..").Path "target\release\gallery-rust.exe"
if (-not (Test-Path $exe)) { throw "missing $exe" }
$p = Start-Process -FilePath $exe -PassThru
Start-Sleep -Seconds 3
$p.Refresh()
$h = $p.MainWindowHandle
if ($h -eq [IntPtr]::Zero) { throw "no MainWindowHandle (alive=$(-not $p.HasExited))" }
[WinCap]::ShowWindow($h, 9) | Out-Null
[WinCap]::SetWindowPos($h, [IntPtr](-1), 0, 0, 0, 0, 0x0003) | Out-Null   # HWND_TOPMOST, NOMOVE|NOSIZE
[WinCap]::SetForegroundWindow($h) | Out-Null
Start-Sleep -Seconds 1

$wr = New-Object WinCap+RECT; [WinCap]::GetWindowRect($h, [ref]$wr) | Out-Null
$w = $wr.Right - $wr.Left; $ht = $wr.Bottom - $wr.Top
Write-Host "DPI(window)=$([WinCap]::GetDpiForWindow($h))  physical window rect=($($wr.Left),$($wr.Top)) ${w}x${ht}"

$bmp = New-Object System.Drawing.Bitmap $w, $ht
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($wr.Left, $wr.Top, 0, 0, (New-Object System.Drawing.Size($w, $ht)))
$bmp.Save($OutPath, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $bmp.Dispose()
Write-Host "saved $OutPath (${w}x${ht})"
Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
