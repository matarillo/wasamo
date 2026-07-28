# M4-Phase 1 T4 probe capture — the per-window scale seeding and the DIP
# window-size correction.
#
# Unlike capture-t3-label-writes.ps1 this script **never moves or resizes the
# window**: the size `window::create` produced *is* the measurement, so
# repositioning would destroy the artifact. It reports the outer rect, the
# client rect and `GetDpiForWindow` alongside the frame.
#
# Run once per state, against a build produced by
# `cargo build --release --workspace` (T3 finding F-21 — a host-package build
# relinks wasamo.dll around a stale uplifted rlib and the frame silently shows
# the previous runtime):
#
#   unaware-with-correction    the tree as committed at T4 — the shipped state
#   aware-without-correction   throwaway V2 declaration, correction call removed
#   aware-with-correction      throwaway V2 declaration only
#
# The three are the positive control and no one of them is evidence: the first
# and third share a window rectangle (1000x750), the first and second share a
# WrapPanel tile count (7). Only the pair of numbers separates all three. The
# two throwaway states require source edits that T4 reverts before it closes;
# T9 is the task that lands the declaration for real.
param([Parameter(Mandatory = $true)][string]$Tag)

$ErrorActionPreference = "Stop"

if (-not ('WinT4Cap' -as [type])) {
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class WinT4Cap {
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr ctx);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
}
'@
}

[WinT4Cap]::SetProcessDpiAwarenessContext([IntPtr](-4)) | Out-Null
Add-Type -AssemblyName System.Drawing

$repo = (Resolve-Path "$PSScriptRoot\..\..\..\..\..").Path
$exe = Join-Path $repo "target\release\gallery-rust.exe"
if (-not (Test-Path $exe)) { throw "missing $exe — run cargo build --release --workspace first" }
$outDir = Join-Path $PSScriptRoot "t4-probe"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

$proc = Start-Process -FilePath $exe -PassThru
Start-Sleep -Seconds 4
$proc.Refresh()
$hwnd = $proc.MainWindowHandle
if ($hwnd -eq [IntPtr]::Zero) { Stop-Process -Id $proc.Id -Force; throw "no MainWindowHandle" }
[WinT4Cap]::SetForegroundWindow($hwnd) | Out-Null
Start-Sleep -Milliseconds 1200

$r = New-Object WinT4Cap+RECT
$c = New-Object WinT4Cap+RECT
[WinT4Cap]::GetWindowRect($hwnd, [ref]$r) | Out-Null
[WinT4Cap]::GetClientRect($hwnd, [ref]$c) | Out-Null
$w = $r.Right - $r.Left
$h = $r.Bottom - $r.Top
Write-Host ("[{0}] GetDpiForWindow={1} window={2}x{3} client={4}x{5} at ({6},{7})" -f `
  $Tag, [WinT4Cap]::GetDpiForWindow($hwnd), $w, $h, ($c.Right - $c.Left), ($c.Bottom - $c.Top), $r.Left, $r.Top)

$bmp = New-Object System.Drawing.Bitmap $w, $h
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($r.Left, $r.Top, 0, 0, (New-Object System.Drawing.Size($w, $h)))
$out = Join-Path $outDir "$Tag.png"
$bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()
Write-Host "  -> $out"
Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
