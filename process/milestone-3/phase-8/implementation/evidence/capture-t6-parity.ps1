# T6 cross-host default-render parity precheck.
#
# Captures the first-loaded Gallery view in Rust, C, and Zig hosts. This is a
# T6 parity precheck only; T7 owns the authoritative GUI positive controls.
param(
  [string]$Tag = "t6-parity"
)

$ErrorActionPreference = "Stop"

if (-not ('WinT6Cap' -as [type])) {
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class WinT6Cap {
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr ctx);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int hgt, bool repaint);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
}
'@
}

[WinT6Cap]::SetProcessDpiAwarenessContext([IntPtr](-4)) | Out-Null
Add-Type -AssemblyName System.Drawing

$repo = (Resolve-Path "$PSScriptRoot\..\..\..\..\..").Path
$targetRelease = Join-Path $repo "target\release"
$env:PATH = "$targetRelease;$env:PATH"

$hosts = @(
  @{ Name = "rust"; Exe = Join-Path $repo "target\release\gallery-rust.exe" },
  @{ Name = "c"; Exe = Join-Path $repo "build\gallery-c\Release\gallery-c.exe" },
  @{ Name = "zig"; Exe = Join-Path $repo "build\gallery-zig\bin\gallery-zig.exe" }
)

function CaptureHost($hostInfo) {
  $name = $hostInfo.Name
  $exe = $hostInfo.Exe
  if (-not (Test-Path $exe)) { throw "missing $name host executable: $exe" }

  $p = Start-Process -FilePath $exe -PassThru
  try {
    Start-Sleep -Seconds 3
    $p.Refresh()
    $h = $p.MainWindowHandle
    if ($h -eq [IntPtr]::Zero) { throw "$name host has no MainWindowHandle (alive=$(-not $p.HasExited))" }

    [WinT6Cap]::ShowWindow($h, 9) | Out-Null
    [WinT6Cap]::MoveWindow($h, 0, 0, 1200, 760, $true) | Out-Null
    [WinT6Cap]::SetWindowPos($h, [IntPtr](-1), 0, 0, 1200, 760, 0x0040) | Out-Null
    [WinT6Cap]::SetForegroundWindow($h) | Out-Null
    Start-Sleep -Milliseconds 1000

    $r = New-Object WinT6Cap+RECT
    [WinT6Cap]::GetWindowRect($h, [ref]$r) | Out-Null
    $actualW = $r.Right - $r.Left
    $actualH = $r.Bottom - $r.Top
    if ($actualW -le 0 -or $actualH -le 0) { throw "$name host invalid window size ${actualW}x${actualH}" }

    $out = Join-Path $PSScriptRoot "$Tag-$name.png"
    Write-Host "[$name] DPI(window)=$([WinT6Cap]::GetDpiForWindow($h)) physical window rect=($($r.Left),$($r.Top)) ${actualW}x${actualH}"

    $bmp = New-Object System.Drawing.Bitmap $actualW, $actualH
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($r.Left, $r.Top, 0, 0, (New-Object System.Drawing.Size($actualW, $actualH)))
    $bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose()
    $bmp.Dispose()

    Write-Host "[$name] -> $out"
  }
  finally {
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
  }
}

foreach ($hostInfo in $hosts) {
  CaptureHost $hostInfo
}
