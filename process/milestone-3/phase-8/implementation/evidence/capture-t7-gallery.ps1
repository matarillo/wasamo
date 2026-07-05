# T7 authoritative Gallery GUI evidence capture.
#
# Requires a visible Windows desktop. The script uses CopyFromScreen over the
# foreground window; DirectComposition content is blank under PrintWindow.
param(
  [string]$Tag = "t7-gallery"
)

$ErrorActionPreference = "Stop"

if (-not ('WinT7Cap' -as [type])) {
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class WinT7Cap {
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

[WinT7Cap]::SetProcessDpiAwarenessContext([IntPtr](-4)) | Out-Null
Add-Type -AssemblyName System.Drawing

$repo = (Resolve-Path "$PSScriptRoot\..\..\..\..\..").Path
$exe = Join-Path $repo "target\release\gallery-rust.exe"
if (-not (Test-Path $exe)) { throw "missing $exe" }

$p = Start-Process -FilePath $exe -PassThru
Start-Sleep -Seconds 3
$p.Refresh()
$h = $p.MainWindowHandle
if ($h -eq [IntPtr]::Zero) { throw "no MainWindowHandle (alive=$(-not $p.HasExited))" }

[WinT7Cap]::ShowWindow($h, 9) | Out-Null

function PositionWindow($width, $height) {
  [WinT7Cap]::MoveWindow($h, 0, 0, $width, $height, $true) | Out-Null
  [WinT7Cap]::SetWindowPos($h, [IntPtr](-1), 0, 0, $width, $height, 0x0040) | Out-Null
  [WinT7Cap]::SetForegroundWindow($h) | Out-Null
  Start-Sleep -Milliseconds 1000
}

function CaptureFrame($name) {
  $r = New-Object WinT7Cap+RECT
  [WinT7Cap]::GetWindowRect($h, [ref]$r) | Out-Null
  $actualW = $r.Right - $r.Left
  $actualH = $r.Bottom - $r.Top
  $out = Join-Path $PSScriptRoot "$Tag-$name.png"
  Write-Host "[$name] DPI(window)=$([WinT7Cap]::GetDpiForWindow($h)) physical window rect=($($r.Left),$($r.Top)) ${actualW}x${actualH}"
  if ($actualW -le 0 -or $actualH -le 0) { throw "invalid window size ${actualW}x${actualH}" }

  $bmp = New-Object System.Drawing.Bitmap $actualW, $actualH
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($r.Left, $r.Top, 0, 0, (New-Object System.Drawing.Size($actualW, $actualH)))
  $bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
  $g.Dispose()
  $bmp.Dispose()

  Write-Host "[$name] -> $out"
}

function ClickAt($x, $y) {
  [WinT7Cap]::SetCursorPos($x, $y) | Out-Null
  [WinT7Cap]::mouse_event(0x0002, 0, 0, 0, [IntPtr]::Zero)
  [WinT7Cap]::mouse_event(0x0004, 0, 0, 0, [IntPtr]::Zero)
}

try {
  PositionWindow 1200 760
  CaptureFrame "default-all"

  ClickAt 112 72
  Start-Sleep -Milliseconds 800
  CaptureFrame "selected-albums"

  ClickAt 220 72
  Start-Sleep -Milliseconds 800
  CaptureFrame "selected-favorites"

  ClickAt 1100 72
  Start-Sleep -Milliseconds 800
  CaptureFrame "lightbox-open"

  ClickAt 885 168
  Start-Sleep -Milliseconds 800
  CaptureFrame "lightbox-closed"

  PositionWindow 760 420
  CaptureFrame "narrow-before-scroll"

  ClickAt 397 72
  Start-Sleep -Milliseconds 800
  CaptureFrame "narrow-after-scroll"
}
finally {
  Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
}
