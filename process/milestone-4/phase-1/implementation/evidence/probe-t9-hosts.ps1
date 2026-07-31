# T9 — the three-host artifact for DD-M4-P1-001's declarative-host boundary
# claim (preamble obligation 6, risk R-8).
#
# Launches each example host and asks what DPI awareness level is in force in
# that host's own process, plus its realised window and client rectangles.
#
# WHY THE PROBE DECLARES ITS OWN AWARENESS (T9 finding F-48, measured).
# A DPI-unaware process asking GetWindowRect / GetClientRect about an *aware*
# window is answered in virtualized coordinates -- the real rectangle divided
# by the system scale. The first run of this script did not declare, reported
# outer=800x600 for windows that are really 1000x750, and would have supported
# the conclusion that T4's DIP-size correction never ran. The reading was
# internally consistent and wrong by exactly the scale factor. GetDpiForWindow
# and GetWindowDpiAwarenessContext are NOT virtualized, so the level readback
# was correct in both runs; it is the coordinates that moved.
#
# Prerequisites, in this order (finding F-21: a host-package build relinks
# wasamo.dll around a stale uplifted rlib, silently and green):
#   cargo build -p wasamo-runtime --release
#   cargo build --release --workspace
#   examples/counter-c:   cmake -S . -B build; cmake --build build --config Release
#   examples/counter-zig: zig build
#   $env:Path = "<repo>\target\release;" + $env:Path      # wasamo.dll
#
# Recorded result, 2026-07-31, development machine at 125%:
#   counter-c      level=PER_MONITOR_AWARE_V2  GetDpiForWindow=120  outer=1000x750  client=982x703
#   counter-rust   level=PER_MONITOR_AWARE_V2  GetDpiForWindow=120  outer=1000x750  client=982x703
#   counter-zig    level=PER_MONITOR_AWARE_V2  GetDpiForWindow=120  outer=1000x750  client=982x703
#
# The LEVEL is the falsifier, not the rectangle: T1 finding F-9 measured that an
# unaware process also produces 1000x750, because DWM stretches the logical
# rectangle by the same factor. "All three hosts still build and run" is a claim
# three unaware processes would also satisfy. The rectangle is a second fact of
# a different shape beside it, and it agrees with T4's independent measurement
# of 982 x 703.

Add-Type @"
using System;
using System.Runtime.InteropServices;
public class W {
  [DllImport("user32.dll")] public static extern IntPtr GetWindowDpiAwarenessContext(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern bool AreDpiAwarenessContextsEqual(IntPtr a, IntPtr b);
  [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out RECT r);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr hwnd, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr ctx);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int left, top, right, bottom; }
  public static IntPtr UNAWARE = new IntPtr(-1);
  public static IntPtr SYSTEM  = new IntPtr(-2);
  public static IntPtr PMV1    = new IntPtr(-3);
  public static IntPtr PMV2    = new IntPtr(-4);
}
"@

function Probe([string]$Label, [string]$Exe, [string]$WorkDir) {
  $p = Start-Process -FilePath $Exe -WorkingDirectory $WorkDir -PassThru
  $hwnd = [IntPtr]::Zero
  for ($i = 0; $i -lt 60 -and $hwnd -eq [IntPtr]::Zero; $i++) {
    Start-Sleep -Milliseconds 250
    $p.Refresh()
    if ($p.HasExited) { break }
    $hwnd = $p.MainWindowHandle
  }
  if ($p.HasExited) {
    Write-Output "$Label : PROCESS EXITED EARLY (exit code $($p.ExitCode))"
    return
  }
  if ($hwnd -eq [IntPtr]::Zero) {
    Write-Output "$Label : no main window after 15s"
    $p.Kill(); return
  }
  $ctx = [W]::GetWindowDpiAwarenessContext($hwnd)
  $lvl = if ([W]::AreDpiAwarenessContextsEqual($ctx, [W]::PMV2))    { "PER_MONITOR_AWARE_V2" }
    elseif ([W]::AreDpiAwarenessContextsEqual($ctx, [W]::PMV1))     { "PER_MONITOR_AWARE_V1" }
    elseif ([W]::AreDpiAwarenessContextsEqual($ctx, [W]::SYSTEM))   { "SYSTEM_AWARE" }
    elseif ([W]::AreDpiAwarenessContextsEqual($ctx, [W]::UNAWARE))  { "UNAWARE" }
    else { "UNRECOGNISED" }
  $dpi = [W]::GetDpiForWindow($hwnd)
  $wr = New-Object W+RECT; $cr = New-Object W+RECT
  [void][W]::GetWindowRect($hwnd, [ref]$wr); [void][W]::GetClientRect($hwnd, [ref]$cr)
  Start-Sleep -Milliseconds 1500
  $p.Refresh()
  $alive = -not $p.HasExited
  Write-Output ("{0,-14} level={1,-22} GetDpiForWindow={2}  outer={3}x{4}  client={5}x{6}  alive_after_probe={7}" -f `
     $Label, $lvl, $dpi, ($wr.right-$wr.left), ($wr.bottom-$wr.top), ($cr.right-$cr.left), ($cr.bottom-$cr.top), $alive)
  if ($alive) { $p.Kill() }
}

# See the header. Without this the coordinate readings below are virtualized.
Write-Output ("probe declared PMv2: " + [W]::SetProcessDpiAwarenessContext([W]::PMV2))

$root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))))
Probe "counter-c"    "$root\examples\counter-c\build\Release\counter.exe"      "$root\examples\counter-c\build\Release"
Probe "counter-rust" "$root\target\release\counter-rust.exe"                    "$root\target\release"
Probe "counter-zig"  "$root\examples\counter-zig\zig-out\bin\counter-zig.exe"  "$root\examples\counter-zig\zig-out\bin"
