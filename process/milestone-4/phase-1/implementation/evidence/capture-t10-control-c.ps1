# M4-Phase 1 T10 — positive control C, path form.
#
# Two (here three) frames across a REAL display-scale change, delivered by the
# OS to a live window. The assistant captures the path; a human makes the
# change. That division is the ADR's own — implementation/preamble.md's
# verification-closure item (5) says "assistant captures the **path** on the
# development machine, owner captures the **literal cross-monitor form**" —
# and the change is a human action here because this session has no working
# programmatic route to it (see §Control C in ../log.md).
#
# ── How to run it ────────────────────────────────────────────────────────────
#
#   1. Start this script. It launches the gallery host, positions it, and
#      captures the "before" frame.
#   2. It then polls GetDpiForWindow and waits. Open Settings > System >
#      Display and change **Scale** to any other value.
#   3. It captures the "after" frame as soon as the window's DPI moves, then
#      keeps waiting.
#   4. Set Scale back to its original value. It captures the "restored" frame
#      and exits.
#
# Nothing is typed at this script and it never takes the keyboard. It raises
# the host window with HWND_TOPMOST immediately before each capture — not
# SetForegroundWindow, which a background process cannot rely on and which
# would fight the Settings window for focus.
#
# ── What each leg is for ─────────────────────────────────────────────────────
#
# The invariants control C asserts are **element order and wrap structure**,
# not identical wrap positions (plan.md §T10; T4 delta review finding 2). On
# this path the OS chooses the window rectangle, so the client extent moves
# with the non-client metrics and a wrap position sitting on a line-break
# boundary may legitimately shift by a DIP or two. The restored leg is the
# cheap second instance: coming back to the original scale should reproduce
# the original layout, and it costs one more click.
#
# ── What is and is not exercised before a real change happens ────────────────
#
# The capture-and-record step is a single function used by all three legs, so
# the "before" leg exercises it for real every run. The **DPI-change detection
# comparison itself is not exercised until a change actually arrives** — that
# is stated rather than predicted away (T9 finding F-49: a hazard neutralised
# by reasoning in the same breath is what stops the check being run).
#
# Requires a build produced by `cargo build -p wasamo-runtime --release` then
# `cargo build --release --workspace` (T1 finding F-5, T3 finding F-21).
param(
  [string]$Tag = "t10-control-c",
  [string]$Exe,
  [int]$ClientW = 1200,
  [int]$ClientH = 720,
  [int]$WaitSeconds = 900
)

$ErrorActionPreference = "Stop"

if (-not ('WinT10C' -as [type])) {
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class WinT10C {
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
    [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr ctx);
    [DllImport("user32.dll")] public static extern IntPtr GetThreadDpiAwarenessContext();
    [DllImport("user32.dll")] public static extern IntPtr GetWindowDpiAwarenessContext(IntPtr h);
    [DllImport("user32.dll")] public static extern bool AreDpiAwarenessContextsEqual(IntPtr a, IntPtr b);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int hgt, bool repaint);
    [DllImport("user32.dll")] public static extern IntPtr MonitorFromWindow(IntPtr h, uint flags);
    [DllImport("shcore.dll")] public static extern int GetDpiForMonitor(IntPtr mon, int type, out uint dx, out uint dy);
}
'@
}

function AwarenessName([IntPtr]$ctx) {
  if ([WinT10C]::AreDpiAwarenessContextsEqual($ctx, [IntPtr](-4))) { return "PER_MONITOR_AWARE_V2" }
  if ([WinT10C]::AreDpiAwarenessContextsEqual($ctx, [IntPtr](-3))) { return "PER_MONITOR_AWARE_V1" }
  if ([WinT10C]::AreDpiAwarenessContextsEqual($ctx, [IntPtr](-2))) { return "SYSTEM_AWARE" }
  if ([WinT10C]::AreDpiAwarenessContextsEqual($ctx, [IntPtr](-1))) { return "UNAWARE" }
  return "UNRECOGNISED($ctx)"
}

# Attempt, discard the result, read back what is in force (T9 finding F-49).
[void][WinT10C]::SetProcessDpiAwarenessContext([IntPtr](-4))
$harness = AwarenessName ([WinT10C]::GetThreadDpiAwarenessContext())
if ($harness -ne "PER_MONITOR_AWARE_V2") {
  throw "harness awareness readback is $harness, not PER_MONITOR_AWARE_V2 — every coordinate would be virtualized (F-48). Aborting."
}

Add-Type -AssemblyName System.Drawing

$repo = (Resolve-Path "$PSScriptRoot\..\..\..\..\..").Path
if (-not $Exe) { $Exe = Join-Path $repo "target\release\gallery-rust.exe" }
if (-not (Test-Path $Exe)) { throw "missing $Exe" }

$outDir = Join-Path $PSScriptRoot $Tag
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$lines = New-Object System.Collections.Generic.List[string]
function Rec([string]$s) { Write-Host $s; $lines.Add($s) }
function Flush { Set-Content -Path (Join-Path $outDir "measurements.txt") -Value $lines }

Rec ("tag                : {0}" -f $Tag)
Rec ("started            : {0}" -f (Get-Date -Format "yyyy-MM-dd HH:mm:ss"))
Rec ("harness awareness  : {0}  (READ BACK, not the call's return value)" -f $harness)
Rec ("host executable    : {0}" -f $Exe)
Rec ("host build stamp   : {0}" -f (Get-Item $Exe).LastWriteTime.ToString("yyyy-MM-dd HH:mm:ss"))
Rec ("scale change by    : a human, in Settings > System > Display > Scale")
Rec ""

$proc = Start-Process -FilePath $Exe -PassThru
$script:HWND = [IntPtr]::Zero
for ($i = 0; $i -lt 60 -and $script:HWND -eq [IntPtr]::Zero; $i++) {
  Start-Sleep -Milliseconds 250
  $proc.Refresh()
  if ($proc.HasExited) { throw "host exited early (code $($proc.ExitCode))" }
  $script:HWND = $proc.MainWindowHandle
}
if ($script:HWND -eq [IntPtr]::Zero) { throw "no MainWindowHandle after 15s" }

$script:WR = New-Object WinT10C+RECT
$script:CR = New-Object WinT10C+RECT
function ReadRects {
  $w = New-Object WinT10C+RECT; $c = New-Object WinT10C+RECT
  [void][WinT10C]::GetWindowRect($script:HWND, [ref]$w)
  [void][WinT10C]::GetClientRect($script:HWND, [ref]$c)
  $script:WR = $w; $script:CR = $c
}
function CurOuterW { $script:WR.Right - $script:WR.Left }
function CurOuterH { $script:WR.Bottom - $script:WR.Top }
function CurClientW { $script:CR.Right - $script:CR.Left }
function CurClientH { $script:CR.Bottom - $script:CR.Top }

# One capture-and-record routine for all three legs, so the "before" leg
# exercises every line the later legs depend on.
function CaptureLeg([string]$leg) {
  [void][WinT10C]::SetWindowPos($script:HWND, [IntPtr](-1), 0, 0, 0, 0, 0x0043) # TOPMOST|NOMOVE|NOSIZE|SHOWWINDOW
  Start-Sleep -Milliseconds 1500
  ReadRects
  $dpi = [WinT10C]::GetDpiForWindow($script:HWND)
  $mon = [WinT10C]::MonitorFromWindow($script:HWND, 2)
  $mdx = 0; $mdy = 0
  [void][WinT10C]::GetDpiForMonitor($mon, 0, [ref]$mdx, [ref]$mdy)
  $ow = CurOuterW; $oh = CurOuterH; $cw = CurClientW; $ch = CurClientH
  $s = $dpi / 96.0
  $origin = New-Object WinT10C+POINT; $origin.X = 0; $origin.Y = 0
  [void][WinT10C]::ClientToScreen($script:HWND, [ref]$origin)

  $bmp = New-Object System.Drawing.Bitmap $cw, $ch
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($origin.X, $origin.Y, 0, 0, (New-Object System.Drawing.Size($cw, $ch)))
  $bmp.Save((Join-Path $outDir "$leg.png"), [System.Drawing.Imaging.ImageFormat]::Png)
  $g.Dispose(); $bmp.Dispose()

  Rec ("[{0}] at {1}" -f $leg, (Get-Date -Format "HH:mm:ss"))
  Rec ("  window awareness : {0}" -f (AwarenessName ([WinT10C]::GetWindowDpiAwarenessContext($script:HWND))))
  Rec ("  GetDpiForWindow  : {0}   monitor DPI: {1}" -f $dpi, $mdx)
  Rec ("  outer            : {0}x{1}  (physical)" -f $ow, $oh)
  Rec ("  client           : {0}x{1}  (physical)" -f $cw, $ch)
  Rec ("  non-client frame : {0} wide, {1} tall" -f ($ow - $cw), ($oh - $ch))
  Rec ("  layout extent    : {0} x {1} DIP" -f ($cw / $s), ($ch / $s))
  Rec ("  client origin    : ({0},{1})  (screen, physical)" -f $origin.X, $origin.Y)
  Rec ("  frame            : {0}.png  ({1}x{2})" -f $leg, $cw, $ch)
  Rec ""
  Flush
  return $dpi
}

try {
  # Position once, before the "before" frame. After that the window is left
  # alone: on this path **the OS chooses the rectangle**, and resizing it
  # afterwards would destroy the thing being measured (the same reason
  # capture-t4-probe.ps1 never moves its window).
  ReadRects
  $ow = CurOuterW; $oh = CurOuterH
  for ($i = 0; $i -lt 10; $i++) {
    $ow += $ClientW - (CurClientW); $oh += $ClientH - (CurClientH)
    [void][WinT10C]::MoveWindow($script:HWND, 40, 40, $ow, $oh, $true)
    Start-Sleep -Milliseconds 350
    ReadRects
    if ((CurClientW) -eq $ClientW -and (CurClientH) -eq $ClientH) { break }
  }
  Rec ("positioned client  : {0}x{1} physical (target {2}x{3})" -f (CurClientW), (CurClientH), $ClientW, $ClientH)
  Rec ""

  $baseline = CaptureLeg "1-before"

  Rec ("waiting for a display-scale change (up to {0}s) — change Settings > System > Display > Scale now" -f $WaitSeconds)
  Flush
  $changed = 0
  $deadline = (Get-Date).AddSeconds($WaitSeconds)
  while ((Get-Date) -lt $deadline) {
    Start-Sleep -Milliseconds 500
    $proc.Refresh()
    if ($proc.HasExited) { throw "host exited while waiting for the scale change" }
    $now = [WinT10C]::GetDpiForWindow($script:HWND)
    if ($now -ne $baseline) { $changed = $now; break }
  }
  if ($changed -eq 0) {
    Rec ("NO CHANGE OBSERVED within {0}s. The window's DPI stayed {1}." -f $WaitSeconds, $baseline)
    Rec ("This records that no change reached the window — it does NOT establish why.")
    Flush
    return
  }
  Rec ("DPI moved {0} -> {1}" -f $baseline, $changed)
  Rec ""
  [void](CaptureLeg "2-changed")

  Rec ("waiting for the scale to be set back (up to {0}s)" -f $WaitSeconds)
  Flush
  $deadline = (Get-Date).AddSeconds($WaitSeconds)
  $restored = 0
  while ((Get-Date) -lt $deadline) {
    Start-Sleep -Milliseconds 500
    $proc.Refresh()
    if ($proc.HasExited) { throw "host exited while waiting for the scale to be restored" }
    $now = [WinT10C]::GetDpiForWindow($script:HWND)
    if ($now -eq $baseline) { $restored = $now; break }
  }
  if ($restored -eq 0) {
    Rec ("NOT RESTORED within {0}s; the two-frame pair above stands on its own." -f $WaitSeconds)
    Flush
    return
  }
  Rec ("DPI moved {0} -> {1} (restored)" -f $changed, $restored)
  Rec ""
  [void](CaptureLeg "3-restored")
}
finally {
  Flush
  if ($null -ne $proc -and -not $proc.HasExited) { Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue }
  Write-Host "  -> $outDir"
}
