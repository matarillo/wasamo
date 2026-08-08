# M4-Phase 2 T12 GUI evidence with positive controls (assistant-automated
# GUI evidence). One capture run drives the shipped `gallery-rust.exe`
# host through all four controls the framing's §検証方針 lists
# (process/milestone-4/phase-2/requirements/framing.md), each with a
# difference leg and an agreement leg, per docs/dsl_spec.md §4.19
# (click routing, propagation/consumption, focus traversal, modal focus
# scopes). Capture mechanics (PMv2 declared+read back, CopyFromScreen over
# the CLIENT rectangle, foreground earned and read back before any key,
# >=2 frames per set, scale recorded) follow
# docs/notes/verification-environments.md Observation 4; cited here, not
# restated.
#
# ── What each control discriminates, and what a WRONG implementation
#    would render that looks the same ─────────────────────────────────
#
# A — click routing and item identity (per-item handler binder reads).
#     A lightbox that opens from ANY thumbnail click and always shows
#     "Photo #0" renders one open frame indistinguishable from the
#     correct one. The DIFFERENCE leg (caption differs by which
#     thumbnail was clicked) is necessary; alone it is satisfiable by
#     ANY per-open variation (window jitter, a redraw artefact), so the
#     AGREEMENT leg (the SAME thumbnail clicked twice must render the
#     SAME caption) is what localises the difference to identity rather
#     than to noise. A second agreement leg (the `[photo]` placeholder
#     Box, which does not depend on `selected_index` this phase) shows
#     the difference is localised to the caption, not "the lightbox
#     repainted". T10 took this control first (evidence/t10-frames/),
#     and plan.md's T12 row leaves the choice between citing those
#     frames and re-taking them. This script re-takes -- not because
#     T10's are stale (the runtime diff since is T11's WM_POINTER* arms
#     alone, which no mouse or key path passes through) but so that all
#     four controls share one build, one launch, one window and one
#     measured scale. T10's set stands beside this one as an independent
#     earlier sitting.
#
# B — traversal order (`focus-group`, Tab/Shift+Tab). An implementation
#     that moves focus on EVERY Tab press but skips or duplicates a stop
#     would still show "something changed" at each step, indistinguishable
#     from correct traversal in a single before/after pair. What
#     separates them: each stop's painted change must be a DISJOINT,
#     LEFT-TO-RIGHT-INCREASING region (monotone + disjoint bboxes) --
#     catches a skipped or reordered stop; wrapping after the last stop
#     must return to exactly the first stop's pixels (an implementation
#     that paints the indicator but never clears the previous stop would
#     pass every difference leg and fail this one, DD-M4-P2-003's shape);
#     repeating the same walk must be deterministic; Shift+Tab must
#     reverse it. The toolbar-left group is one Tab stop
#     (`focus-group: true`), so B walks it as a single step, not three.
#
# C — containment and occlusion (`modal-scope`, DD-004, hit-testing
#     topmost-wins). A scrim that merely LOOKS opaque but does not
#     actually block hit-testing would still render an unchanged
#     background on a covered click -- indistinguishable from correct
#     containment in a single frame, UNLESS the click is checked to have
#     written no state either (`c-blocked` vs `c-closed`, whole client,
#     "checked in the clear") and the SAME coordinate is shown to fire
#     once the lightbox is closed (`c-fired` vs `c-closed`) -- a dead
#     button at that point would pass "does nothing" vacuously. The
#     SENSOR-FIRST leg (`c-openA` vs `c-openB`, toolbar band) exists
#     because the toolbar must be checked OBSERVABLE through the scrim
#     before "the covered click does nothing" can be trusted as
#     containment rather than as "the measuring apparatus can't see
#     anything back there anyway" -- see T10 close-gate's finding that
#     the blocker is the lightbox's own `Grid` (reverse-order hit-test
#     winner), not the scrim, which is exactly the distinction a sensor
#     check is for. Five Tabs inside the open scope must never reach the
#     toolbar (bounded) while visibly moving focus among `<`/`>`/`x`
#     (not inert); one Tab must reach the toolbar the instant the scope
#     is gone. This control also discharges T4's CF-T4-5 and T2's
#     deferred gallery-frame obligation (plan.md's T12 row).
#
# D — Esc (`dismiss`). An implementation that closes the lightbox on ANY
#     key would pass "Escape closes it" and never be caught by a single
#     frame. The discriminating leg is a RECOGNISED key with no handler
#     on this scope (VK_HOME) changing nothing -- before T8 an unrelated
#     key had no authored path to fire on at all, so this leg could not
#     have discriminated (T8 close-gate re-audit); it can now. The
#     RETURN-agreement leg (closed-after-Esc vs the pristine pre-open
#     state) is what stops "closed differs from open" from being
#     satisfiable by the lightbox merely becoming a different, still-open
#     wrong thing.
#
# ── Compare-side revision (post-capture, owner-dispositioned; frames are
#    unchanged from the original -Capture run) ─────────────────────────
#
# The 60-summed-channel `px_over_threshold` bar is calibrated against
# UNSCRIMMED changes (it exists to clear F-33's text jitter). Measured
# directly on this run's own frames: the toolbar's real checked-state
# swing (All -> Albums) is max_channel=157 unscrimmed but only
# max_channel=31 through the lightbox scrim (`fill: #101820cc`) -- a
# 5.06x attenuation that matches `1/(1-alpha)` for the scrim's own alpha
# (`cc` = 204/255 = 0.8) almost exactly. `px_differing_at_all` for the
# SAME comparison is unchanged by the scrim (2608 px either way) -- the
# scrim does not hide the signal, it divides its contrast by five, below
# the 60-summed bar. So: the two toolbar-band comparisons taken WHILE THE
# LIGHTBOX IS OPEN (the sensor, `c-openA` vs `c-openB`; and containment,
# `c-tab` vs `c-openB`) are judged on `px_differing_at_all` instead,
# against a floor measured from those sets' own frame0/frame1 jitter
# (measured 0). This is a TIGHTENING: the agreement bar moves from "<=40
# px over a 60-summed threshold" to "<=40 px differing by ANY amount",
# strictly harder to pass. Every other toolbar-band leg keeps
# `px_over_threshold`, untouched. `-Compare` prints the attenuation
# measurement and a conservativeness line (the focus indicator's own
# swing at the same face) beside the sensor/containment legs, so the
# metric switch's justification travels with it rather than living only
# in this comment.
#
# The identification leg (was: re-derive the checked-blue mask on
# `c-fired`) is replaced rather than repaired. Albums ends up BOTH
# checked and focused after the click (a real Button click moves
# keyboard focus, unlike a thumbnail `Box` click -- see Control A),
# so its rendered colour is the checked+focused blend
# (DD-M4-P2-003), which a predicate tuned on the checked+UNFOCUSED blue
# cannot find -- re-tuning the predicate would make the leg depend on a
# colour blend rather than on a behaviour. In its place: All's own bbox
# (never clicked, and never focused in this sequence -- focus moves from
# stop 2 to Albums, not through All) must lose its checked colour between
# `c-closed` and `c-fired`, which only the `clicked` handler mutating
# `tab_all_selected` can cause. A look-alike exclusion rules out the one
# thing that could otherwise explain a colour change at All's face
# without the handler running: focus landing there instead (ruled out by
# showing `c-fired`'s mean colour at All's bbox is far from `b1`'s own
# measured checked+focused blend at that same bbox).
#
# ── In-run guards ───────────────────────────────────────────────────
# After every lightbox-opening action, the whole-client diff against the
# immediately preceding closed frame must exceed cw*ch/20 (the lightbox
# actually opened); after every Escape, the whole-client diff against
# that same preceding closed frame must fall back below cw*ch/200 (it
# actually closed, and closed to the SAME place). A run that silently
# produced a wrong artifact -- a click that missed, a key that went
# nowhere -- throws immediately rather than saving a frame set that
# LOOKS like evidence.
#
# Every coordinate below is derived from examples/gallery/gallery.ui's
# own numbers (Grid track specs, HStack padding/spacing, WrapPanel
# item-cross-size/spacing) times the scale measured at capture time --
# T10's Thumb-Point pattern -- never a hand-worked-out pixel constant.
#
# Usage:
#   powershell.exe -NoProfile -STA -File capture-t12-controls.ps1 -Capture
#   powershell.exe -NoProfile -STA -File capture-t12-controls.ps1 -Compare
#   powershell.exe -NoProfile -STA -File capture-t12-controls.ps1 -SelfCheck
param(
  [switch]$Capture,
  [switch]$Compare,
  [switch]$SelfCheck,
  [string]$OutDir = (Join-Path $PSScriptRoot 't12-frames'),
  [string]$OutputPrefix = 't12-controls'
)

$ErrorActionPreference = "Stop"

if (-not $Capture -and -not $Compare -and -not $SelfCheck) {
  throw "pass -Capture, -Compare, and/or -SelfCheck"
}

if (-not ('WinT12Cap' -as [type])) {
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class WinT12Cap {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
    [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr ctx);
    [DllImport("user32.dll")] public static extern IntPtr GetThreadDpiAwarenessContext();
    [DllImport("user32.dll")] public static extern bool AreDpiAwarenessContextsEqual(IntPtr a, IntPtr b);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc cb, IntPtr lp);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr h, StringBuilder sb, int max);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
    [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
    [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr h, uint msg, IntPtr wp, IntPtr lp);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
}
'@
}

# Per-Monitor-Aware V2, declared AND verified: declaring is not evidence
# that the declaration took effect (Observation 4 / Phase 1 F-48).
$PMV2 = [IntPtr](-4)
[WinT12Cap]::SetProcessDpiAwarenessContext($PMV2) | Out-Null
if (-not [WinT12Cap]::AreDpiAwarenessContextsEqual([WinT12Cap]::GetThreadDpiAwarenessContext(), $PMV2)) {
  throw "capture tool is not Per-Monitor-Aware V2; every rectangle below would be virtualized"
}
Add-Type -AssemblyName System.Drawing
New-Item -ItemType Directory -Force $OutDir | Out-Null

# Every frame-set tag this script produces, B/A/D/C in the order captured.
$AllTags = @(
  "b-n", "b1", "b2", "b3", "b4", "b5", "b3b", "brev",
  "a0", "a3", "a0b",
  "d-pre", "d-open", "d-home", "d-closed",
  "c-closed", "c-openA", "c-openA-click", "c-blocked", "c-fired", "c-openB", "c-tab", "c-final", "c-tab-closed"
)

# ── Shared machinery (T10's proven shape) ──────────────────────────────

function Find-GalleryWindow($ProcessId) {
  $script:found = [IntPtr]::Zero
  [WinT12Cap]::EnumWindows({
    param($h, $l)
    $owner = 0
    [WinT12Cap]::GetWindowThreadProcessId($h, [ref]$owner) | Out-Null
    if ($owner -ne $ProcessId -or -not [WinT12Cap]::IsWindowVisible($h)) { return $true }
    $sb = [Text.StringBuilder]::new(256)
    [WinT12Cap]::GetWindowTextW($h, $sb, 256) | Out-Null
    if ($sb.ToString() -eq "Gallery") { $script:found = $h; return $false }
    return $true
  }, [IntPtr]::Zero) | Out-Null
  return $script:found
}

# T11 carry-forward CF-T11-5: a fixed wait bakes in the load of the
# machine it was written on. Poll instead, and throw with the elapsed
# time -- not a bare "not found" -- if the window never appears.
function Wait-ForGalleryWindow($ProcessId, $TimeoutSeconds = 20) {
  $start = Get-Date
  $deadline = $start.AddSeconds($TimeoutSeconds)
  while ((Get-Date) -lt $deadline) {
    $h = Find-GalleryWindow $ProcessId
    if ($h -ne [IntPtr]::Zero) { return $h }
    Start-Sleep -Milliseconds 500
  }
  $elapsed = ((Get-Date) - $start).TotalSeconds
  throw "no visible Gallery HWND after polling for $elapsed s (timeout ${TimeoutSeconds}s)"
}

# The client rectangle in screen coordinates: both corners mapped with
# ClientToScreen, never derived from GetWindowRect minus a guessed frame.
function Client-ScreenRect($Handle) {
  $c = New-Object WinT12Cap+RECT
  [WinT12Cap]::GetClientRect($Handle, [ref]$c) | Out-Null
  $tl = New-Object WinT12Cap+POINT; $tl.X = $c.Left;  $tl.Y = $c.Top
  $br = New-Object WinT12Cap+POINT; $br.X = $c.Right; $br.Y = $c.Bottom
  [WinT12Cap]::ClientToScreen($Handle, [ref]$tl) | Out-Null
  [WinT12Cap]::ClientToScreen($Handle, [ref]$br) | Out-Null
  return @{ X = $tl.X; Y = $tl.Y; W = $br.X - $tl.X; H = $br.Y - $tl.Y }
}

function Capture-Client($Handle) {
  $r = Client-ScreenRect $Handle
  if ($r.W -le 0 -or $r.H -le 0) { throw "invalid client rect $($r.W)x$($r.H)" }
  $bmp = New-Object System.Drawing.Bitmap $r.W, $r.H
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($r.X, $r.Y, 0, 0, (New-Object System.Drawing.Size($r.W, $r.H)))
  $g.Dispose()
  return $bmp
}

function Bitmap-Bytes($bmp) {
  $rect = New-Object System.Drawing.Rectangle 0, 0, $bmp.Width, $bmp.Height
  $data = $bmp.LockBits($rect, [System.Drawing.Imaging.ImageLockMode]::ReadOnly,
                        [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
  $len = [Math]::Abs($data.Stride) * $bmp.Height
  $buf = New-Object byte[] $len
  [System.Runtime.InteropServices.Marshal]::Copy($data.Scan0, $buf, 0, $len)
  $bmp.UnlockBits($data)
  return @{ Bytes = $buf; Stride = $data.Stride; W = $bmp.Width; H = $bmp.Height }
}

# A single un-saved capture, used only for in-run guard checks that have
# no corresponding named/saved frame set (Control A's post-Escape steps).
function Capture-Frame($Handle) {
  $bmp = Capture-Client $Handle
  $d = Bitmap-Bytes $bmp
  $bmp.Dispose()
  return $d
}

function Move-To($Handle, $ClientX, $ClientY) {
  $r = Client-ScreenRect $Handle
  [WinT12Cap]::SetCursorPos($r.X + $ClientX, $r.Y + $ClientY) | Out-Null
  Start-Sleep -Milliseconds 400
}

function Click-At($Handle, $ClientX, $ClientY) {
  Move-To $Handle $ClientX $ClientY
  [WinT12Cap]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)   # LEFTDOWN
  Start-Sleep -Milliseconds 60
  [WinT12Cap]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)   # LEFTUP
  Start-Sleep -Milliseconds 900   # let the click's drain + re-layout settle
}

# Foreground activation earned with a real click and read back, never
# requested and assumed (Observation 4 / T10's Try-Activate).
function Try-Activate($Handle, $ClientX, $ClientY, $Attempts = 5) {
  for ($i = 1; $i -le $Attempts; $i++) {
    Click-At $Handle $ClientX $ClientY
    [WinT12Cap]::SetForegroundWindow($Handle) | Out-Null
    Start-Sleep -Milliseconds 300
    if ([WinT12Cap]::GetForegroundWindow() -eq $Handle) {
      if ($i -gt 1) { Write-Host "foreground acquired on attempt $i" }
      return $true
    }
    Write-Host ("attempt {0}/{1}: foreground is {2}, retrying" -f $i, $Attempts, [WinT12Cap]::GetForegroundWindow())
    Start-Sleep -Milliseconds 700
  }
  return $false
}

# A key press through the strongest input path this environment supports
# (capture-t5-focus.ps1 / capture-t10-item-identity.ps1's shape). The path
# actually used is printed, because the frames look identical either way.
function Send-Key($Handle, [byte]$Vk) {
  if ($script:RealKeys) {
    if ([WinT12Cap]::GetForegroundWindow() -ne $Handle) {
      throw "the Gallery window lost foreground mid-capture; a real key press would go somewhere else"
    }
    [WinT12Cap]::keybd_event($Vk, 0, 0, [UIntPtr]::Zero)       # key down
    Start-Sleep -Milliseconds 40
    [WinT12Cap]::keybd_event($Vk, 0, 2, [UIntPtr]::Zero)       # KEYEVENTF_KEYUP
  } else {
    [WinT12Cap]::PostMessageW($Handle, 0x0100, [IntPtr]$Vk, [IntPtr]0) | Out-Null  # WM_KEYDOWN
  }
  Start-Sleep -Milliseconds 900
}

function Send-Tab($Handle) { Send-Key $Handle 0x09 }
function Send-Escape($Handle) { Send-Key $Handle 0x1B }

# Shift+Tab. Under real keys: keybd_event(VK_SHIFT down), Tab down/up,
# keybd_event(VK_SHIFT up) -- all real OS input. Under the posted-message
# fallback, WM_KEYDOWN VK_TAB is POSTED (the weaker claim, as every other
# key in this script), but the shift MODIFIER state is still set with a
# real keybd_event either way, because PostMessageW cannot carry modifier
# key state -- and this is said in the output, not left silent.
function Send-ShiftTab($Handle) {
  if ($script:RealKeys) {
    if ([WinT12Cap]::GetForegroundWindow() -ne $Handle) {
      throw "the Gallery window lost foreground mid-capture; a real Shift+Tab would go somewhere else"
    }
    [WinT12Cap]::keybd_event(0x10, 0, 0, [UIntPtr]::Zero)   # VK_SHIFT down
    Start-Sleep -Milliseconds 40
    [WinT12Cap]::keybd_event(0x09, 0, 0, [UIntPtr]::Zero)   # VK_TAB down
    Start-Sleep -Milliseconds 40
    [WinT12Cap]::keybd_event(0x09, 0, 2, [UIntPtr]::Zero)   # VK_TAB up
    Start-Sleep -Milliseconds 40
    [WinT12Cap]::keybd_event(0x10, 0, 2, [UIntPtr]::Zero)   # VK_SHIFT up
  } else {
    Write-Host "  Shift+Tab via posted-message fallback: shift state set with a real keybd_event, VK_TAB itself posted as WM_KEYDOWN (weaker claim)"
    [WinT12Cap]::keybd_event(0x10, 0, 0, [UIntPtr]::Zero)   # VK_SHIFT down (real; PostMessage cannot carry modifier state)
    Start-Sleep -Milliseconds 40
    [WinT12Cap]::PostMessageW($Handle, 0x0100, [IntPtr]0x09, [IntPtr]0) | Out-Null  # WM_KEYDOWN VK_TAB
    Start-Sleep -Milliseconds 40
    [WinT12Cap]::keybd_event(0x10, 0, 2, [UIntPtr]::Zero)   # VK_SHIFT up
  }
  Start-Sleep -Milliseconds 900
}

# ── Pixel measurement (shared by in-run guards, -Compare, -SelfCheck) ──

# Three quantities over one rectangle, T11's correction reproduced here:
#   MaxChannel  - the largest single-channel delta anywhere in the region
#                 (what the F-33 tolerance, 13/channel measured, is
#                 checked against).
#   DiffAny     - pixels differing by ANY amount: the real noise floor.
#   DiffOver    - pixels whose summed |dR|+|dG|+|dB| exceeds 60 (F-33
#                 measured up to 13/channel = 39 summed; 60 is above the
#                 jitter and far below a glyph appearing). Verdicts use
#                 this one.
function Diff-Count($a, $b, $x0, $x1, $y0, $y1) {
  $maxC = 0; $diffAny = 0; $diffOver = 0
  for ($y = $y0; $y -lt $y1; $y++) {
    $row = $y * $a.Stride
    for ($x = $x0; $x -lt $x1; $x++) {
      $i = $row + $x * 4
      $db = [Math]::Abs([int]$a.Bytes[$i]   - [int]$b.Bytes[$i])
      $dg = [Math]::Abs([int]$a.Bytes[$i+1] - [int]$b.Bytes[$i+1])
      $dr = [Math]::Abs([int]$a.Bytes[$i+2] - [int]$b.Bytes[$i+2])
      $m = [Math]::Max($db, [Math]::Max($dg, $dr))
      if ($m -gt $maxC) { $maxC = $m }
      $s = $db + $dg + $dr
      if ($s -gt 0) { $diffAny++ }
      if ($s -gt 60) { $diffOver++ }
    }
  }
  return @{ MaxChannel = $maxC; DiffAny = $diffAny; DiffOver = $diffOver }
}

# A region is an ARRAY of rectangles (the lightbox side-control columns
# are two disjoint rectangles summed; every other region is a one-element
# array), so every leg routes through this one function regardless of
# region shape.
function Measure-Region($A, $B, $Region) {
  $maxC = 0; $diffAny = 0; $diffOver = 0
  foreach ($r in $Region) {
    $m = Diff-Count $A $B $r.X0 $r.X1 $r.Y0 $r.Y1
    if ($m.MaxChannel -gt $maxC) { $maxC = $m.MaxChannel }
    $diffAny += $m.DiffAny
    $diffOver += $m.DiffOver
  }
  return @{ MaxChannel = $maxC; DiffAny = $diffAny; DiffOver = $diffOver }
}

# The pixels over the visible-change threshold, plus their bbox and
# centroid_x -- used only for Control B's monotone/disjoint checks, over
# a single rectangle.
function Measure-OverThreshold($a, $b, $x0, $x1, $y0, $y1) {
  $n = 0; $minX = [int]::MaxValue; $maxX = -1; $sumX = 0.0
  for ($y = $y0; $y -lt $y1; $y++) {
    $row = $y * $a.Stride
    for ($x = $x0; $x -lt $x1; $x++) {
      $i = $row + $x * 4
      $s = [Math]::Abs([int]$a.Bytes[$i] - [int]$b.Bytes[$i]) +
           [Math]::Abs([int]$a.Bytes[$i+1] - [int]$b.Bytes[$i+1]) +
           [Math]::Abs([int]$a.Bytes[$i+2] - [int]$b.Bytes[$i+2])
      if ($s -gt 60) {
        $n++; $sumX += $x
        if ($x -lt $minX) { $minX = $x }
        if ($x -gt $maxX) { $maxX = $x }
      }
    }
  }
  $cx = if ($n -gt 0) { $sumX / $n } else { $null }
  return @{ Count = $n; MinX = $minX; MaxX = $maxX; CentroidX = $cx }
}

# capture-t5-focus.ps1's checked "All" ToggleButton predicate, over the
# top 56-DIP toolbar band, left half only.
function Find-BlueMask($d, $scale) {
  $bandH = [Math]::Min([int](56 * $scale), $d.H)
  $halfW = [int]($d.W / 2)
  $minX = [int]::MaxValue; $maxX = -1; $minY = [int]::MaxValue; $maxY = -1
  $count = 0
  for ($y = 0; $y -lt $bandH; $y++) {
    $row = $y * $d.Stride
    for ($x = 0; $x -lt $halfW; $x++) {
      $i = $row + $x * 4
      $bl = $d.Bytes[$i]; $gr = $d.Bytes[$i+1]; $rd = $d.Bytes[$i+2]
      if ($bl -gt ($rd + 60) -and $bl -gt 120 -and $gr -gt $rd) {
        $count++
        if ($x -lt $minX) { $minX = $x }
        if ($x -gt $maxX) { $maxX = $x }
        if ($y -lt $minY) { $minY = $y }
        if ($y -gt $maxY) { $maxY = $y }
      }
    }
  }
  return @{ Count = $count; MinX = $minX; MaxX = $maxX; MinY = $minY; MaxY = $maxY }
}

function Check-Monotone($xs) {
  for ($i = 0; $i -lt $xs.Count - 1; $i++) {
    if (-not ($xs[$i] -lt $xs[$i + 1])) { return $false }
  }
  return $true
}

function Check-Disjoint($bboxes) {
  for ($i = 0; $i -lt $bboxes.Count - 1; $i++) {
    if (-not ($bboxes[$i].MaxX -lt $bboxes[$i + 1].MinX)) { return $false }
  }
  return $true
}

# Mean R/G/B over a region (an array of rectangles, T5's Mask-Mean
# pattern) -- used only by the identification leg's look-alike exclusion,
# which compares absolute colour, not a pixel-count diff.
function Mean-Region($d, $Region) {
  $sr = 0.0; $sg = 0.0; $sb = 0.0; $n = 0
  foreach ($r in $Region) {
    for ($y = $r.Y0; $y -lt $r.Y1; $y++) {
      $row = $y * $d.Stride
      for ($x = $r.X0; $x -lt $r.X1; $x++) {
        $i = $row + $x * 4
        $sb += $d.Bytes[$i]; $sg += $d.Bytes[$i + 1]; $sr += $d.Bytes[$i + 2]
        $n++
      }
    }
  }
  return @{ R = $sr / $n; G = $sg / $n; B = $sb / $n; N = $n }
}

function MaxChannelDelta($m1, $m2) {
  return [Math]::Max([Math]::Abs($m1.R - $m2.R), [Math]::Max([Math]::Abs($m1.G - $m2.G), [Math]::Abs($m1.B - $m2.B)))
}

# ── The three verdict functions every leg (and every self-check) routes
#    through ─────────────────────────────────────────────────────────
# $Metric selects which of the three quantities GOVERNS THE VERDICT; all
# three are always printed regardless (T11's rule). "Over" (default,
# px_over_threshold) is used everywhere except the two toolbar-band
# comparisons taken while the lightbox is open, where "Any"
# (px_differing_at_all) is used instead -- see the header's
# "Compare-side revision" section for why.
function Assert-Differs([string]$Label, $A, $B, $Region, $Limit, [string]$Metric = "Over") {
  $m = Measure-Region $A $B $Region
  $val = if ($Metric -eq "Any") { $m.DiffAny } else { $m.DiffOver }
  $metricName = if ($Metric -eq "Any") { "px_differing_at_all" } else { "px_over_threshold" }
  $pass = $val -gt $Limit
  $v = if ($pass) { "PASS" } else { "FAIL" }
  Write-Host ("DIFFERENCE {0,-70} max_channel={1,3} px_differing_at_all={2,7} px_over_threshold={3,7} [verdict: {4} >{5}] -> {6}" -f $Label, $m.MaxChannel, $m.DiffAny, $m.DiffOver, $metricName, $Limit, $v)
  return $pass
}

function Assert-Agrees([string]$Label, $A, $B, $Region, $Limit, [string]$Metric = "Over") {
  $m = Measure-Region $A $B $Region
  $val = if ($Metric -eq "Any") { $m.DiffAny } else { $m.DiffOver }
  $metricName = if ($Metric -eq "Any") { "px_differing_at_all" } else { "px_over_threshold" }
  $pass = $val -lt $Limit
  $v = if ($pass) { "PASS" } else { "FAIL" }
  Write-Host ("AGREEMENT  {0,-70} max_channel={1,3} px_differing_at_all={2,7} px_over_threshold={3,7} [verdict: {4} <{5}] -> {6}" -f $Label, $m.MaxChannel, $m.DiffAny, $m.DiffOver, $metricName, $Limit, $v)
  return $pass
}

# The look-alike exclusion: are two MEAN colours far apart (not a
# pixel-count diff over two frames, a colour-distance check between two
# already-computed region means). Same print-then-return-bool shape as
# the two functions above, so -SelfCheck can exercise it identically.
function Assert-MeansDiffer([string]$Label, $MeanA, $MeanB, $MinDelta) {
  $d = MaxChannelDelta $MeanA $MeanB
  $pass = $d -gt $MinDelta
  $v = if ($pass) { "PASS" } else { "FAIL" }
  Write-Host ("EXCLUDE    {0,-70} A=(R={1:N1},G={2:N1},B={3:N1}) B=(R={4:N1},G={5:N1},B={6:N1}) max_channel_delta={7:N1} (limit >{8}) -> {9}" -f $Label, $MeanA.R, $MeanA.G, $MeanA.B, $MeanB.R, $MeanB.G, $MeanB.B, $d, $MinDelta, $v)
  return $pass
}

# ── Regions, derived from the lightbox Grid's own track spec
#    (columns: 1* 56 400 56 1*, rows: 1* 44 300 64 1*) and the toolbar
#    row's own height (rows: 56 1* 28), times the measured scale ────────
function Build-Regions($cw, $ch, $scale) {
  $toolbarY1 = [Math]::Min([int](56 * $scale), $ch)
  $toolbar = @(@{ X0 = 0; X1 = $cw; Y0 = 0; Y1 = $toolbarY1 })

  $fixedColW = (56 + 400 + 56) * $scale
  $flexColW = ($cw - $fixedColW) / 2.0
  $col2x0 = [int]($flexColW + 56 * $scale)
  $col2x1 = [int]($flexColW + (56 + 400) * $scale)

  $fixedRowH = (44 + 300 + 64) * $scale
  $flexRowH = ($ch - $fixedRowH) / 2.0
  $photoY0 = [int]($flexRowH + 44 * $scale)
  $photoY1 = [int]($flexRowH + (44 + 300) * $scale)
  $capY0 = [int]($flexRowH + (44 + 300) * $scale)
  $capY1 = [int]($flexRowH + (44 + 300 + 64) * $scale)

  $photo = @(@{ X0 = $col2x0; X1 = $col2x1; Y0 = $photoY0; Y1 = $photoY1 })
  $caption = @(@{ X0 = $col2x0; X1 = $col2x1; Y0 = $capY0; Y1 = $capY1 })

  $sideX0a = [int]$flexColW
  $sideX1a = [int]($flexColW + 56 * $scale)
  $sideX0b = [int]($flexColW + (56 + 400) * $scale)
  $sideX1b = [int]($flexColW + (56 + 400 + 56) * $scale)
  $sideY0 = [int]$flexRowH
  $sideY1 = [int]($flexRowH + (44 + 300 + 64) * $scale)
  $side = @(
    @{ X0 = $sideX0a; X1 = $sideX1a; Y0 = $sideY0; Y1 = $sideY1 },
    @{ X0 = $sideX0b; X1 = $sideX1b; Y0 = $sideY0; Y1 = $sideY1 }
  )

  $whole = @(@{ X0 = 0; X1 = $cw; Y0 = 0; Y1 = $ch })

  return @{ Toolbar = $toolbar; Photo = $photo; Caption = $caption; Side = $side; Whole = $whole }
}

# The bbox/centroid stats for Control B's four Tab stops, shared by
# -Compare (which prints and asserts them) and -SelfCheck (which builds
# a deliberately wrong monotone/disjoint input from the SAME real stats).
function Get-BStopStats($ctx) {
  $r = $ctx.Regions.Toolbar[0]
  $centroids = @{}; $bboxes = @{}
  for ($k = 1; $k -le 4; $k++) {
    $stats = Measure-OverThreshold $ctx.Data["b$k-0"] $ctx.Data["b-n-0"] $r.X0 $r.X1 $r.Y0 $r.Y1
    $centroids[$k] = $stats.CentroidX
    $bboxes[$k] = @{ MinX = $stats.MinX; MaxX = $stats.MaxX }
  }
  return @{ Centroids = $centroids; Bboxes = $bboxes }
}

# ── Capture mode ────────────────────────────────────────────────────────

function Do-Capture() {
  $runStart = Get-Date -Format "o"
  $repo = (Resolve-Path "$PSScriptRoot\..\..\..\..\..").Path
  $exe = Join-Path $repo "target\release\gallery-rust.exe"
  if (-not (Test-Path $exe)) { throw "missing $exe (run cargo build --release --workspace first)" }
  $commit = (& git -C $repo rev-parse HEAD).Trim()
  Write-Host "commit=$commit"

  $p = Start-Process -FilePath $exe -PassThru
  try {
    $h = Wait-ForGalleryWindow $p.Id 20
    Write-Host "Gallery HWND=$h found"

    [WinT12Cap]::ShowWindow($h, 9) | Out-Null
    [WinT12Cap]::SetWindowPos($h, [IntPtr](-1), 120, 120, 1000, 750, 0x0040) | Out-Null
    Start-Sleep -Milliseconds 1200

    $dpi = [WinT12Cap]::GetDpiForWindow($h)
    $scale = $dpi / 96.0
    $cr = Client-ScreenRect $h
    Write-Host "dpi=$dpi scale=$scale client=$($cr.W)x$($cr.H) px = $($cr.W/$scale)x$($cr.H/$scale) DIP at ($($cr.X),$($cr.Y))"

    $parkX = [int]($cr.W * 0.5)
    $parkY = [int]($cr.H - 28 * $scale - 30 * $scale)
    $scrimX = [int]($cr.W * 0.05)
    $scrimY = [int]($cr.H * 0.5)

    function Thumb-Point($i) {
      $x = [int]((12 + $i * 100 + 44) * $scale)
      $y = [int]((68 + 44) * $scale)
      return @($x, $y)
    }

    $script:RealKeys = Try-Activate $h $parkX $parkY
    if ($script:RealKeys) {
      Write-Host "input path: REAL KEY PRESSES (keybd_event); foreground activation acquired and read back"
    } else {
      Write-Host "input path: POSTED WM_KEYDOWN (weaker claim: bypasses the OS input queue); foreground could not be acquired"
    }

    function Move-Park() { Move-To $h $parkX $parkY }
    function Move-Scrim() { Move-To $h $scrimX $scrimY }

    function Save-Set([string]$Tag) {
      $frames = @()
      for ($i = 0; $i -lt 2; $i++) {
        $bmp = Capture-Client $h
        $out = Join-Path $OutDir "$OutputPrefix-$Tag-$i.png"
        $bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
        Write-Host "saved $out"
        $frames += , (Bitmap-Bytes $bmp)
        $bmp.Dispose()
        if ($i -eq 0) { Start-Sleep -Milliseconds 250 }
      }
      return $frames
    }

    $openLimit = [int]($cr.W * $cr.H / 20)
    $closedLimit = [int]($cr.W * $cr.H / 200)

    function Guard-Open([string]$Tag, $Frame, $PrecedingClosed) {
      $m = Diff-Count $Frame $PrecedingClosed 0 $cr.W 0 $cr.H
      Write-Host ("  guard(open) '$Tag' vs preceding closed: max_channel=$($m.MaxChannel) px_differing_at_all=$($m.DiffAny) px_over_threshold=$($m.DiffOver) (must exceed $openLimit)")
      if ($m.DiffOver -le $openLimit) {
        throw "in-run guard failed: opening the lightbox for '$Tag' only changed $($m.DiffOver) px against the preceding closed frame (limit >$openLimit) -- the lightbox may not have opened"
      }
    }

    function Guard-Closed([string]$Tag, $Frame, $PrecedingClosed) {
      $m = Diff-Count $Frame $PrecedingClosed 0 $cr.W 0 $cr.H
      Write-Host ("  guard(closed) '$Tag' vs preceding closed: max_channel=$($m.MaxChannel) px_differing_at_all=$($m.DiffAny) px_over_threshold=$($m.DiffOver) (must be below $closedLimit)")
      if ($m.DiffOver -ge $closedLimit) {
        throw "in-run guard failed: after Escape at '$Tag', $($m.DiffOver) px still differ from the preceding closed frame (limit <$closedLimit) -- the lightbox may not have closed to the same state"
      }
    }

    # ── Control B — traversal order ────────────────────────────────────
    Write-Host ""
    Write-Host "Control B -- traversal order"
    Move-Park
    $bn = Save-Set "b-n"

    Send-Tab $h; Move-Park; $null = Save-Set "b1"
    Send-Tab $h; Move-Park; $null = Save-Set "b2"
    Send-Tab $h; Move-Park; $null = Save-Set "b3"
    Send-Tab $h; Move-Park; $null = Save-Set "b4"
    Send-Tab $h; Move-Park; $null = Save-Set "b5"           # wraps
    Send-Tab $h; Send-Tab $h; Move-Park; $null = Save-Set "b3b"
    Send-ShiftTab $h; Move-Park; $lastClosed = (Save-Set "brev")[0]

    # ── Control A — click routing and item identity ────────────────────
    Write-Host ""
    Write-Host "Control A -- click routing and item identity"
    $pt0 = Thumb-Point 0
    Write-Host "clicking thumbnail 0 at client ($($pt0[0]),$($pt0[1])) px -> set 'a0'"
    Click-At $h $pt0[0] $pt0[1]
    Move-Scrim
    $a0 = (Save-Set "a0")[0]
    Guard-Open "a0" $a0 $lastClosed
    Send-Escape $h; Move-Park
    $prevClosed = $lastClosed
    $lastClosed = Capture-Frame $h
    Guard-Closed "a0-esc" $lastClosed $prevClosed

    $pt3 = Thumb-Point 3
    Write-Host "clicking thumbnail 3 at client ($($pt3[0]),$($pt3[1])) px -> set 'a3'"
    Click-At $h $pt3[0] $pt3[1]
    Move-Scrim
    $a3 = (Save-Set "a3")[0]
    Guard-Open "a3" $a3 $lastClosed
    Send-Escape $h; Move-Park
    $prevClosed = $lastClosed
    $lastClosed = Capture-Frame $h
    Guard-Closed "a3-esc" $lastClosed $prevClosed

    Write-Host "clicking thumbnail 0 at client ($($pt0[0]),$($pt0[1])) px -> set 'a0b'"
    Click-At $h $pt0[0] $pt0[1]
    Move-Scrim
    $a0b = (Save-Set "a0b")[0]
    Guard-Open "a0b" $a0b $lastClosed
    Send-Escape $h; Move-Park
    $prevClosed = $lastClosed
    $lastClosed = Capture-Frame $h
    Guard-Closed "a0b-esc" $lastClosed $prevClosed

    # ── Control D — Esc ─────────────────────────────────────────────────
    Write-Host ""
    Write-Host "Control D -- Esc"
    Move-Park
    $dPre = (Save-Set "d-pre")[0]
    $lastClosed = $dPre

    $pt1 = Thumb-Point 1
    Write-Host "clicking thumbnail 1 at client ($($pt1[0]),$($pt1[1])) px -> set 'd-open'"
    Click-At $h $pt1[0] $pt1[1]
    Move-Scrim
    $dOpen = (Save-Set "d-open")[0]
    Guard-Open "d-open" $dOpen $lastClosed

    Write-Host "sending VK_HOME (recognised key, no handler on this scope) -> set 'd-home'"
    Send-Key $h 0x24
    $null = Save-Set "d-home"   # cursor unchanged, still over scrim

    Send-Escape $h; Move-Park
    $dClosed = (Save-Set "d-closed")[0]
    Guard-Closed "d-closed" $dClosed $lastClosed
    $lastClosed = $dClosed

    # ── Control C — containment and occlusion ──────────────────────────
    Write-Host ""
    Write-Host "Control C -- containment and occlusion"
    Move-Park
    $cClosed = (Save-Set "c-closed")[0]
    $lastClosed = $cClosed

    $allMask = Find-BlueMask $cClosed $scale
    if ($allMask.Count -lt 200) {
      throw "checked 'All' ToggleButton blue not found on c-closed (mask=$($allMask.Count) px); cannot derive the Albums click point"
    }
    Write-Host "All mask: $($allMask.Count) px, bbox x[$($allMask.MinX)..$($allMask.MaxX)] y[$($allMask.MinY)..$($allMask.MaxY)]"
    # 8 DIP `spacing` past All's right edge, then 20 DIP into the Albums
    # button (HStack { spacing: 8px; padding: 8px } around the ToggleButtons).
    $albumsX = $allMask.MaxX + [int]((8 + 20) * $scale)
    $albumsY = [int](($allMask.MinY + $allMask.MaxY) / 2)
    Write-Host "derived Albums click point: ($albumsX,$albumsY) client px"

    $pt2 = Thumb-Point 2
    Write-Host "clicking thumbnail 2 at client ($($pt2[0]),$($pt2[1])) px -> set 'c-openA'"
    Click-At $h $pt2[0] $pt2[1]
    Move-Scrim
    $cOpenA = (Save-Set "c-openA")[0]
    Guard-Open "c-openA" $cOpenA $lastClosed

    Write-Host "clicking (albumsX,albumsY) WITH the lightbox open -> set 'c-openA-click'"
    Click-At $h $albumsX $albumsY
    Move-Scrim
    $null = Save-Set "c-openA-click"

    Send-Escape $h; Move-Park
    $cBlocked = (Save-Set "c-blocked")[0]
    Guard-Closed "c-blocked" $cBlocked $lastClosed
    $lastClosed = $cBlocked

    Write-Host "clicking (albumsX,albumsY) WITH the lightbox closed -> set 'c-fired'"
    Click-At $h $albumsX $albumsY
    Move-Park
    $cFired = (Save-Set "c-fired")[0]
    $lastClosed = $cFired   # still closed; Albums click does not open the lightbox

    Write-Host "clicking thumbnail 2 at client ($($pt2[0]),$($pt2[1])) px -> set 'c-openB'"
    Click-At $h $pt2[0] $pt2[1]
    Move-Scrim
    $cOpenB = (Save-Set "c-openB")[0]
    Guard-Open "c-openB" $cOpenB $lastClosed

    Send-Tab $h; Send-Tab $h; Send-Tab $h; Send-Tab $h; Send-Tab $h
    Move-Scrim
    $null = Save-Set "c-tab"

    Send-Escape $h; Move-Park
    $cFinal = (Save-Set "c-final")[0]
    Guard-Closed "c-final" $cFinal $lastClosed
    $lastClosed = $cFinal

    Send-Tab $h
    Move-Park
    $null = Save-Set "c-tab-closed"

    # ── Meta ─────────────────────────────────────────────────────────
    $metaPath = Join-Path $OutDir "$OutputPrefix-meta.txt"
    $metaLines = New-Object System.Collections.Generic.List[string]
    $metaLines.Add("scale=$scale")
    $metaLines.Add("client_w=$($cr.W)")
    $metaLines.Add("client_h=$($cr.H)")
    $metaLines.Add("real_keys=$($script:RealKeys)")
    $metaLines.Add("commit=$commit")
    $metaLines.Add("run_started=$runStart")
    $metaLines.Add("all_mask_min_x=$($allMask.MinX)")
    $metaLines.Add("all_mask_max_x=$($allMask.MaxX)")
    $metaLines.Add("all_mask_min_y=$($allMask.MinY)")
    $metaLines.Add("all_mask_max_y=$($allMask.MaxY)")
    $metaLines.Add("albums_x=$albumsX")
    $metaLines.Add("albums_y=$albumsY")
    $metaLines.Add("park_x=$parkX")
    $metaLines.Add("park_y=$parkY")
    $metaLines.Add("scrim_x=$scrimX")
    $metaLines.Add("scrim_y=$scrimY")
    Set-Content -Path $metaPath -Value $metaLines

    $shaLines = New-Object System.Collections.Generic.List[string]
    foreach ($tag in $AllTags) {
      for ($i = 0; $i -lt 2; $i++) {
        $f = "$OutputPrefix-$tag-$i.png"
        $hash = (Get-FileHash -Path (Join-Path $OutDir $f) -Algorithm SHA256).Hash
        $shaLines.Add("sha256 $f $hash")
      }
    }
    Add-Content -Path $metaPath -Value $shaLines
    Write-Host ""
    Write-Host "wrote $metaPath"
    Write-Host "capture complete: all in-run guards passed."
  } finally {
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
  }
}

# ── Shared context for -Compare and -SelfCheck ─────────────────────────

function Load-Context() {
  $metaPath = Join-Path $OutDir "$OutputPrefix-meta.txt"
  if (-not (Test-Path $metaPath)) { throw "missing $metaPath -- run -Capture first" }
  $meta = @{}
  foreach ($line in Get-Content $metaPath) {
    if ($line -eq "" -or $line -match '^sha256 ') { continue }
    $idx = $line.IndexOf('=')
    if ($idx -lt 0) { continue }
    $meta[$line.Substring(0, $idx)] = $line.Substring($idx + 1)
  }
  $scale = [double]$meta['scale']
  $cw = [int]$meta['client_w']
  $ch = [int]$meta['client_h']

  $bmps = New-Object System.Collections.Generic.List[object]
  $data = @{}
  foreach ($tag in $AllTags) {
    for ($i = 0; $i -lt 2; $i++) {
      $path = Join-Path $OutDir "$OutputPrefix-$tag-$i.png"
      if (-not (Test-Path $path)) { throw "missing $path -- run -Capture first" }
      $bmp = New-Object System.Drawing.Bitmap $path
      if ($bmp.Width -ne $cw -or $bmp.Height -ne $ch) {
        throw "$tag-$i is $($bmp.Width)x$($bmp.Height), expected ${cw}x${ch} -- frames are not comparable"
      }
      $bmps.Add($bmp)
      $data["$tag-$i"] = Bitmap-Bytes $bmp
    }
  }

  $regions = Build-Regions $cw $ch $scale

  # All's own bbox (measured, not derivable from .ui numbers alone --
  # read back from the mask Do-Capture already found and recorded), used
  # by the replacement identification leg.
  $allMinX = [int]$meta['all_mask_min_x']; $allMaxX = [int]$meta['all_mask_max_x']
  $allMinY = [int]$meta['all_mask_min_y']; $allMaxY = [int]$meta['all_mask_max_y']
  $regions.AllBbox = @(@{ X0 = $allMinX; X1 = $allMaxX + 1; Y0 = $allMinY; Y1 = $allMaxY + 1 })

  function Region-Noise($tags, $region, [string]$Metric = "Over") {
    $mx = 0
    foreach ($t in $tags) {
      $m = Measure-Region $data["$t-0"] $data["$t-1"] $region
      $val = if ($Metric -eq "Any") { $m.DiffAny } else { $m.DiffOver }
      if ($val -gt $mx) { $mx = $val }
    }
    return $mx
  }
  $toolbarTags = @("b-n", "b1", "b2", "b3", "b4", "b5", "b3b", "brev", "c-openA", "c-openB", "c-fired", "c-closed", "c-tab", "c-tab-closed", "c-final")
  $captionTags = @("a0", "a3", "a0b")
  $photoTags = @("a0", "a3")
  $wholeTags = @("d-home", "d-open", "d-closed", "d-pre", "c-openA-click", "c-openA", "c-blocked", "c-closed", "c-final", "c-fired")
  $sideTags = @("c-tab", "c-openB")
  # The two toolbar-band comparisons taken while the lightbox is open
  # (sensor, containment) -- judged on px_differing_at_all; see header.
  $toolbarOpenTags = @("c-openA", "c-openB", "c-tab")
  # The replacement identification leg's own region (All's bbox).
  $allBboxTags = @("c-fired", "c-closed")

  $nToolbar = Region-Noise $toolbarTags $regions.Toolbar
  $nCaption = Region-Noise $captionTags $regions.Caption
  $nPhoto = Region-Noise $photoTags $regions.Photo
  $nWhole = Region-Noise $wholeTags $regions.Whole
  $nSide = Region-Noise $sideTags $regions.Side
  $nToolbarOpenAny = Region-Noise $toolbarOpenTags $regions.Toolbar "Any"
  $nAllBbox = Region-Noise $allBboxTags $regions.AllBbox

  $bands = @{
    Toolbar         = [Math]::Max($nToolbar * 4, 40)
    Caption         = [Math]::Max($nCaption * 4, 40)
    Photo           = [Math]::Max($nPhoto * 4, 40)
    Side            = [Math]::Max($nSide * 4, 40)
    WholeAgree      = [Math]::Max($nWhole * 4, [int]($cw * $ch / 2000))
    WholeDiffer     = [int]($cw * $ch / 20)
    ToolbarOpenAny  = [Math]::Max($nToolbarOpenAny * 4, 40)
    AllBbox         = [Math]::Max($nAllBbox * 4, 40)
  }

  return @{
    Meta = $meta; Scale = $scale; Cw = $cw; Ch = $ch
    Data = $data; Bitmaps = $bmps; Regions = $regions
    NToolbar = $nToolbar; NCaption = $nCaption; NPhoto = $nPhoto; NWhole = $nWhole; NSide = $nSide
    NToolbarOpenAny = $nToolbarOpenAny; NAllBbox = $nAllBbox
    Bands = $bands
  }
}

# ── Compare mode ────────────────────────────────────────────────────────

function Do-Compare() {
  $ctx = Load-Context
  Write-Host "commit=$($ctx.Meta['commit'])"
  Write-Host "scale=$($ctx.Scale) client=$($ctx.Cw)x$($ctx.Ch) real_keys=$($ctx.Meta['real_keys'])"
  Write-Host ""
  Write-Host "noise floors (max px_over_threshold, frame0 vs frame1, per region):"
  Write-Host "  toolbar=$($ctx.NToolbar) caption=$($ctx.NCaption) photo=$($ctx.NPhoto) side=$($ctx.NSide) whole=$($ctx.NWhole)"
  Write-Host "  toolbar_open (px_differing_at_all, lightbox-open comparisons only)=$($ctx.NToolbarOpenAny)  all_bbox (px_over_threshold)=$($ctx.NAllBbox)"
  Write-Host "pass bands: toolbar=$($ctx.Bands.Toolbar) caption=$($ctx.Bands.Caption) photo=$($ctx.Bands.Photo) side=$($ctx.Bands.Side) whole_agree=$($ctx.Bands.WholeAgree) whole_differ=$($ctx.Bands.WholeDiffer) toolbar_open_any=$($ctx.Bands.ToolbarOpenAny) all_bbox=$($ctx.Bands.AllBbox)"

  $ok = $true

  # ── Control B -- traversal order (toolbar band) ──────────────────────
  Write-Host ""
  Write-Host "Control B -- traversal order (toolbar band)"
  Write-Host "within-set jitter, reported as the plan's own agreement leg ('two frames with no input agree within the measured jitter', F-33 measured tolerance up to 13/channel):"
  foreach ($t in @("b-n", "b1", "b2", "b3", "b4", "b5", "b3b", "brev")) {
    $r = Assert-Agrees "B two frames with no input agree within the measured jitter ($t)" $ctx.Data["$t-0"] $ctx.Data["$t-1"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar
    $ok = $ok -and $r
  }

  $bstats = Get-BStopStats $ctx
  for ($k = 1; $k -le 4; $k++) {
    $r = Assert-Differs "B stop $k painted" $ctx.Data["b$k-0"] $ctx.Data["b-n-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar
    $ok = $ok -and $r
    Write-Host ("  stop $k bbox x[$($bstats.Bboxes[$k].MinX)..$($bstats.Bboxes[$k].MaxX)] centroid_x=$($bstats.Centroids[$k])")
  }
  $monoOk = Check-Monotone @($bstats.Centroids[1], $bstats.Centroids[2], $bstats.Centroids[3], $bstats.Centroids[4])
  Write-Host "B monotone: stop centroids increase left-to-right -> $(if ($monoOk) { 'PASS' } else { 'FAIL' })"
  $ok = $ok -and $monoOk
  $disjointOk = Check-Disjoint @($bstats.Bboxes[1], $bstats.Bboxes[2], $bstats.Bboxes[3], $bstats.Bboxes[4])
  Write-Host "B disjoint: consecutive stops' painted bboxes do not overlap -> $(if ($disjointOk) { 'PASS' } else { 'FAIL' })"
  $ok = $ok -and $disjointOk

  $r = Assert-Agrees "B wrap returns to the first stop" $ctx.Data["b5-0"] $ctx.Data["b1-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar; $ok = $ok -and $r
  $r = Assert-Agrees "B traversal is deterministic" $ctx.Data["b3b-0"] $ctx.Data["b3-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar; $ok = $ok -and $r
  $r = Assert-Agrees "B Shift+Tab from stop 3 returns to stop 2" $ctx.Data["brev-0"] $ctx.Data["b2-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar; $ok = $ok -and $r
  $r = Assert-Differs "B Shift+Tab actually moved" $ctx.Data["brev-0"] $ctx.Data["b3-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar; $ok = $ok -and $r

  # ── Control A -- click routing and item identity ─────────────────────
  Write-Host ""
  Write-Host "Control A -- click routing and item identity"
  foreach ($t in @("a0", "a3", "a0b")) {
    $r = Assert-Agrees "A two frames with no input agree within the measured jitter ($t)" $ctx.Data["$t-0"] $ctx.Data["$t-1"] $ctx.Regions.Caption $ctx.Bands.Caption
    $ok = $ok -and $r
  }
  $r = Assert-Differs "A caption, thumbnail 0 vs 3" $ctx.Data["a0-0"] $ctx.Data["a3-0"] $ctx.Regions.Caption $ctx.Bands.Caption; $ok = $ok -and $r
  $r = Assert-Agrees "A caption, thumbnail 0 twice" $ctx.Data["a0-0"] $ctx.Data["a0b-0"] $ctx.Regions.Caption $ctx.Bands.Caption; $ok = $ok -and $r
  $r = Assert-Agrees "A photo box, thumbnail 0 vs 3" $ctx.Data["a0-0"] $ctx.Data["a3-0"] $ctx.Regions.Photo $ctx.Bands.Photo; $ok = $ok -and $r

  # ── Control D -- Esc (whole client) ───────────────────────────────────
  Write-Host ""
  Write-Host "Control D -- Esc (whole client)"
  $r = Assert-Agrees "D a recognised key with no handler changes nothing" $ctx.Data["d-home-0"] $ctx.Data["d-open-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree; $ok = $ok -and $r
  $r = Assert-Differs "D Escape closed the lightbox" $ctx.Data["d-closed-0"] $ctx.Data["d-open-0"] $ctx.Regions.Whole $ctx.Bands.WholeDiffer; $ok = $ok -and $r
  $r = Assert-Agrees "D the client returned to its pre-open state" $ctx.Data["d-closed-0"] $ctx.Data["d-pre-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree; $ok = $ok -and $r

  # ── Control C -- containment and occlusion ────────────────────────────
  Write-Host ""
  Write-Host "Control C -- containment and occlusion"

  # SENSOR (metric: px_differing_at_all -- see header's "Compare-side
  # revision"; the 60-summed threshold provably cannot see a change
  # attenuated by the scrim's 0.8 alpha).
  $sensorOk = Assert-Differs "C the toolbar is observable through the scrim (SENSOR)" $ctx.Data["c-openA-0"] $ctx.Data["c-openB-0"] $ctx.Regions.Toolbar $ctx.Bands.ToolbarOpenAny "Any"
  if (-not $sensorOk) {
    Write-Host "THE CONTAINMENT LEG IS WITHDRAWN, NOT PASSED: THE SENSOR CANNOT SEE THE TOOLBAR THROUGH THE SCRIM, SO 'NOTHING CHANGED' BELOW WOULD BE UNFALSIFIABLE."
  }
  $ok = $ok -and $sensorOk

  # Scrim attenuation: the justification for the metric switch above,
  # measured directly on this run's own frames, All's own bbox, the SAME
  # underlying state change (checked All -> checked Albums) with and
  # without the scrim in front of it.
  $unscrimmed = Measure-Region $ctx.Data["c-fired-0"] $ctx.Data["c-closed-0"] $ctx.Regions.AllBbox
  $scrimmed = Measure-Region $ctx.Data["c-openB-0"] $ctx.Data["c-openA-0"] $ctx.Regions.AllBbox
  $ratio = if ($scrimmed.MaxChannel -gt 0) { [Math]::Round($unscrimmed.MaxChannel / $scrimmed.MaxChannel, 2) } else { [double]::PositiveInfinity }
  Write-Host "  scrim attenuation (All's bbox, same state change All->Albums):"
  Write-Host "    unscrimmed (c-fired vs c-closed): max_channel=$($unscrimmed.MaxChannel) px_differing_at_all=$($unscrimmed.DiffAny) px_over_threshold=$($unscrimmed.DiffOver)"
  Write-Host "    scrimmed   (c-openB vs c-openA):  max_channel=$($scrimmed.MaxChannel) px_differing_at_all=$($scrimmed.DiffAny) px_over_threshold=$($scrimmed.DiffOver)"
  Write-Host "    ratio unscrimmed/scrimmed max_channel = $ratio, vs 1/(1-alpha) = $([Math]::Round(1/(1-0.8),2)) from gallery.ui's scrim fill #101820cc (alpha 0xcc = $([Math]::Round(204/255,3)))"

  # Conservativeness: a focus indicator landing on a toolbar stop under
  # the scrim would cover the SAME face (All's bbox) the checked-state
  # swing covers, so it would register on this same sensor -- stated so
  # the sensor's relation to what containment is actually about is
  # explicit, not assumed.
  $focusSwing = Measure-Region $ctx.Data["b1-0"] $ctx.Data["b-n-0"] $ctx.Regions.AllBbox
  Write-Host "  conservativeness: focus-indicator swing at the same face (b1 vs b-n, All's bbox): max_channel=$($focusSwing.MaxChannel) px_differing_at_all=$($focusSwing.DiffAny) -- a focus indicator on a toolbar stop under the scrim would register here too"

  $r = Assert-Agrees "C a click on the covered toolbar does nothing" $ctx.Data["c-openA-click-0"] $ctx.Data["c-openA-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree; $ok = $ok -and $r
  $r = Assert-Agrees "C and it wrote no state either -- checked in the clear" $ctx.Data["c-blocked-0"] $ctx.Data["c-closed-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree; $ok = $ok -and $r
  $r = Assert-Differs "C the same coordinate fires with the lightbox closed" $ctx.Data["c-fired-0"] $ctx.Data["c-closed-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar; $ok = $ok -and $r

  # Identification, replaced (owner disposition): lives on All's own
  # bbox rather than Albums's, and on the handler's BEHAVIOUR (loses its
  # checked colour) rather than on a colour blend -- see header.
  $closedMean = Mean-Region $ctx.Data["c-closed-0"] $ctx.Regions.AllBbox
  $firedMean = Mean-Region $ctx.Data["c-fired-0"] $ctx.Regions.AllBbox
  $b1Mean = Mean-Region $ctx.Data["b1-0"] $ctx.Regions.AllBbox
  Write-Host ("  All's bbox mean RGB: c-closed (checked)=(R={0:N1},G={1:N1},B={2:N1})  c-fired (after Albums click)=(R={3:N1},G={4:N1},B={5:N1})" -f $closedMean.R, $closedMean.G, $closedMean.B, $firedMean.R, $firedMean.G, $firedMean.B)
  $r = Assert-Differs "C the handler ran -- the previously checked tab lost its checked colour" $ctx.Data["c-fired-0"] $ctx.Data["c-closed-0"] $ctx.Regions.AllBbox $ctx.Bands.AllBbox; $ok = $ok -and $r
  $r = Assert-MeansDiffer "C look-alike exclusion -- All's new colour is not the checked+focused blend (rules out 'focus landed on All' instead)" $firedMean $b1Mean 40; $ok = $ok -and $r

  # CONTAINMENT (metric: px_differing_at_all -- same reason as the
  # sensor above).
  $r = Assert-Agrees "C five Tabs inside the scope never reach the toolbar" $ctx.Data["c-tab-0"] $ctx.Data["c-openB-0"] $ctx.Regions.Toolbar $ctx.Bands.ToolbarOpenAny "Any"; $ok = $ok -and $r
  $r = Assert-Differs "C ...but they did move focus inside it" $ctx.Data["c-tab-0"] $ctx.Data["c-openB-0"] $ctx.Regions.Side $ctx.Bands.Side; $ok = $ok -and $r
  $r = Assert-Differs "C with the scope gone, one Tab reaches the toolbar" $ctx.Data["c-tab-closed-0"] $ctx.Data["c-final-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar; $ok = $ok -and $r
  $r = Assert-Agrees "C the world returned after open/Tab/close" $ctx.Data["c-final-0"] $ctx.Data["c-fired-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree; $ok = $ok -and $r

  Write-Host ""
  if ($ok) {
    Write-Host "PASS: all controls' difference and agreement legs hold."
  } else {
    Write-Host "FAIL: see the leg(s) marked FAIL above."
  }

  foreach ($b in $ctx.Bitmaps) { $b.Dispose() }
  if (-not $ok) { exit 1 }
}

# ── SelfCheck mode ──────────────────────────────────────────────────────

function Do-SelfCheck() {
  $ctx = Load-Context
  Write-Host "SelfCheck: scale=$($ctx.Scale) client=$($ctx.Cw)x$($ctx.Ch)"
  Write-Host "Every verdict below is exercised with a deliberately WRONG pairing and must return false (i.e. the verdict must be ABLE to go red)."
  Write-Host ""

  $bstats = Get-BStopStats $ctx
  $rows = New-Object System.Collections.Generic.List[object]
  function Rec([string]$Name, [string]$Pairing, [bool]$Result) {
    $fired = -not $Result
    $rows.Add([PSCustomObject]@{ Verdict = $Name; WrongPairing = $Pairing; Fired = $fired })
  }

  # ── Control B ──────────────────────────────────────────────────────
  $r = Assert-Differs "[WRONG] B stop k painted" $ctx.Data["b-n-0"] $ctx.Data["b-n-1"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar
  Rec "B stop k painted (difference)" "b-n frame0 vs frame1" $r

  $revMono = Check-Monotone @($bstats.Centroids[4], $bstats.Centroids[3], $bstats.Centroids[2], $bstats.Centroids[1])
  Rec "B monotone" "reversed centroid list" $revMono

  $selfDisjoint = Check-Disjoint @($bstats.Bboxes[1], $bstats.Bboxes[1])
  Rec "B disjointness" "bbox(b1) against itself" $selfDisjoint

  $r = Assert-Agrees "[WRONG] B wrap returns to the first stop" $ctx.Data["b1-0"] $ctx.Data["b2-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar
  Rec "B wrap agreement" "b1 vs b2" $r

  $r = Assert-Agrees "[WRONG] B traversal is deterministic" $ctx.Data["b3-0"] $ctx.Data["b4-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar
  Rec "B determinism agreement" "b3 vs b4" $r

  $r = Assert-Agrees "[WRONG] B Shift+Tab from stop 3 returns to stop 2" $ctx.Data["brev-0"] $ctx.Data["b3-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar
  Rec "B reverse agreement" "brev vs b3" $r

  $r = Assert-Differs "[WRONG] B Shift+Tab actually moved" $ctx.Data["brev-0"] $ctx.Data["b2-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar
  Rec "B reverse difference" "brev vs b2" $r

  # ── Control A ──────────────────────────────────────────────────────
  $r = Assert-Differs "[WRONG] A caption, thumbnail 0 vs 3" $ctx.Data["a0-0"] $ctx.Data["a0b-0"] $ctx.Regions.Caption $ctx.Bands.Caption
  Rec "A caption difference" "a0 vs a0b" $r

  $r = Assert-Agrees "[WRONG] A caption, thumbnail 0 twice" $ctx.Data["a0-0"] $ctx.Data["a3-0"] $ctx.Regions.Caption $ctx.Bands.Caption
  Rec "A caption agreement" "a0 vs a3" $r

  $r = Assert-Agrees "[WRONG] A photo box, thumbnail 0 vs 3" $ctx.Data["c-openA-0"] $ctx.Data["c-closed-0"] $ctx.Regions.Photo $ctx.Bands.Photo
  Rec "A photo agreement" "c-openA vs c-closed (photo region)" $r

  # ── Control D ──────────────────────────────────────────────────────
  $r = Assert-Agrees "[WRONG] D a recognised key with no handler changes nothing" $ctx.Data["d-open-0"] $ctx.Data["d-closed-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree
  Rec "D agreement" "d-open vs d-closed" $r

  $r = Assert-Differs "[WRONG] D Escape closed the lightbox" $ctx.Data["d-open-0"] $ctx.Data["d-open-1"] $ctx.Regions.Whole $ctx.Bands.WholeDiffer
  Rec "D difference" "d-open frame0 vs frame1" $r

  $r = Assert-Agrees "[WRONG] D the client returned to its pre-open state" $ctx.Data["d-pre-0"] $ctx.Data["d-open-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree
  Rec "D return agreement" "d-pre vs d-open" $r

  # ── Control C ──────────────────────────────────────────────────────
  # Sensor and containment now verdict on px_differing_at_all (metric
  # "Any") -- see header's "Compare-side revision". Same wrong pairings
  # as before; the containment row is the one this metric switch fixes
  # (it read a false PASS under the old px_over_threshold metric,
  # because that metric could not see the scrimmed signal at all).
  $r = Assert-Differs "[WRONG] C the toolbar is observable through the scrim" $ctx.Data["c-openA-0"] $ctx.Data["c-openA-1"] $ctx.Regions.Toolbar $ctx.Bands.ToolbarOpenAny "Any"
  Rec "C sensor" "c-openA frame0 vs frame1" $r

  $r = Assert-Agrees "[WRONG] C a click on the covered toolbar does nothing" $ctx.Data["c-openA-0"] $ctx.Data["c-closed-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree
  Rec "C click-does-nothing" "c-openA vs c-closed" $r

  $r = Assert-Agrees "[WRONG] C and it wrote no state either" $ctx.Data["c-closed-0"] $ctx.Data["c-fired-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree
  Rec "C wrote-no-state" "c-closed vs c-fired" $r

  $r = Assert-Differs "[WRONG] C the same coordinate fires with the lightbox closed" $ctx.Data["c-closed-0"] $ctx.Data["c-blocked-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar
  Rec "C fires-when-closed" "c-closed vs c-blocked" $r

  $r = Assert-Agrees "[WRONG] C five Tabs inside the scope never reach the toolbar" $ctx.Data["c-openA-0"] $ctx.Data["c-openB-0"] $ctx.Regions.Toolbar $ctx.Bands.ToolbarOpenAny "Any"
  Rec "C containment" "c-openA vs c-openB (band)" $r

  $r = Assert-Differs "[WRONG] C ...but they did move focus inside it" $ctx.Data["c-openB-0"] $ctx.Data["c-openB-1"] $ctx.Regions.Side $ctx.Bands.Side
  Rec "C moved-inside" "c-openB frame0 vs frame1" $r

  $r = Assert-Differs "[WRONG] C with the scope gone, one Tab reaches the toolbar" $ctx.Data["c-final-0"] $ctx.Data["c-final-1"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar
  Rec "C tab-reaches-toolbar" "c-final frame0 vs frame1" $r

  $r = Assert-Agrees "[WRONG] C the world returned after open/Tab/close" $ctx.Data["c-fired-0"] $ctx.Data["c-openA-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree
  Rec "C world-returned" "c-fired vs c-openA" $r

  # C handler-ran (replacement identification leg): wrong pairing is the
  # byte-identical c-closed/c-blocked (both truly-closed, All-checked,
  # nothing written) -- must not register as a colour-loss.
  $r = Assert-Differs "[WRONG] C the handler ran" $ctx.Data["c-closed-0"] $ctx.Data["c-blocked-0"] $ctx.Regions.AllBbox $ctx.Bands.AllBbox
  Rec "C handler-ran" "c-closed vs c-blocked" $r

  # C look-alike exclusion: feed it b1's own mean against itself -- the
  # exclusion must fail to claim a difference when the two means
  # coincide.
  $b1MeanSC = Mean-Region $ctx.Data["b1-0"] $ctx.Regions.AllBbox
  $r = Assert-MeansDiffer "[WRONG] C look-alike exclusion" $b1MeanSC $b1MeanSC 40
  Rec "C look-alike exclusion" "b1 mean vs b1 mean (itself)" $r

  Write-Host ""
  Write-Host ("{0,-45} {1,-40} {2}" -f "verdict", "wrong pairing", "fired-as-expected")
  $allFired = $true
  foreach ($row in $rows) {
    $yn = if ($row.Fired) { "yes" } else { "no" }
    if (-not $row.Fired) { $allFired = $false }
    Write-Host ("{0,-45} {1,-40} {2}" -f $row.Verdict, $row.WrongPairing, $yn)
  }

  Write-Host ""
  foreach ($b in $ctx.Bitmaps) { $b.Dispose() }
  if ($allFired) {
    Write-Host "PASS: every verdict fired red under its wrong pairing."
  } else {
    Write-Host "FAIL: at least one verdict passed under a wrong pairing -- see 'no' rows above."
    exit 1
  }
}

if ($Capture) { Do-Capture }
if ($Compare) { Do-Compare }
if ($SelfCheck) { Do-SelfCheck }
exit 0
