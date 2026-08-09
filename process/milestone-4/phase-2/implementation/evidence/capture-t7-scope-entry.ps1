# M4-Phase 2 T7 modal-scope entry positive control (assistant-automated GUI
# evidence).
#
# What it discriminates: whether a modal scope's ENTRY actually paints the
# focus indicator on a node created moments earlier by the same drain, not
# merely whether the runtime's retained state says a path is focused.
# `docs/architecture.md` §13.4 / `docs/dsl_spec.md` §4.19: a subtree carrying
# `modal-scope: true` is entered by *being present* -- the materialisation
# seam captures the restore target and moves focus to the scope's first
# stop. `set_button_focused_at` then starts a colour animation on a brush
# belonging to a `Visual` built during that same message
# (`focus_mechanism_fixture.rs` / `modal_scope_integration.rs` pin the
# state side of this with `__focus_path_for_test` /
# `__button_focused_for_test`). No state read-back can show that the
# indicator actually *paints* -- `__button_focused_for_test` reports the
# same boolean whatever colour the brush ends up at (the same limitation
# capture-t5-focus.ps1's header records for the traversal case). Only a
# captured frame can.
#
# `examples/gallery/gallery.ui` is a tracked file that T10, not this task,
# owns landing the annotation in -- this capture is a THROWAWAY PROBE, and
# the annotation edit, the two builds, and the revert are OPERATOR STEPS
# taken between this script's invocations, not something the script itself
# performs: this script only launches an already-built exe, drives it, and
# captures or compares frames -- it never edits `gallery.ui`, never runs
# `cargo build`, and never runs `git checkout`. Because the probe requires
# two structurally-compared trees built from two different `gallery.ui`
# contents, one process launch cannot produce both frame sets the way
# capture-t5-focus.ps1's single launch produces its four -- E and U are two
# separate `cargo build --release --workspace` outputs the operator
# produces before the matching `-Variant` invocation. The full sequence,
# script steps and operator steps interleaved:
#
#   1. (operator) edit `examples/gallery/gallery.ui`: add `modal-scope: true`
#      and a `dismiss` handler to the lightbox's outer ZStack -- this is
#      the E tree.
#   2. (operator) `cargo build --release --workspace`.
#   3. (script)   .\capture-t7-scope-entry.ps1 -Variant E -OutDir <dir>
#   4. (operator) `git checkout -- examples/gallery/gallery.ui` -- reverts
#      to the U tree (both lines removed).
#   5. (operator) `cargo build --release --workspace`.
#   6. (script)   .\capture-t7-scope-entry.ps1 -Variant U -OutDir <dir>
#   7. (script)   .\capture-t7-scope-entry.ps1 -Compare   -OutDir <dir>
#
# Steps 3 and 6 each launch the already-built `target\release\gallery-rust.exe`,
# open the lightbox, and save two client-rectangle frames to `-OutDir`; step
# 7 loads both frame sets back from disk and reports -- it launches no
# process and touches no `.ui` source.
#
# The two frame sets, both at one window size and position, both opening the
# lightbox the same way and parking the cursor away from its controls
# afterwards (T4 measured that the toolbar hovers *through* the scrim, so a
# careless park point can contaminate a sample that has nothing to do with
# hover):
#
#   E  the lightbox's outer ZStack carries `modal-scope: true` and a
#      `dismiss` handler -> opening it enters the scope -> focus lands on
#      the scope's first stop, the "<" Button, which must show the focus
#      indicator.
#   U  the identical tree with both of those lines removed -> opening the
#      lightbox does nothing to focus -> "<" must show its plain colour.
#
#     "<"  E vs U : THE DIFFERENCE LEG -- entry must visibly change the
#                    first stop's own pixels.
#     ">"  E vs U : THE AGREEMENT LEG -- the second Button is unfocused in
#                    both trees, so it must NOT differ beyond jitter. Without
#                    this leg, "E and U differ" could equally be produced by
#                    any global shift (a different window position, a
#                    different backdrop tint) and would say nothing about
#                    focus specifically.
#
# The "<" / ">" pixel masks are derived ONCE, from a single reference frame
# (U's first frame -- the one with neither Button showing the indicator, so
# the effect under test cannot influence where it is measured, the same
# principle capture-t5-focus.ps1's header states for its own mask), and the
# identical pixel-index mask is then sampled in every frame of both sets.
# The mask itself is built the same way capture-t4-hover.ps1 /
# capture-t5-focus.ps1 build theirs: restrict to a small search region, then
# threshold individual pixels against a per-frame background sample. Here
# the search region is derived from the lightbox `Grid`'s own column/row
# spec (`columns: 1* 56 400 56 1*`, `rows: 1* 44 300 64 1*|), which fixes
# the "<" / ">" cells' fraction of the client rectangle independent of the
# absolute window size, and the threshold is "differs from a background
# swatch sampled from the empty flex row above the buttons (row 0) by more
# than a fixed sum-of-channel-deltas" rather than T4/T5's colour-family
# test, because what marks a Button here is translucency over the scrim
# (plain, or amber-tinted when focused), not a specific hue.
#
# Capture mechanics follow docs/notes/verification-environments.md
# Observation 4: PMv2 declared AND read back, CopyFromScreen (never
# PrintWindow) over the CLIENT rectangle mapped with ClientToScreen, window
# raised foreground+topmost, and at least two frames per side.
param(
  [ValidateSet("E", "U")]
  [string]$Variant,
  [switch]$Compare,
  [string]$OutDir = $PSScriptRoot,
  [string]$OutputPrefix = "t7-scope-entry"
)

if (-not $Variant -and -not $Compare) {
  throw "pass -Variant E, -Variant U, or -Compare"
}

if (-not ('WinT7Cap' -as [type])) {
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class WinT7Cap {
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
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
}
'@
}

# Per-Monitor-Aware V2, declared AND verified: declaring is not evidence that
# the declaration took effect (Observation 4 / Phase 1 F-48).
$PMV2 = [IntPtr](-4)
[WinT7Cap]::SetProcessDpiAwarenessContext($PMV2) | Out-Null
if (-not [WinT7Cap]::AreDpiAwarenessContextsEqual([WinT7Cap]::GetThreadDpiAwarenessContext(), $PMV2)) {
  throw "capture tool is not Per-Monitor-Aware V2; every rectangle below would be virtualized"
}
Add-Type -AssemblyName System.Drawing

New-Item -ItemType Directory -Force $OutDir | Out-Null

function Find-GalleryWindow($ProcessId) {
  $script:found = [IntPtr]::Zero
  [WinT7Cap]::EnumWindows({
    param($h, $l)
    $owner = 0
    [WinT7Cap]::GetWindowThreadProcessId($h, [ref]$owner) | Out-Null
    if ($owner -ne $ProcessId -or -not [WinT7Cap]::IsWindowVisible($h)) { return $true }
    $sb = [Text.StringBuilder]::new(256)
    [WinT7Cap]::GetWindowTextW($h, $sb, 256) | Out-Null
    if ($sb.ToString() -eq "Gallery") { $script:found = $h; return $false }
    return $true
  }, [IntPtr]::Zero) | Out-Null
  return $script:found
}

# The client rectangle in screen coordinates: both corners mapped with
# ClientToScreen, never derived from GetWindowRect minus a guessed frame.
function Client-ScreenRect($Handle) {
  $c = New-Object WinT7Cap+RECT
  [WinT7Cap]::GetClientRect($Handle, [ref]$c) | Out-Null
  $tl = New-Object WinT7Cap+POINT; $tl.X = $c.Left;  $tl.Y = $c.Top
  $br = New-Object WinT7Cap+POINT; $br.X = $c.Right; $br.Y = $c.Bottom
  [WinT7Cap]::ClientToScreen($Handle, [ref]$tl) | Out-Null
  [WinT7Cap]::ClientToScreen($Handle, [ref]$br) | Out-Null
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

function Move-To($Handle, $ClientX, $ClientY) {
  $r = Client-ScreenRect $Handle
  [WinT7Cap]::SetCursorPos($r.X + $ClientX, $r.Y + $ClientY) | Out-Null
  Start-Sleep -Milliseconds 450   # let any hover-in / hover-out animation settle
}

function Click-At($Handle, $ClientX, $ClientY) {
  Move-To $Handle $ClientX $ClientY
  [WinT7Cap]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)   # LEFTDOWN
  Start-Sleep -Milliseconds 60
  [WinT7Cap]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)   # LEFTUP
  Start-Sleep -Milliseconds 900   # let the click's drain + entry's colour animation settle
}

# ── Capture mode: launch, open the lightbox, save >= 2 client frames ───────

function Capture-Variant([string]$V) {
  $repo = (Resolve-Path "$PSScriptRoot\..\..\..\..\..").Path
  $exe = Join-Path $repo "target\release\gallery-rust.exe"
  if (-not (Test-Path $exe)) { throw "missing $exe (run cargo build --release --workspace first)" }

  $p = Start-Process -FilePath $exe -PassThru
  try {
    Start-Sleep -Seconds 3
    $h = Find-GalleryWindow $p.Id
    if ($h -eq [IntPtr]::Zero) { throw "no visible Gallery HWND (alive=$(-not $p.HasExited))" }

    [WinT7Cap]::ShowWindow($h, 9) | Out-Null
    [WinT7Cap]::SetWindowPos($h, [IntPtr](-1), 120, 120, 1000, 750, 0x0040) | Out-Null
    [WinT7Cap]::SetForegroundWindow($h) | Out-Null
    Start-Sleep -Milliseconds 1200

    $dpi = [WinT7Cap]::GetDpiForWindow($h)
    $scale = $dpi / 96.0
    $cr = Client-ScreenRect $h
    Write-Host "[$V] Gallery HWND=$h dpi=$dpi scale=$scale client=$($cr.W)x$($cr.H) at ($($cr.X),$($cr.Y))"

    # Locate the accent-styled "Open lightbox" Button by colour predicate --
    # the same blue-dominance test capture-t4-hover.ps1 /
    # capture-t5-focus.ps1 use for the checked ToggleButton's blue, applied
    # to the RIGHT half of the 56-DIP toolbar band instead of the left half
    # (the toolbar's right HStack holds "Scroll down" / "Scroll up" / "Open
    # lightbox"; only the accent-styled Button is blue-dominant, so the
    # predicate isolates it from its siblings without any hand-worked-out
    # coordinate).
    $b0 = Capture-Client $h
    $d0 = Bitmap-Bytes $b0
    $bandH = [Math]::Min([int](56 * $scale), $d0.H)
    $halfW = [int]($d0.W / 2)
    $minX = [int]::MaxValue; $maxX = -1; $minY = [int]::MaxValue; $maxY = -1
    $cnt = 0
    for ($y = 0; $y -lt $bandH; $y++) {
      $row = $y * $d0.Stride
      for ($x = $halfW; $x -lt $d0.W; $x++) {
        $i = $row + $x * 4
        $bl = $d0.Bytes[$i]; $gr = $d0.Bytes[$i+1]; $rd = $d0.Bytes[$i+2]
        if ($bl -gt ($rd + 60) -and $bl -gt 120 -and $gr -gt $rd) {
          $cnt++
          if ($x -lt $minX) { $minX = $x }; if ($x -gt $maxX) { $maxX = $x }
          if ($y -lt $minY) { $minY = $y }; if ($y -gt $maxY) { $maxY = $y }
        }
      }
    }
    $b0.Dispose()
    if ($cnt -lt 50) {
      throw "[$V] the accent 'Open lightbox' Button was not found by the blue predicate in the toolbar's right half (mask=$cnt px); cannot proceed"
    }
    Write-Host "[$V] 'Open lightbox' mask: $cnt px, bbox client x[$minX..$maxX] y[$minY..$maxY]"
    $openX = [int](($minX + $maxX) / 2)
    $openY = [int](($minY + $maxY) / 2)

    Click-At $h $openX $openY

    # Park the cursor well away from every lightbox control: the middle of
    # the client, which lands over the lightbox's own photo/text column
    # (Grid column index 2, a Box/Text with no hover behaviour), never over
    # "<" or ">" (columns 1 / 3). T4 measured that the toolbar hovers
    # *through* the scrim pre-fix; the equivalent risk here is contaminating
    # "<" / ">" with a stray hover, which this parking point avoids by
    # construction (it is nowhere near either Button's column).
    $parkX = [int]($cr.W * 0.5)
    $parkY = [int]($cr.H * 0.75)
    Move-To $h $parkX $parkY

    $frames = @()
    for ($i = 0; $i -lt 2; $i++) {
      $bmp = Capture-Client $h
      $out = Join-Path $OutDir "$OutputPrefix-$V-$i.png"
      $bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
      Write-Host "[$V] saved $out"
      $frames += $bmp
      Start-Sleep -Milliseconds 250
    }
    foreach ($f in $frames) { $f.Dispose() }

    # Record the measured scale/client rect beside the frames so -Compare
    # does not have to re-derive geometry from a live window.
    $meta = "$scale`n$($cr.W)`n$($cr.H)"
    Set-Content -Path (Join-Path $OutDir "$OutputPrefix-$V-meta.txt") -Value $meta
  } finally {
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
  }
}

# ── Compare mode: load both sets from disk, derive masks once, report ──────

function Compare-Sets() {
  function Load($V, $i) {
    $path = Join-Path $OutDir "$OutputPrefix-$V-$i.png"
    if (-not (Test-Path $path)) { throw "missing $path -- run -Variant $V first" }
    return New-Object System.Drawing.Bitmap $path
  }
  $frames = @{
    E0 = Load "E" 0; E1 = Load "E" 1
    U0 = Load "U" 0; U1 = Load "U" 1
  }
  $metaLines = Get-Content (Join-Path $OutDir "$OutputPrefix-U-meta.txt")
  $scale = [double]$metaLines[0]
  $cw = [int]$metaLines[1]
  $ch = [int]$metaLines[2]
  Write-Host "Compare: scale=$scale client=${cw}x${ch} (from U's recorded metadata)"

  foreach ($k in $frames.Keys) {
    if ($frames[$k].Width -ne $cw -or $frames[$k].Height -ne $ch) {
      throw "$k frame is $($frames[$k].Width)x$($frames[$k].Height), expected ${cw}x${ch} -- E and U were not captured at the same window geometry, so pixel masks would not line up"
    }
  }

  $data = @{}
  foreach ($k in $frames.Keys) { $data[$k] = Bitmap-Bytes $frames[$k] }

  # The lightbox `Grid`'s own spec (examples/gallery/gallery.ui):
  #   columns: 1* 56 400 56 1*      rows: 1* 44 300 64 1*
  # "<" sits at column 1 (56 DIP), row 2..3 (300+64 DIP); ">" at column 3,
  # same rows. These fractions are fixed by the DSL source, independent of
  # the absolute client size, so the search rectangle below is derived from
  # them rather than a hand-worked-out pixel constant.
  # No outward padding on these bounds: a live per-pixel scan of the saved
  # frames found the column-2 photo Box's own fill (`#5b7080`) starting
  # right at the column-1/column-2 boundary with no soft edge, so a +/-4
  # px pad (tried first) pulled a constant strip of the photo Box's colour
  # into the "<" mask at every row -- present in every frame identically,
  # so it did not move the E-vs-U delta, but it diluted the mask's own
  # meaning away from "the Button's pixels". The bounds below sit exactly
  # on the Grid's own column edges instead.
  $fixedColW = (56 + 400 + 56) * $scale
  $flexColW = ($cw - $fixedColW) / 2.0
  $col1x0 = [int]($flexColW); $col1x1 = [int]($flexColW + 56 * $scale)
  $col3x0 = [int]($flexColW + (56 + 400) * $scale); $col3x1 = [int]($flexColW + (56 + 400 + 56) * $scale)

  $fixedRowH = (44 + 300 + 64) * $scale
  $flexRowH = ($ch - $fixedRowH) / 2.0
  $row0y = [int]($flexRowH * 0.4)             # background sample row: the empty top flex row
  # The buttons are v-align:center **within** the combined row-2+3 span, so
  # their true footprint sits tightly around that span's midpoint, not
  # spread across the whole 364-DIP span. A first pass that searched the
  # whole span (measured: mask bbox extended far past the button itself,
  # y[152..526]) turned out to include faint edge bleed from the row-2
  # photo Box's boundary (own fill #5b7080, own bottom edge sits right at
  # that span's midpoint + half its height) -- the photo Box's edge
  # antialiasing is visible several pixels into the neighbouring column at
  # a handful of scattered rows. A live per-pixel scan of the saved frames
  # (both variants) found the actual button glyph confined to a ~35-DIP
  # band centred on the span's midpoint; searching a generous +/-50 DIP
  # around that midpoint keeps the real button comfortably inside while
  # excluding the photo Box's edge, which sits roughly +/-115 DIP away.
  $rowMidY = $flexRowH + (44 + 300 + 64) * $scale / 2.0
  $rowBtnY0 = [int]($rowMidY - 50 * $scale)
  $rowBtnY1 = [int]($rowMidY + 50 * $scale)

  Write-Host "search rects: '<' x[$col1x0..$col1x1] '>' x[$col3x0..$col3x1] y[$rowBtnY0..$rowBtnY1] (centred +/-50 DIP on the row-2+3 midpoint), bg sample row y=$row0y"

  # Reference frame: U0 -- neither Button shows the indicator in the U tree,
  # so the mask cannot be shaped by the very effect it will measure (the
  # same principle capture-t5-focus.ps1 applies to its own baseline frame).
  $ref = $data["U0"]

  function Px($d, $x, $y) {
    $i = $y * $d.Stride + $x * 4
    return @{ B = [int]$d.Bytes[$i]; G = [int]$d.Bytes[$i+1]; R = [int]$d.Bytes[$i+2] }
  }

  function Build-Mask($x0, $x1, $bgX) {
    $bg = Px $ref $bgX $row0y
    $mask = New-Object System.Collections.Generic.List[int]
    $minX = [int]::MaxValue; $maxX = -1; $minY = [int]::MaxValue; $maxY = -1
    for ($y = $rowBtnY0; $y -lt $rowBtnY1; $y++) {
      $row = $y * $ref.Stride
      for ($x = $x0; $x -lt $x1; $x++) {
        $i = $row + $x * 4
        $bl = [int]$ref.Bytes[$i]; $gr = [int]$ref.Bytes[$i+1]; $rd = [int]$ref.Bytes[$i+2]
        $diff = [Math]::Abs($rd - $bg.R) + [Math]::Abs($gr - $bg.G) + [Math]::Abs($bl - $bg.B)
        # 45: comfortably above the ~28-sum difference measured between the
        # scrim's own two flat dither shades (background-vs-background), and
        # below the weakest real button-edge pixel measured (~51 sum, the
        # unfocused Button's soft edge) up to its focused core (~283 sum).
        if ($diff -gt 45) {
          $mask.Add($i) | Out-Null
          if ($x -lt $minX) { $minX = $x }; if ($x -gt $maxX) { $maxX = $x }
          if ($y -lt $minY) { $minY = $y }; if ($y -gt $maxY) { $maxY = $y }
        }
      }
    }
    return @{ Mask = $mask; BBox = "x[$minX..$maxX] y[$minY..$maxY]"; Bg = $bg }
  }

  $lt = Build-Mask $col1x0 $col1x1 ([int](($col1x0 + $col1x1) / 2))
  $gt = Build-Mask $col3x0 $col3x1 ([int](($col3x0 + $col3x1) / 2))

  foreach ($pair in @(@("<", $lt), @(">", $gt))) {
    $label = $pair[0]; $r = $pair[1]
    Write-Host ("mask '{0}': {1} px, bbox {2}, background R={3} G={4} B={5}" -f $label, $r.Mask.Count, $r.BBox, $r.Bg.R, $r.Bg.G, $r.Bg.B)
    if ($r.Mask.Count -lt 30) {
      throw "mask '$label' found only $($r.Mask.Count) px -- the button region was not reliably located; the comparisons below would be vacuous"
    }
  }

  function Mask-Mean($d, $mask) {
    $sb = 0.0; $sg = 0.0; $sr = 0.0
    foreach ($i in $mask) { $sb += $d.Bytes[$i]; $sg += $d.Bytes[$i+1]; $sr += $d.Bytes[$i+2] }
    $n = $mask.Count
    return @{ R = [Math]::Round($sr / $n, 2); G = [Math]::Round($sg / $n, 2); B = [Math]::Round($sb / $n, 2) }
  }

  $means = @{}
  foreach ($side in @("E0","E1","U0","U1")) {
    $means["lt_$side"] = Mask-Mean $data[$side] $lt.Mask
    $means["gt_$side"] = Mask-Mean $data[$side] $gt.Mask
  }

  Write-Host ""
  Write-Host "per-frame masked means:"
  foreach ($btn in @("lt","gt")) {
    foreach ($side in @("E0","E1","U0","U1")) {
      $m = $means["${btn}_$side"]
      Write-Host ("  {0,-3} {1,-3} R={2,7} G={3,7} B={4,7}" -f $btn, $side, $m.R, $m.G, $m.B)
    }
  }

  function Delta($btn, $p, $q) {
    $p0 = $means["${btn}_${p}0"]; $p1 = $means["${btn}_${p}1"]
    $q0 = $means["${btn}_${q}0"]; $q1 = $means["${btn}_${q}1"]
    return @{
      R = [Math]::Round((($p0.R + $p1.R) - ($q0.R + $q1.R)) / 2, 2)
      G = [Math]::Round((($p0.G + $p1.G) - ($q0.G + $q1.G)) / 2, 2)
      B = [Math]::Round((($p0.B + $p1.B) - ($q0.B + $q1.B)) / 2, 2)
    }
  }
  function Jitter($btn, $side) {
    $a = $means["${btn}_${side}0"]; $b = $means["${btn}_${side}1"]
    return @{ R = [Math]::Round($a.R - $b.R, 2); G = [Math]::Round($a.G - $b.G, 2); B = [Math]::Round($a.B - $b.B, 2) }
  }
  function MaxAbs($d) { return [Math]::Max([Math]::Abs($d.R), [Math]::Max([Math]::Abs($d.G), [Math]::Abs($d.B))) }

  $jLtE = Jitter "lt" "E"; $jLtU = Jitter "lt" "U"
  $jGtE = Jitter "gt" "E"; $jGtU = Jitter "gt" "U"
  $noise = [Math]::Max([Math]::Max((MaxAbs $jLtE), (MaxAbs $jLtU)), [Math]::Max((MaxAbs $jGtE), (MaxAbs $jGtU)))
  $tolerance = [Math]::Max($noise, 3.0)   # matches T5's floor form; the jitter measured here decides the actual value used

  $dLt = Delta "lt" "E" "U"   # DIFFERENCE LEG: "<" must differ, E has entry, U does not
  $dGt = Delta "gt" "E" "U"   # AGREEMENT LEG:  ">" must NOT differ -- unfocused in both trees

  Write-Host ""
  Write-Host ("within-side jitter '<' E / U : {0} / {1}   '>' E / U : {2} / {3}" -f (MaxAbs $jLtE), (MaxAbs $jLtU), (MaxAbs $jGtE), (MaxAbs $jGtU))
  Write-Host ("tolerance applied (max jitter observed, floor 3.0): {0}" -f $tolerance)
  Write-Host ("'<' E - U  (DIFFERENCE LEG -- entry must paint the indicator) : R={0} G={1} B={2}  max-abs={3}" -f $dLt.R, $dLt.G, $dLt.B, (MaxAbs $dLt))
  Write-Host ("'>' E - U  (AGREEMENT LEG  -- unfocused in both, must be ~0)  : R={0} G={1} B={2}  max-abs={3}" -f $dGt.R, $dGt.G, $dGt.B, (MaxAbs $dGt))
  Write-Host ""

  $ok = $true
  if ((MaxAbs $dLt) -le $tolerance) { Write-Host "FAIL: '<' does not differ between E and U beyond tolerance -- entry does not appear to paint the indicator"; $ok = $false }
  if ((MaxAbs $dGt) -gt $tolerance) { Write-Host "FAIL: '>' differs between E and U beyond tolerance -- the difference above cannot be attributed to focus alone"; $ok = $false }
  if ($ok) { Write-Host "PASS: '<' differs between E and U beyond tolerance; '>' agrees within tolerance" }

  foreach ($f in $frames.Values) { $f.Dispose() }
}

if ($Compare) {
  Compare-Sets
} else {
  Capture-Variant $Variant
}
