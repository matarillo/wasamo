# M4-Phase 1 T10 — the positive-control capture harness.
#
# One invocation = one launch, one capture, one measurement record, into its
# own tag directory. Repeats are separate invocations with separate tags,
# because T5 finding F-33 measured that a single frame is not a baseline and
# that a session's first launch can differ from its own later ones.
#
# ── Why this script reads its own DPI awareness back ─────────────────────────
#
# T9 finding F-48: a DPI-unaware process asking GetWindowRect / GetClientRect
# about an *aware* window is answered in virtualized coordinates -- the real
# rectangle divided by the system scale. Every number this script prints would
# be wrong by exactly the scale factor, internally consistent, and plausible.
#
# T9 finding F-49: SetProcessDpiAwarenessContext can fail because the process
# awareness was already set before any of this script's code ran. "The call
# returned TRUE" and "PMv2 is in force" are different facts. So the call's
# result is discarded and GetThreadDpiAwarenessContext is read back and
# printed into the artifact; a readback that is not PMv2 aborts the run rather
# than producing coordinates nobody can trust.
#
# GetDpiForWindow and GetWindowDpiAwarenessContext are NOT virtualized, so the
# level and DPI readbacks would survive an undeclared caller. The coordinates
# would not, which is the whole reason for the abort.
#
# ── Why the capture is of the CLIENT rectangle ───────────────────────────────
#
# Control B compares an aware run against an unaware one. Their non-client
# frames are sized by different metrics (47 physical rows of caption at 120 DPI
# against 39 logical rows stretched to about 49), so equal *outer* rectangles
# do not give equal client rectangles and the two captures would not even have
# the same dimensions. Capturing the client area makes both sides exactly the
# controlled size and removes the frame from the comparison. CopyFromScreen,
# not PrintWindow -- Composition content reads back blank under PrintWindow
# (docs/notes/verification-environments.md Observation 4).
#
# ── Parameters ───────────────────────────────────────────────────────────────
#
#   -Tag        output directory under this folder; also the record's name
#   -Exe        host executable; defaults to the repo's release gallery-rust
#   -Unaware    run the child with __COMPAT_LAYER=DPIUNAWARE, so the AppCompat
#               shim sets the process awareness before Wasamo code runs and
#               runtime::init's declaration takes ERROR_ACCESS_DENIED. This is
#               a posture change to the SHIPPED bytes, not a source mutation.
#   -ClientW    target *physical* client width; 0 leaves the created size alone
#   -ClientH    target *physical* client height
#
# Requires a build produced by `cargo build -p wasamo-runtime --release` then
# `cargo build --release --workspace` (T1 finding F-5, T3 finding F-21: a
# host-package build relinks wasamo.dll around a stale uplifted rlib, silently
# and green, with a fresh DLL timestamp).
param(
  [Parameter(Mandatory = $true)][string]$Tag,
  [string]$Exe,
  [switch]$Unaware,
  [int]$ClientW = 0,
  [int]$ClientH = 0
)

$ErrorActionPreference = "Stop"

if (-not ('WinT10' -as [type])) {
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class WinT10 {
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
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int hgt, bool repaint);
    [DllImport("user32.dll")] public static extern IntPtr MonitorFromWindow(IntPtr h, uint flags);
    [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(POINT p);
    [DllImport("shcore.dll")] public static extern int GetDpiForMonitor(IntPtr mon, int type, out uint dx, out uint dy);
}
'@
}

function AwarenessName([IntPtr]$ctx) {
  if ([WinT10]::AreDpiAwarenessContextsEqual($ctx, [IntPtr](-4))) { return "PER_MONITOR_AWARE_V2" }
  if ([WinT10]::AreDpiAwarenessContextsEqual($ctx, [IntPtr](-3))) { return "PER_MONITOR_AWARE_V1" }
  if ([WinT10]::AreDpiAwarenessContextsEqual($ctx, [IntPtr](-2))) { return "SYSTEM_AWARE" }
  if ([WinT10]::AreDpiAwarenessContextsEqual($ctx, [IntPtr](-1))) { return "UNAWARE" }
  return "UNRECOGNISED($ctx)"
}

# Attempt, discard the result, read back what is actually in force (F-49).
[void][WinT10]::SetProcessDpiAwarenessContext([IntPtr](-4))
$harness = AwarenessName ([WinT10]::GetThreadDpiAwarenessContext())
if ($harness -ne "PER_MONITOR_AWARE_V2") {
  throw "harness awareness readback is $harness, not PER_MONITOR_AWARE_V2 — every coordinate this script would print is virtualized (F-48). Aborting rather than recording numbers that are wrong by exactly the system scale."
}

Add-Type -AssemblyName System.Drawing

$repo = (Resolve-Path "$PSScriptRoot\..\..\..\..\..").Path
if (-not $Exe) { $Exe = Join-Path $repo "target\release\gallery-rust.exe" }
if (-not (Test-Path $Exe)) { throw "missing $Exe" }

$outDir = Join-Path $PSScriptRoot $Tag
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$record = Join-Path $outDir "measurements.txt"
$lines = New-Object System.Collections.Generic.List[string]
function Rec([string]$s) { Write-Host $s; $lines.Add($s) }

Rec ("tag                : {0}" -f $Tag)
Rec ("captured           : {0}" -f (Get-Date -Format "yyyy-MM-dd HH:mm:ss"))
Rec ("harness awareness  : {0}  (READ BACK, not the call's return value)" -f $harness)
Rec ("host executable    : {0}" -f $Exe)
Rec ("host build stamp   : {0}" -f (Get-Item $Exe).LastWriteTime.ToString("yyyy-MM-dd HH:mm:ss"))
Rec ("__COMPAT_LAYER     : {0}" -f $(if ($Unaware) { "DPIUNAWARE (AppCompat sets process awareness before Wasamo code runs)" } else { "(unset)" }))

if ($Unaware) { $env:__COMPAT_LAYER = "DPIUNAWARE" } else { Remove-Item Env:__COMPAT_LAYER -ErrorAction SilentlyContinue }
try {
  $proc = Start-Process -FilePath $Exe -PassThru
  $hwnd = [IntPtr]::Zero
  for ($i = 0; $i -lt 60 -and $hwnd -eq [IntPtr]::Zero; $i++) {
    Start-Sleep -Milliseconds 250
    $proc.Refresh()
    if ($proc.HasExited) { throw "host exited early (code $($proc.ExitCode))" }
    $hwnd = $proc.MainWindowHandle
  }
  if ($hwnd -eq [IntPtr]::Zero) { throw "no MainWindowHandle after 15s" }
}
finally { Remove-Item Env:__COMPAT_LAYER -ErrorAction SilentlyContinue }

# `[ref]` on a PowerShell variable holding a boxed struct rebinds the variable
# rather than mutating the box, so the rects are re-read into script scope and
# every reader goes through $script:WR / $script:CR. Keeping a second local
# copy would silently serve the first measurement forever.
$script:HWND = $hwnd
$script:WR = New-Object WinT10+RECT
$script:CR = New-Object WinT10+RECT
function ReadRects {
  $w = New-Object WinT10+RECT
  $c = New-Object WinT10+RECT
  [void][WinT10]::GetWindowRect($script:HWND, [ref]$w)
  [void][WinT10]::GetClientRect($script:HWND, [ref]$c)
  $script:WR = $w
  $script:CR = $c
}
function CurOuterW { $script:WR.Right - $script:WR.Left }
function CurOuterH { $script:WR.Bottom - $script:WR.Top }
function CurClientW { $script:CR.Right - $script:CR.Left }
function CurClientH { $script:CR.Bottom - $script:CR.Top }

try {
  ReadRects
  Rec ("created outer      : {0}x{1}" -f (CurOuterW), (CurOuterH))
  Rec ("created client     : {0}x{1}  (physical)" -f (CurClientW), (CurClientH))

  $iterations = 0
  $residual = "n/a (created size preserved)"
  if ($ClientW -gt 0 -and $ClientH -gt 0) {
    # Drive the *client* rectangle, not the outer one. The non-client frame is
    # sized by DPI-indexed system metrics that differ between the aware and the
    # unaware run (T4 finding F-28), so equalising the outer rectangle would
    # leave the two layouts about 1.6 DIP apart. Measure and adjust rather than
    # computing the frame, because the unaware window's client is quantised in
    # whole *logical* pixels and the arithmetic does not always land.
    $ow = CurOuterW
    $oh = CurOuterH
    for ($iterations = 1; $iterations -le 10; $iterations++) {
      $ow += $ClientW - (CurClientW)
      $oh += $ClientH - (CurClientH)
      [void][WinT10]::MoveWindow($hwnd, 0, 0, $ow, $oh, $true)
      Start-Sleep -Milliseconds 350
      ReadRects
      if ((CurClientW) -eq $ClientW -and (CurClientH) -eq $ClientH) { break }
    }
    $residual = "{0}x{1} against target {2}x{3}" -f `
      ((CurClientW) - $ClientW), ((CurClientH) - $ClientH), $ClientW, $ClientH
    # The loop can also fall out of its bound without converging, in which case
    # `$iterations` is 11 and every earlier version of this line still said
    # "reached in 11 iteration(s)" — asserting success on the one path where the
    # target was missed, next to a residual line that would have said otherwise.
    # A frame at the wrong client size is not comparable against one at the
    # right size, so this fails the run rather than recording it.
    # **This branch has never executed**: every capture in the T10 evidence set
    # converged in 1 iteration. Named rather than predicted harmless (T9 F-49).
    if ((CurClientW) -ne $ClientW -or (CurClientH) -ne $ClientH) {
      Rec ("client target      : {0}x{1} physical — NOT REACHED after {2} iterations" -f $ClientW, $ClientH, $iterations)
      Rec ("client residual    : {0}" -f $residual)
      Set-Content -Path $record -Value $lines
      throw "client did not converge to ${ClientW}x${ClientH}; residual $residual. A frame at a different client size is not comparable, so no capture was taken."
    }
    Rec ("client target      : {0}x{1} physical, reached in {2} iteration(s)" -f $ClientW, $ClientH, $iterations)
    Rec ("client residual    : {0}" -f $residual)
  }

  [void][WinT10]::SetWindowPos($hwnd, [IntPtr](-1), 0, 0, 0, 0, 0x0043)  # HWND_TOPMOST | NOMOVE|NOSIZE|SHOWWINDOW
  [void][WinT10]::SetForegroundWindow($hwnd)
  Start-Sleep -Milliseconds 1500
  ReadRects

  $winLevel = AwarenessName ([WinT10]::GetWindowDpiAwarenessContext($hwnd))
  $winDpi = [WinT10]::GetDpiForWindow($hwnd)
  $mon = [WinT10]::MonitorFromWindow($hwnd, 2)
  $mdx = 0; $mdy = 0
  [void][WinT10]::GetDpiForMonitor($mon, 0, [ref]$mdx, [ref]$mdy)

  $ow = CurOuterW; $oh = CurOuterH
  $cw = CurClientW; $ch = CurClientH

  # What extent did the *layout engine* receive, in the units it works in?
  # An aware host divides the physical client by its own window DPI. An
  # unaware host is handed a client the OS already divided by the monitor's
  # scale. Both denominators are the monitor scale here; they are written out
  # separately so the equality is visible rather than assumed.
  $scale = if ($winLevel -eq "UNAWARE") { $mdx / 96.0 } else { $winDpi / 96.0 }
  $denom = if ($winLevel -eq "UNAWARE") { "monitor DPI $mdx (OS virtualization)" } else { "window DPI $winDpi (runtime division)" }

  Rec ("window awareness   : {0}" -f $winLevel)
  Rec ("GetDpiForWindow    : {0}" -f $winDpi)
  Rec ("monitor DPI        : {0}" -f $mdx)
  Rec ("final outer        : {0}x{1}  (physical)" -f $ow, $oh)
  Rec ("final client       : {0}x{1}  (physical)" -f $cw, $ch)
  Rec ("layout extent      : {0} x {1}  via {2}" -f ($cw / $scale), ($ch / $scale), $denom)
  Rec ("non-client frame   : {0} wide, {1} tall  (physical)" -f ($ow - $cw), ($oh - $ch))

  # Client rectangle in screen coordinates. The harness is PMv2, so this is
  # physical for both the aware and the unaware window.
  $origin = New-Object WinT10+POINT
  $origin.X = 0; $origin.Y = 0
  [void][WinT10]::ClientToScreen($hwnd, [ref]$origin)
  Rec ("client origin      : ({0},{1})  (screen, physical)" -f $origin.X, $origin.Y)

  # `CopyFromScreen` photographs a screen rectangle, not a window: anything that
  # got in front of the host between the topmost call and the capture would be
  # in the frame, and the artifact would not say so. Ask the OS what actually
  # owns four interior points and fail rather than record a photograph of
  # something else.
  $probes = @(@(0.1, 0.1), @(0.9, 0.1), @(0.1, 0.9), @(0.9, 0.9))
  foreach ($f in $probes) {
    $pt = New-Object WinT10+POINT
    $pt.X = $origin.X + [int]($cw * $f[0]); $pt.Y = $origin.Y + [int]($ch * $f[1])
    $owner = [WinT10]::WindowFromPoint($pt)
    if ($owner -ne $hwnd) {
      throw "the point ($($pt.X),$($pt.Y)) inside the client belongs to window $owner, not the host's $hwnd — something is in front of it. Refusing to record a frame of another window."
    }
  }
  Rec ("occlusion check    : 4 interior points, all owned by the host window")

  $bmp = New-Object System.Drawing.Bitmap $cw, $ch
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($origin.X, $origin.Y, 0, 0, (New-Object System.Drawing.Size($cw, $ch)))
  $out = Join-Path $outDir "gallery-client.png"
  $bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
  $g.Dispose(); $bmp.Dispose()
  Rec ("frame              : {0}  ({1}x{2})" -f $out, $cw, $ch)

  $proc.Refresh()
  Rec ("alive after capture: {0}" -f (-not $proc.HasExited))
}
finally {
  if ($null -ne $proc -and -not $proc.HasExited) { Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue }
}

Set-Content -Path $record -Value $lines
Write-Host "  -> $record"
