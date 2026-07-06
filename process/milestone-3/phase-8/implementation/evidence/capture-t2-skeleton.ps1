# T2 layout-skeleton technical smoke (DPI-aware).
#
# Captures the Phase 8 gallery skeleton at two window widths, then captures
# explicit scroll and lightbox states. The resize pair is the positive control:
# it distinguishes a live WrapPanel/ScrollView layout from a coincidental static
# frame.
param(
  [string]$Tag = "t2-skeleton"
)

if (-not ('WinT2Cap' -as [type])) {
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class WinT2Cap {
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr ctx);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int hgt, bool repaint);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, IntPtr extraInfo);
}
'@
}

[WinT2Cap]::SetProcessDpiAwarenessContext([IntPtr](-4)) | Out-Null
Add-Type -AssemblyName System.Drawing

$repo = (Resolve-Path "$PSScriptRoot\..\..\..\..\..").Path
$exe = Join-Path $repo "target\release\gallery-rust.exe"
if (-not (Test-Path $exe)) { throw "missing $exe" }

$p = Start-Process -FilePath $exe -PassThru
Start-Sleep -Seconds 3
$p.Refresh()
$h = $p.MainWindowHandle
if ($h -eq [IntPtr]::Zero) { throw "no MainWindowHandle (alive=$(-not $p.HasExited))" }

[WinT2Cap]::ShowWindow($h, 9) | Out-Null

function CaptureFrame($width, $height, $name) {
  $moved = [WinT2Cap]::MoveWindow($h, 0, 0, $width, $height, $true)
  $positioned = [WinT2Cap]::SetWindowPos($h, [IntPtr](-1), 0, 0, $width, $height, 0x0040)
  [WinT2Cap]::SetForegroundWindow($h) | Out-Null
  Start-Sleep -Milliseconds 1200

  $r = New-Object WinT2Cap+RECT
  [WinT2Cap]::GetWindowRect($h, [ref]$r) | Out-Null
  $actualW = $r.Right - $r.Left
  $actualH = $r.Bottom - $r.Top
  $out = Join-Path $PSScriptRoot "$Tag-$name.png"
  Write-Host "[$name] moved=$moved positioned=$positioned DPI(window)=$([WinT2Cap]::GetDpiForWindow($h)) physical window rect=($($r.Left),$($r.Top)) ${actualW}x${actualH}"
  if ($actualW -le 0 -or $actualH -le 0) { throw "invalid window size ${actualW}x${actualH}" }

  $bmp = New-Object System.Drawing.Bitmap $actualW, $actualH
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($r.Left, $r.Top, 0, 0, (New-Object System.Drawing.Size($actualW, $actualH)))
  $bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
  $g.Dispose()
  $bmp.Dispose()

  Write-Host "[$name] DPI(window)=$([WinT2Cap]::GetDpiForWindow($h)) physical window rect=($($r.Left),$($r.Top)) ${actualW}x${actualH} -> $out"
}

function ClickAt($x, $y) {
  [WinT2Cap]::SetCursorPos($x, $y) | Out-Null
  [WinT2Cap]::mouse_event(0x0002, 0, 0, 0, [IntPtr]::Zero)
  [WinT2Cap]::mouse_event(0x0004, 0, 0, 0, [IntPtr]::Zero)
}

try {
  CaptureFrame 1200 800 "wide"
  CaptureFrame 760 800 "narrow"
  CaptureFrame 760 420 "scroll-before"
  [WinT2Cap]::MoveWindow($h, 0, 0, 760, 420, $true) | Out-Null
  [WinT2Cap]::SetWindowPos($h, [IntPtr](-1), 0, 0, 760, 420, 0x0040) | Out-Null
  [WinT2Cap]::SetForegroundWindow($h) | Out-Null
  Start-Sleep -Milliseconds 800
  ClickAt 397 72
  Start-Sleep -Milliseconds 800
  CaptureFrame 760 420 "scrolled"
  [WinT2Cap]::MoveWindow($h, 0, 0, 1200, 800, $true) | Out-Null
  [WinT2Cap]::SetWindowPos($h, [IntPtr](-1), 0, 0, 1200, 800, 0x0040) | Out-Null
  [WinT2Cap]::SetForegroundWindow($h) | Out-Null
  Start-Sleep -Milliseconds 800
  ClickAt 1100 72
  Start-Sleep -Milliseconds 800
  CaptureFrame 1200 800 "lightbox"
}
finally {
  Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
}
