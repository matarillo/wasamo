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
# ── Band policy, noise-floor gate, and self-check coverage (this pass's
#    revision, independent review finding) ─────────────────────────────
#
# Bands used to be `max(within-set-noise * 4, floor)`, where the noise
# was measured over exactly the tags each jitter leg covers -- so the
# jitter legs were compared against a band built FROM their own
# measurement (jitter_t <= noise <= max(4*noise, floor) always: no frame
# set could ever fail those legs), and the inflated band then leaked
# into every other leg sharing that region. Every band below is now an
# INDEPENDENT CONSTANT, not a measurement:
#   Toolbar = Caption = Photo = Side = AllBbox = ToolbarOpenAny = 40 px
#     -- under 0.006% of the 982x703 client, and an order of magnitude
#     below the smallest real change this set measures (79 px, control
#     A's caption, thumbnail 0 vs 3).
#   WholeAgree = cw*ch/2000 (<0.05% of the client), WholeDiffer =
#     cw*ch/20 (a lightbox opening changes over 5%) -- same values as
#     before, just no longer max'd against noise.
#
# The within-set noise this sitting actually measured is not discarded;
# it becomes a CHECKED quantity instead of a band input. Before any
# other leg, a per-region gate asserts max_channel<=13 and
# px_over_threshold==0 across every within-set frame pair in that region
# (F-33's independently measured per-channel tolerance, worst case
# 13/channel = 39 summed -- below the 60-summed px_over_threshold bar,
# so a clean within-set pair must show 0 pixels over it). If a sitting's
# own noise exceeds this, the run FAILS with that finding instead of
# quietly widening a band to absorb it -- the legs below cannot be
# judged against a noise floor that has not itself been shown clean.
# Control B/A's per-set "two frames with no input agree" legs are judged
# by these same two fixed numbers, for the same reason -- never against
# a band derived from themselves.
#
# Coverage is enforced by the script, not asserted in prose: `-Compare`
# registers every verdict it prints under a stable name, in order;
# `-SelfCheck` registers the name each of its rows exercises (plus two
# rows for the in-run guards below, which `-Compare` never touches,
# since only `-Capture` calls them); and `-SelfCheck` replays
# `-Compare`'s verdict pass over the same loaded frames (pure pixel
# comparison, no capture) purely to obtain that name list, then FAILS if
# any `-Compare` name has no matching `-SelfCheck` row. Both counts are
# printed, so the claim "every verdict is self-checked" is something the
# run itself can falsify, not something this comment asserts.
#
# A self-check row's WRONG pairing is only informative if it forces the
# assertion to look at the actual sampled pixels: a byte-identical pair
# (this sitting's own within-set frame0/frame1 pairs ARE byte-identical
# -- see the noise-floor gate above) proves only that the comparator's
# inequality is spelled correctly; it cannot catch a mis-specified
# region -- if the caption region pointed at a blank rectangle, a
# byte-identical wrong pairing would still self-check green. Each
# DIFFERENCE row's wrong pairing is therefore classified and printed as
# one of:
#   region-scoped -- the two frames genuinely differ elsewhere in the
#     client, but agree in THIS row's sampled region, so the row proves
#     the assertion is actually reading that region (a blank or
#     mis-pointed rectangle would fail this self-check).
#   degenerate -- the two frames do not differ anywhere (whole-client
#     DIFFERENCE legs only: any two frames that differ at all differ
#     over the whole client, so no region-scoped wrong pairing can exist
#     for one -- this is said in the output, not left looking like an
#     oversight).
# AGREEMENT/noise/jitter/monotone/disjoint/exclusion/guard rows are not
# classified this way: their wrong pairing already has to differ IN the
# sampled region for the row to mean anything, which is the direct case,
# not the elsewhere-but-not-here case DIFFERENCE rows need.
#
# ── In-run guards ───────────────────────────────────────────────────
# After every lightbox-opening action, the whole-client diff against the
# immediately preceding closed frame must exceed cw*ch/20 (the lightbox
# actually opened); after every Escape, the whole-client diff against
# that same preceding closed frame must fall back below cw*ch/200 (it
# actually closed, and closed to the SAME place). A run that silently
# produced a wrong artifact -- a click that missed, a key that went
# nowhere -- throws immediately rather than saving a frame set that
# LOOKS like evidence. The two predicates (`Test-OpenedGuard`,
# `Test-ClosedGuard`) are module-scope pure functions so `-SelfCheck` can
# exercise them too (they used to be declared inside `Do-Capture`, which
# `-SelfCheck` cannot reach) -- `Do-Capture`'s `Guard-Open`/`Guard-Closed`
# call them and throw exactly as before, no behaviour change to the
# capture path.
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

# Defect-4 fix (independent review): the in-run guard predicates,
# extracted to module scope as pure functions over two frame byte-buffers
# plus the client size, so -SelfCheck can reach and exercise them with
# wrong inputs -- they used to be declared inside Do-Capture, which
# -SelfCheck cannot call into. Do-Capture's Guard-Open/Guard-Closed below
# call these and throw exactly as before; no behaviour change to the
# capture path.
function Test-OpenedGuard($Frame, $PrecedingClosed, $Cw, $Ch) {
  $openLimit = [int]($Cw * $Ch / 20)
  $m = Diff-Count $Frame $PrecedingClosed 0 $Cw 0 $Ch
  return ($m.DiffOver -gt $openLimit)
}

function Test-ClosedGuard($Frame, $PrecedingClosed, $Cw, $Ch) {
  $closedLimit = [int]($Cw * $Ch / 200)
  $m = Diff-Count $Frame $PrecedingClosed 0 $Cw 0 $Ch
  return ($m.DiffOver -lt $closedLimit)
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

# ── The verdict functions every leg (and every self-check) routes
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

# Defect-1 fix (independent review): the noise-floor GATE. Aggregates
# max_channel and px_over_threshold over a LIST of same-tag frame0/frame1
# pairs in one region and checks them against F-33's independently
# measured tolerance (max_channel<=13, px_over_threshold==0) -- a fixed
# criterion, never a band built from this same measurement. Run once per
# region, before any other leg; a FAIL here means the legs below cannot
# be trusted to discriminate signal from this sitting's own noise.
function Assert-NoiseFloor([string]$Label, $TagPairs, $Region, $Data) {
  $maxC = 0; $maxOver = 0
  foreach ($p in $TagPairs) {
    $m = Measure-Region $Data[$p[0]] $Data[$p[1]] $Region
    if ($m.MaxChannel -gt $maxC) { $maxC = $m.MaxChannel }
    if ($m.DiffOver -gt $maxOver) { $maxOver = $m.DiffOver }
  }
  $pass = ($maxC -le 13) -and ($maxOver -eq 0)
  $v = if ($pass) { "PASS" } else { "FAIL" }
  Write-Host ("NOISE      {0,-70} max within-set max_channel={1,3} (limit <=13) max within-set px_over_threshold={2,7} (limit ==0) -> {3}" -f $Label, $maxC, $maxOver, $v)
  return $pass
}

# Defect-1 fix, part (c): the per-set jitter legs ("two frames with no
# input agree") are judged by these same two fixed F-33 numbers, applied
# to a SINGLE pair -- never against a band derived from themselves (the
# circularity the independent review reported).
function Assert-WithinNoise([string]$Label, $A, $B, $Region) {
  $m = Measure-Region $A $B $Region
  $pass = ($m.MaxChannel -le 13) -and ($m.DiffOver -eq 0)
  $v = if ($pass) { "PASS" } else { "FAIL" }
  Write-Host ("JITTER     {0,-70} max_channel={1,3} px_differing_at_all={2,7} px_over_threshold={3,7} [verdict: max_channel<=13 and px_over_threshold==0] -> {4}" -f $Label, $m.MaxChannel, $m.DiffAny, $m.DiffOver, $v)
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
      if (-not (Test-OpenedGuard $Frame $PrecedingClosed $cr.W $cr.H)) {
        throw "in-run guard failed: opening the lightbox for '$Tag' only changed $($m.DiffOver) px against the preceding closed frame (limit >$openLimit) -- the lightbox may not have opened"
      }
    }

    function Guard-Closed([string]$Tag, $Frame, $PrecedingClosed) {
      $m = Diff-Count $Frame $PrecedingClosed 0 $cr.W 0 $cr.H
      Write-Host ("  guard(closed) '$Tag' vs preceding closed: max_channel=$($m.MaxChannel) px_differing_at_all=$($m.DiffAny) px_over_threshold=$($m.DiffOver) (must be below $closedLimit)")
      if (-not (Test-ClosedGuard $Frame $PrecedingClosed $cr.W $cr.H)) {
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

  # Tag lists feeding the noise-floor gate (defect 1b): every tag whose
  # own frame0/frame1 pair is a same-input, no-intervening-action capture
  # in that region. Same composition as this script used before the fix
  # (when it drove the band multiplier); now it drives a fixed-criterion
  # CHECK instead -- see Assert-NoiseFloor.
  $toolbarTags = @("b-n", "b1", "b2", "b3", "b4", "b5", "b3b", "brev", "c-openA", "c-openB", "c-fired", "c-closed", "c-tab", "c-tab-closed", "c-final")
  $captionTags = @("a0", "a3", "a0b")
  $photoTags = @("a0", "a3")
  $wholeTags = @("d-home", "d-open", "d-closed", "d-pre", "c-openA-click", "c-openA", "c-blocked", "c-closed", "c-final", "c-fired")
  $sideTags = @("c-tab", "c-openB")
  $allBboxTags = @("c-fired", "c-closed")

  # Defect 1(a): every band is now an INDEPENDENT CONSTANT (chosen, not
  # measured) -- see header "Band policy, noise-floor gate, and
  # self-check coverage".
  $bands = @{
    Toolbar        = 40
    Caption        = 40
    Photo          = 40
    Side           = 40
    AllBbox        = 40
    ToolbarOpenAny = 40
    WholeAgree     = [int]($cw * $ch / 2000)
    WholeDiffer    = [int]($cw * $ch / 20)
  }

  return @{
    Meta = $meta; Scale = $scale; Cw = $cw; Ch = $ch
    Data = $data; Bitmaps = $bmps; Regions = $regions
    ToolbarTags = $toolbarTags; CaptionTags = $captionTags; PhotoTags = $photoTags
    WholeTags = $wholeTags; SideTags = $sideTags; AllBboxTags = $allBboxTags
    Bands = $bands
  }
}

# ── The canonical verdict pass ──────────────────────────────────────────
# Defect-2 fix (independent review): this is the SINGLE place -Compare's
# verdicts are computed. -Compare calls it directly; -SelfCheck ALSO
# calls it (over the same already-loaded frames -- pure pixel comparison,
# no capture) purely to harvest the ordered list of verdict NAMES it
# registers, so the "-SelfCheck covers every -Compare verdict" claim is
# something the run checks against this function's actual behaviour,
# not a hand-maintained list that can drift from it.
function Invoke-CompareVerdicts($ctx) {
  $ok = $true
  $names = New-Object System.Collections.Generic.List[string]
  function CReg([string]$Name) { $names.Add($Name) }

  # ── Noise-floor gate (defect 1b) -- before every other leg ──────────
  Write-Host "Noise-floor gate -- F-33's independently measured per-channel tolerance (max_channel<=13, px_over_threshold==0), checked over every within-set frame pair per region, before any leg below can be judged:"
  $noiseRegions = @(
    @{ Name = "Toolbar"; Tags = $ctx.ToolbarTags; Region = $ctx.Regions.Toolbar },
    @{ Name = "Caption"; Tags = $ctx.CaptionTags; Region = $ctx.Regions.Caption },
    @{ Name = "Photo";   Tags = $ctx.PhotoTags;   Region = $ctx.Regions.Photo },
    @{ Name = "Side";    Tags = $ctx.SideTags;    Region = $ctx.Regions.Side },
    @{ Name = "Whole";   Tags = $ctx.WholeTags;   Region = $ctx.Regions.Whole },
    @{ Name = "AllBbox"; Tags = $ctx.AllBboxTags; Region = $ctx.Regions.AllBbox }
  )
  foreach ($nr in $noiseRegions) {
    $label = "Noise floor -- $($nr.Name) region"
    $pairs = @($nr.Tags | ForEach-Object { , @("$_-0", "$_-1") })
    $r = Assert-NoiseFloor $label $pairs $nr.Region $ctx.Data
    CReg $label
    $ok = $ok -and $r
  }
  if (-not $ok) {
    Write-Host ""
    Write-Host "FAIL: this sitting's noise exceeds the independently measured F-33 tolerance in at least one region above -- the legs below cannot be judged."
    return @{ Ok = $false; Names = $names }
  }

  # ── Control B -- traversal order (toolbar band) ──────────────────────
  Write-Host ""
  Write-Host "Control B -- traversal order (toolbar band)"
  Write-Host "within-set jitter, judged directly against F-33's tolerance (max_channel<=13, px_over_threshold==0) -- never against a band derived from this same measurement (defect 1's fix):"
  foreach ($t in @("b-n", "b1", "b2", "b3", "b4", "b5", "b3b", "brev")) {
    $label = "B two frames with no input agree within the measured jitter ($t)"
    $r = Assert-WithinNoise $label $ctx.Data["$t-0"] $ctx.Data["$t-1"] $ctx.Regions.Toolbar
    CReg $label
    $ok = $ok -and $r
  }

  $bstats = Get-BStopStats $ctx
  for ($k = 1; $k -le 4; $k++) {
    $label = "B stop $k painted"
    $r = Assert-Differs $label $ctx.Data["b$k-0"] $ctx.Data["b-n-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar
    CReg $label
    $ok = $ok -and $r
    Write-Host ("  stop $k bbox x[$($bstats.Bboxes[$k].MinX)..$($bstats.Bboxes[$k].MaxX)] centroid_x=$($bstats.Centroids[$k])")
  }
  $monoOk = Check-Monotone @($bstats.Centroids[1], $bstats.Centroids[2], $bstats.Centroids[3], $bstats.Centroids[4])
  Write-Host "B monotone: stop centroids increase left-to-right -> $(if ($monoOk) { 'PASS' } else { 'FAIL' })"
  CReg "B monotone: stop centroids increase left-to-right"
  $ok = $ok -and $monoOk
  $disjointOk = Check-Disjoint @($bstats.Bboxes[1], $bstats.Bboxes[2], $bstats.Bboxes[3], $bstats.Bboxes[4])
  Write-Host "B disjoint: consecutive stops' painted bboxes do not overlap -> $(if ($disjointOk) { 'PASS' } else { 'FAIL' })"
  CReg "B disjoint: consecutive stops' painted bboxes do not overlap"
  $ok = $ok -and $disjointOk

  $label = "B wrap returns to the first stop"
  $r = Assert-Agrees $label $ctx.Data["b5-0"] $ctx.Data["b1-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar; CReg $label; $ok = $ok -and $r
  $label = "B traversal is deterministic"
  $r = Assert-Agrees $label $ctx.Data["b3b-0"] $ctx.Data["b3-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar; CReg $label; $ok = $ok -and $r
  $label = "B Shift+Tab from stop 3 returns to stop 2"
  $r = Assert-Agrees $label $ctx.Data["brev-0"] $ctx.Data["b2-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar; CReg $label; $ok = $ok -and $r
  $label = "B Shift+Tab actually moved"
  $r = Assert-Differs $label $ctx.Data["brev-0"] $ctx.Data["b3-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar; CReg $label; $ok = $ok -and $r

  # ── Control A -- click routing and item identity ─────────────────────
  Write-Host ""
  Write-Host "Control A -- click routing and item identity"
  foreach ($t in @("a0", "a3", "a0b")) {
    $label = "A two frames with no input agree within the measured jitter ($t)"
    $r = Assert-WithinNoise $label $ctx.Data["$t-0"] $ctx.Data["$t-1"] $ctx.Regions.Caption
    CReg $label
    $ok = $ok -and $r
  }
  $label = "A caption, thumbnail 0 vs 3"
  $r = Assert-Differs $label $ctx.Data["a0-0"] $ctx.Data["a3-0"] $ctx.Regions.Caption $ctx.Bands.Caption; CReg $label; $ok = $ok -and $r
  $label = "A caption, thumbnail 0 twice"
  $r = Assert-Agrees $label $ctx.Data["a0-0"] $ctx.Data["a0b-0"] $ctx.Regions.Caption $ctx.Bands.Caption; CReg $label; $ok = $ok -and $r
  $label = "A photo box, thumbnail 0 vs 3"
  $r = Assert-Agrees $label $ctx.Data["a0-0"] $ctx.Data["a3-0"] $ctx.Regions.Photo $ctx.Bands.Photo; CReg $label; $ok = $ok -and $r

  # ── Control D -- Esc (whole client) ───────────────────────────────────
  Write-Host ""
  Write-Host "Control D -- Esc (whole client)"
  $label = "D a recognised key with no handler changes nothing"
  $r = Assert-Agrees $label $ctx.Data["d-home-0"] $ctx.Data["d-open-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree; CReg $label; $ok = $ok -and $r
  $label = "D Escape closed the lightbox"
  $r = Assert-Differs $label $ctx.Data["d-closed-0"] $ctx.Data["d-open-0"] $ctx.Regions.Whole $ctx.Bands.WholeDiffer; CReg $label; $ok = $ok -and $r
  $label = "D the client returned to its pre-open state"
  $r = Assert-Agrees $label $ctx.Data["d-closed-0"] $ctx.Data["d-pre-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree; CReg $label; $ok = $ok -and $r

  # ── Control C -- containment and occlusion ────────────────────────────
  Write-Host ""
  Write-Host "Control C -- containment and occlusion"

  # SENSOR (metric: px_differing_at_all -- see header's "Compare-side
  # revision"; the 60-summed threshold provably cannot see a change
  # attenuated by the scrim's 0.8 alpha).
  $label = "C the toolbar is observable through the scrim (SENSOR)"
  $sensorOk = Assert-Differs $label $ctx.Data["c-openA-0"] $ctx.Data["c-openB-0"] $ctx.Regions.Toolbar $ctx.Bands.ToolbarOpenAny "Any"
  CReg $label
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

  $label = "C a click on the covered toolbar does nothing"
  $r = Assert-Agrees $label $ctx.Data["c-openA-click-0"] $ctx.Data["c-openA-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree; CReg $label; $ok = $ok -and $r
  $label = "C and it wrote no state either -- checked in the clear"
  $r = Assert-Agrees $label $ctx.Data["c-blocked-0"] $ctx.Data["c-closed-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree; CReg $label; $ok = $ok -and $r
  $label = "C the same coordinate fires with the lightbox closed"
  $r = Assert-Differs $label $ctx.Data["c-fired-0"] $ctx.Data["c-closed-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar; CReg $label; $ok = $ok -and $r

  # Identification, replaced (owner disposition): lives on All's own
  # bbox rather than Albums's, and on the handler's BEHAVIOUR (loses its
  # checked colour) rather than on a colour blend -- see header.
  $closedMean = Mean-Region $ctx.Data["c-closed-0"] $ctx.Regions.AllBbox
  $firedMean = Mean-Region $ctx.Data["c-fired-0"] $ctx.Regions.AllBbox
  $b1Mean = Mean-Region $ctx.Data["b1-0"] $ctx.Regions.AllBbox
  Write-Host ("  All's bbox mean RGB: c-closed (checked)=(R={0:N1},G={1:N1},B={2:N1})  c-fired (after Albums click)=(R={3:N1},G={4:N1},B={5:N1})" -f $closedMean.R, $closedMean.G, $closedMean.B, $firedMean.R, $firedMean.G, $firedMean.B)
  $label = "C the handler ran -- the previously checked tab lost its checked colour"
  $r = Assert-Differs $label $ctx.Data["c-fired-0"] $ctx.Data["c-closed-0"] $ctx.Regions.AllBbox $ctx.Bands.AllBbox; CReg $label; $ok = $ok -and $r
  $label = "C look-alike exclusion -- All's new colour is not the checked+focused blend (rules out 'focus landed on All' instead)"
  $r = Assert-MeansDiffer $label $firedMean $b1Mean 40; CReg $label; $ok = $ok -and $r

  # CONTAINMENT (metric: px_differing_at_all -- same reason as the
  # sensor above).
  $label = "C five Tabs inside the scope never reach the toolbar"
  $r = Assert-Agrees $label $ctx.Data["c-tab-0"] $ctx.Data["c-openB-0"] $ctx.Regions.Toolbar $ctx.Bands.ToolbarOpenAny "Any"; CReg $label; $ok = $ok -and $r
  $label = "C ...but they did move focus inside it"
  $r = Assert-Differs $label $ctx.Data["c-tab-0"] $ctx.Data["c-openB-0"] $ctx.Regions.Side $ctx.Bands.Side; CReg $label; $ok = $ok -and $r
  $label = "C with the scope gone, one Tab reaches the toolbar"
  $r = Assert-Differs $label $ctx.Data["c-tab-closed-0"] $ctx.Data["c-final-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar; CReg $label; $ok = $ok -and $r
  $label = "C the world returned after open/Tab/close"
  $r = Assert-Agrees $label $ctx.Data["c-final-0"] $ctx.Data["c-fired-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree; CReg $label; $ok = $ok -and $r

  return @{ Ok = $ok; Names = $names }
}

# ── Compare mode ────────────────────────────────────────────────────────

function Do-Compare() {
  $ctx = Load-Context
  Write-Host "commit=$($ctx.Meta['commit'])"
  Write-Host "scale=$($ctx.Scale) client=$($ctx.Cw)x$($ctx.Ch) real_keys=$($ctx.Meta['real_keys'])"
  Write-Host ""
  Write-Host "pass bands (chosen constants, not measurements -- see header 'Band policy, noise-floor gate, and self-check coverage'):"
  Write-Host "  toolbar=$($ctx.Bands.Toolbar) caption=$($ctx.Bands.Caption) photo=$($ctx.Bands.Photo) side=$($ctx.Bands.Side) all_bbox=$($ctx.Bands.AllBbox) toolbar_open_any=$($ctx.Bands.ToolbarOpenAny) whole_agree=$($ctx.Bands.WholeAgree) whole_differ=$($ctx.Bands.WholeDiffer)"

  $result = Invoke-CompareVerdicts $ctx

  Write-Host ""
  if ($result.Ok) {
    Write-Host "PASS: all controls' difference and agreement legs hold (including the noise-floor gate)."
  } else {
    Write-Host "FAIL: see the leg(s) marked FAIL above."
  }
  Write-Host "verdicts registered: $($result.Names.Count)"

  foreach ($b in $ctx.Bitmaps) { $b.Dispose() }
  if (-not $result.Ok) { exit 1 }
}

# ── SelfCheck mode ──────────────────────────────────────────────────────

function Do-SelfCheck() {
  $ctx = Load-Context
  Write-Host "SelfCheck: scale=$($ctx.Scale) client=$($ctx.Cw)x$($ctx.Ch)"
  Write-Host "Every verdict below is exercised with a deliberately WRONG pairing and must return false (i.e. the verdict must be ABLE to go red)."
  Write-Host ""

  $bstats = Get-BStopStats $ctx
  $rows = New-Object System.Collections.Generic.List[object]
  $selfCheckNames = New-Object System.Collections.Generic.List[string]
  function Rec([string]$Name, [string]$Pairing, [bool]$Result, [string]$Class = "") {
    $fired = -not $Result
    $rows.Add([PSCustomObject]@{ Verdict = $Name; WrongPairing = $Pairing; Fired = $fired; Class = $Class })
    $selfCheckNames.Add($Name)
  }

  # ── Noise-floor gate rows (defect 1b) ─────────────────────────────
  # Wrong input: c-closed vs c-openA (lightbox closed vs open) -- verified
  # to differ substantially in every region below (measured directly on
  # this run's frames), so the gate must detect it as noise-tolerance
  # violation, proving the gate is not vacuously always-PASS.
  foreach ($nr in @("Toolbar", "Caption", "Photo", "Side", "Whole", "AllBbox")) {
    $label = "Noise floor -- $nr region"
    $region = $ctx.Regions.$nr
    $r = Assert-NoiseFloor "[WRONG] $label" @(, @("c-closed-0", "c-openA-0")) $region $ctx.Data
    Rec $label "c-closed vs c-openA (open vs closed, $nr region)" $r
  }

  # ── Control B jitter rows (defect 2: previously uncovered) ─────────
  # Wrong input: each B tag paired against a DIFFERENT B tag known to
  # differ in the toolbar band (the real "B stop k painted" difference
  # legs), proving Assert-WithinNoise can detect real jitter/change.
  $bJitterPartner = @{ "b-n" = "b1"; "b1" = "b-n"; "b2" = "b-n"; "b3" = "b-n"; "b4" = "b-n"; "b5" = "b-n"; "b3b" = "b-n"; "brev" = "b-n" }
  foreach ($t in @("b-n", "b1", "b2", "b3", "b4", "b5", "b3b", "brev")) {
    $label = "B two frames with no input agree within the measured jitter ($t)"
    $partner = $bJitterPartner[$t]
    $r = Assert-WithinNoise "[WRONG] $label" $ctx.Data["$t-0"] $ctx.Data["$partner-0"] $ctx.Regions.Toolbar
    Rec $label "$t vs $partner (Toolbar)" $r
  }

  # ── Control B ──────────────────────────────────────────────────────
  # "B stop k painted" (k=1..4): wrong pairing a0 vs a3 -- VERIFIED
  # (measured on this run's frames): Toolbar band max_channel=0,
  # px_over_threshold=0 between a0 and a3 (both lightbox-open with the
  # same tab checked and the same focus, differing only in the caption)
  # -- so the row is region-scoped, not degenerate.
  for ($k = 1; $k -le 4; $k++) {
    $label = "B stop $k painted"
    $r = Assert-Differs "[WRONG] $label" $ctx.Data["a0-0"] $ctx.Data["a3-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar
    Rec $label "a0 vs a3 (Toolbar)" $r "region-scoped"
  }

  $revMono = Check-Monotone @($bstats.Centroids[4], $bstats.Centroids[3], $bstats.Centroids[2], $bstats.Centroids[1])
  Rec "B monotone: stop centroids increase left-to-right" "reversed centroid list" $revMono

  $selfDisjoint = Check-Disjoint @($bstats.Bboxes[1], $bstats.Bboxes[1])
  Rec "B disjoint: consecutive stops' painted bboxes do not overlap" "bbox(b1) against itself" $selfDisjoint

  $r = Assert-Agrees "[WRONG] B wrap returns to the first stop" $ctx.Data["b1-0"] $ctx.Data["b2-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar
  Rec "B wrap returns to the first stop" "b1 vs b2" $r

  $r = Assert-Agrees "[WRONG] B traversal is deterministic" $ctx.Data["b3-0"] $ctx.Data["b4-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar
  Rec "B traversal is deterministic" "b3 vs b4" $r

  $r = Assert-Agrees "[WRONG] B Shift+Tab from stop 3 returns to stop 2" $ctx.Data["brev-0"] $ctx.Data["b3-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar
  Rec "B Shift+Tab from stop 3 returns to stop 2" "brev vs b3" $r

  # "B Shift+Tab actually moved": wrong pairing a0 vs a3 -- same
  # verified region-scoped property as "B stop k painted" above.
  $r = Assert-Differs "[WRONG] B Shift+Tab actually moved" $ctx.Data["a0-0"] $ctx.Data["a3-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar
  Rec "B Shift+Tab actually moved" "a0 vs a3 (Toolbar)" $r "region-scoped"

  # ── Control A jitter rows (defect 2: previously uncovered) ─────────
  $aJitterPartner = @{ "a0" = "a3"; "a3" = "a0"; "a0b" = "a3" }
  foreach ($t in @("a0", "a3", "a0b")) {
    $label = "A two frames with no input agree within the measured jitter ($t)"
    $partner = $aJitterPartner[$t]
    $r = Assert-WithinNoise "[WRONG] $label" $ctx.Data["$t-0"] $ctx.Data["$partner-0"] $ctx.Regions.Caption
    Rec $label "$t vs $partner (Caption)" $r
  }

  # ── Control A ──────────────────────────────────────────────────────
  # "A caption difference": wrong pairing b1 vs b2 -- VERIFIED: Caption
  # region max_channel=0, px_over_threshold=0 (both closed frames,
  # differing only in the toolbar band) -- region-scoped.
  $r = Assert-Differs "[WRONG] A caption, thumbnail 0 vs 3" $ctx.Data["b1-0"] $ctx.Data["b2-0"] $ctx.Regions.Caption $ctx.Bands.Caption
  Rec "A caption, thumbnail 0 vs 3" "b1 vs b2 (Caption)" $r "region-scoped"

  $r = Assert-Agrees "[WRONG] A caption, thumbnail 0 twice" $ctx.Data["a0-0"] $ctx.Data["a3-0"] $ctx.Regions.Caption $ctx.Bands.Caption
  Rec "A caption, thumbnail 0 twice" "a0 vs a3" $r

  $r = Assert-Agrees "[WRONG] A photo box, thumbnail 0 vs 3" $ctx.Data["c-openA-0"] $ctx.Data["c-closed-0"] $ctx.Regions.Photo $ctx.Bands.Photo
  Rec "A photo box, thumbnail 0 vs 3" "c-openA vs c-closed (photo region)" $r

  # ── Control D ──────────────────────────────────────────────────────
  $r = Assert-Agrees "[WRONG] D a recognised key with no handler changes nothing" $ctx.Data["d-open-0"] $ctx.Data["d-closed-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree
  Rec "D a recognised key with no handler changes nothing" "d-open vs d-closed" $r

  # "D Escape closed the lightbox" is a WHOLE-CLIENT DIFFERENCE leg: any
  # two frames that differ at all differ over the whole client, so no
  # region-scoped wrong pairing can exist for it -- kept degenerate
  # (d-open frame0 vs frame1, byte-identical on this sitting), and
  # printed as such rather than left looking like an oversight.
  $r = Assert-Differs "[WRONG] D Escape closed the lightbox" $ctx.Data["d-open-0"] $ctx.Data["d-open-1"] $ctx.Regions.Whole $ctx.Bands.WholeDiffer
  Rec "D Escape closed the lightbox" "d-open frame0 vs frame1 (no region-scoped alternative exists for a whole-client leg)" $r "degenerate"

  $r = Assert-Agrees "[WRONG] D the client returned to its pre-open state" $ctx.Data["d-pre-0"] $ctx.Data["d-open-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree
  Rec "D the client returned to its pre-open state" "d-pre vs d-open" $r

  # ── Control C ──────────────────────────────────────────────────────
  # "C sensor": wrong pairing c-openB vs c-tab -- VERIFIED: Toolbar band
  # max_channel=0, px_differing_at_all=0 (agree in the band; they differ
  # substantially in the lightbox's own side columns instead, measured
  # 4369 px over threshold there) -- region-scoped.
  $r = Assert-Differs "[WRONG] C the toolbar is observable through the scrim (SENSOR)" $ctx.Data["c-openB-0"] $ctx.Data["c-tab-0"] $ctx.Regions.Toolbar $ctx.Bands.ToolbarOpenAny "Any"
  Rec "C the toolbar is observable through the scrim (SENSOR)" "c-openB vs c-tab (Toolbar, Any)" $r "region-scoped"

  $r = Assert-Agrees "[WRONG] C a click on the covered toolbar does nothing" $ctx.Data["c-openA-0"] $ctx.Data["c-closed-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree
  Rec "C a click on the covered toolbar does nothing" "c-openA vs c-closed" $r

  $r = Assert-Agrees "[WRONG] C and it wrote no state either -- checked in the clear" $ctx.Data["c-closed-0"] $ctx.Data["c-fired-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree
  Rec "C and it wrote no state either -- checked in the clear" "c-closed vs c-fired" $r

  # "C fires-when-closed": wrong pairing a0 vs a3 -- same verified
  # region-scoped Toolbar property used above.
  $r = Assert-Differs "[WRONG] C the same coordinate fires with the lightbox closed" $ctx.Data["a0-0"] $ctx.Data["a3-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar
  Rec "C the same coordinate fires with the lightbox closed" "a0 vs a3 (Toolbar)" $r "region-scoped"

  # "C handler-ran": wrong pairing b2 vs b3 -- VERIFIED: AllBbox region
  # max_channel=0, px_over_threshold=0 (they differ at the right of the
  # toolbar -- 10183 px over threshold there -- but agree at All's own
  # face) -- region-scoped.
  $r = Assert-Differs "[WRONG] C the handler ran -- the previously checked tab lost its checked colour" $ctx.Data["b2-0"] $ctx.Data["b3-0"] $ctx.Regions.AllBbox $ctx.Bands.AllBbox
  Rec "C the handler ran -- the previously checked tab lost its checked colour" "b2 vs b3 (AllBbox)" $r "region-scoped"

  $b1MeanSC = Mean-Region $ctx.Data["b1-0"] $ctx.Regions.AllBbox
  $r = Assert-MeansDiffer "[WRONG] C look-alike exclusion" $b1MeanSC $b1MeanSC 40
  Rec "C look-alike exclusion -- All's new colour is not the checked+focused blend (rules out 'focus landed on All' instead)" "b1 mean vs b1 mean (itself)" $r

  $r = Assert-Agrees "[WRONG] C five Tabs inside the scope never reach the toolbar" $ctx.Data["c-openA-0"] $ctx.Data["c-openB-0"] $ctx.Regions.Toolbar $ctx.Bands.ToolbarOpenAny "Any"
  Rec "C five Tabs inside the scope never reach the toolbar" "c-openA vs c-openB (Toolbar, Any)" $r

  # "C moved-inside": wrong pairing c-openA vs c-openB -- VERIFIED: Side
  # region max_channel=0, px_over_threshold=0 (they differ in the
  # toolbar band instead -- the Albums click between the two captures --
  # but agree in the side columns) -- region-scoped.
  $r = Assert-Differs "[WRONG] C ...but they did move focus inside it" $ctx.Data["c-openA-0"] $ctx.Data["c-openB-0"] $ctx.Regions.Side $ctx.Bands.Side
  Rec "C ...but they did move focus inside it" "c-openA vs c-openB (Side)" $r "region-scoped"

  # "C tab-reaches-toolbar": wrong pairing a0 vs a3 -- same verified
  # region-scoped Toolbar property used above.
  $r = Assert-Differs "[WRONG] C with the scope gone, one Tab reaches the toolbar" $ctx.Data["a0-0"] $ctx.Data["a3-0"] $ctx.Regions.Toolbar $ctx.Bands.Toolbar
  Rec "C with the scope gone, one Tab reaches the toolbar" "a0 vs a3 (Toolbar)" $r "region-scoped"

  $r = Assert-Agrees "[WRONG] C the world returned after open/Tab/close" $ctx.Data["c-fired-0"] $ctx.Data["c-openA-0"] $ctx.Regions.Whole $ctx.Bands.WholeAgree
  Rec "C the world returned after open/Tab/close" "c-fired vs c-openA" $r

  # ── In-run guard rows (defect 4) -- extra: -Compare never touches
  #    these (only -Capture calls them), so they are not part of the
  #    -Compare/-SelfCheck coverage cross-check below, but they still
  #    have to be exercisable, which they were not before this fix.
  $openGuardResult = Test-OpenedGuard $ctx.Data["c-closed-0"] $ctx.Data["c-blocked-0"] $ctx.Cw $ctx.Ch
  Rec "Guard(open): the lightbox actually opened (Test-OpenedGuard)" "c-closed vs c-blocked (both closed)" $openGuardResult

  $closedGuardResult = Test-ClosedGuard $ctx.Data["c-closed-0"] $ctx.Data["c-openA-0"] $ctx.Cw $ctx.Ch
  Rec "Guard(closed): the lightbox actually closed to the same place (Test-ClosedGuard)" "c-closed vs c-openA (open, not closed)" $closedGuardResult

  Write-Host ""
  Write-Host ("{0,-70} {1,-45} {2,-6} {3}" -f "verdict", "wrong pairing", "fired", "class")
  $allFired = $true
  $regionScopedCount = 0; $degenerateCount = 0
  foreach ($row in $rows) {
    $yn = if ($row.Fired) { "yes" } else { "no" }
    if (-not $row.Fired) { $allFired = $false }
    if ($row.Class -eq "region-scoped") { $regionScopedCount++ }
    if ($row.Class -eq "degenerate") { $degenerateCount++ }
    Write-Host ("{0,-70} {1,-45} {2,-6} {3}" -f $row.Verdict, $row.WrongPairing, $yn, $row.Class)
  }

  Write-Host ""
  Write-Host "DIFFERENCE-row wrong-pairing classification (defect 3): $regionScopedCount region-scoped, $degenerateCount degenerate, out of $($regionScopedCount + $degenerateCount) classified rows. (AGREEMENT/jitter/noise/monotone/disjoint/exclusion/guard rows are unclassified -- their wrong pairing already has to differ IN the sampled region, the direct case, not the elsewhere-but-not-here case this distinction is for.)"

  # ── Coverage self-enforcement (defect 2) ────────────────────────────
  Write-Host ""
  Write-Host "-- Coverage cross-check: replaying -Compare's verdict pass over the same loaded frames (pure pixel comparison, no capture) to obtain the authoritative list of registered verdict names --"
  $cmp = Invoke-CompareVerdicts $ctx
  if (-not $cmp.Ok) {
    Write-Host ""
    Write-Host "NOTE: the replayed -Compare pass reported FAIL above -- that is a real-evidence finding, independent of self-check coverage. Run -Compare directly for the full annotated report."
  }

  $missing = @($cmp.Names | Where-Object { $selfCheckNames -notcontains $_ })
  $extra = @($selfCheckNames | Where-Object { $cmp.Names -notcontains $_ })
  Write-Host ""
  Write-Host "-Compare registers $($cmp.Names.Count) verdicts. -SelfCheck exercises $($rows.Count) rows: $($rows.Count - $extra.Count) matching -Compare registrations, plus $($extra.Count) extra (the in-run guards -Compare never touches)."
  $coverageOk = ($missing.Count -eq 0)
  if (-not $coverageOk) {
    Write-Host "COVERAGE GAP: $($missing.Count) -Compare verdict(s) have no -SelfCheck row:"
    foreach ($m in $missing) { Write-Host "  - $m" }
  } else {
    Write-Host "Coverage complete: every -Compare-registered verdict has a matching -SelfCheck row."
  }

  foreach ($b in $ctx.Bitmaps) { $b.Dispose() }

  Write-Host ""
  if ($allFired -and $coverageOk) {
    Write-Host "PASS: every verdict fired red under its wrong pairing, and coverage is complete."
  } else {
    if (-not $allFired) { Write-Host "FAIL: at least one verdict passed under a wrong pairing -- see 'no' rows above." }
    if (-not $coverageOk) { Write-Host "FAIL: coverage gap -- see COVERAGE GAP list above." }
    exit 1
  }
}

if ($Capture) { Do-Capture }
if ($Compare) { Do-Compare }
if ($SelfCheck) { Do-SelfCheck }
exit 0
