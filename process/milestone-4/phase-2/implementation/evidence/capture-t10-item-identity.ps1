# M4-Phase 2 T10 item-identity positive control (assistant-automated GUI
# evidence).
#
# What it discriminates: whether a thumbnail click carries **which**
# thumbnail through to the rendered lightbox, in the shipped app, from a
# real host process. `docs/dsl_spec.md` §4.19 §Per-item handlers: a
# per-item handler's binder read resolves when the handler runs, so
# clicking thumbnail N must produce a lightbox whose caption reads
# `Photo #N`. A single frame of an open lightbox cannot support that
# claim -- an implementation that opened the lightbox from any click, or
# that captured a fixed index at generation time, would produce a
# frame that looks exactly the same. Two different thumbnails producing
# two different captions is what distinguishes them, and the same
# thumbnail twice producing the same caption is the leg that stops "the
# two frames differ" from being satisfiable by any incidental change.
#
# The state-level twin of this control is
# `wasamo-runtime/tests/gallery_slice_integration.rs`'s G1, which reads
# the same caption back as widget state. This script is the *rendered*
# half: it shows the pixels a user would see, from the release host
# binary, which no state read-back can show.
#
# Unlike capture-t7-scope-entry.ps1 this needs ONE build and ONE launch:
# both sides of the comparison are produced by the same binary, differing
# only in which thumbnail the script clicks. There is no operator step
# and no `.ui` edit -- the annotation this exercises is committed
# (`f1a6555`), which is exactly what T7's own re-audit said T10 owed.
#
#   closed   the lightbox absent -- the before-state, captured and
#            asserted rather than assumed.
#   a0       thumbnail 0 clicked -> lightbox open, caption "Photo #0".
#   a3       thumbnail 3 clicked (after Esc) -> caption "Photo #3".
#   a0b      thumbnail 0 clicked again (after Esc) -> caption "Photo #0".
#   k5/k6/k4 thumbnail 5 clicked, then ArrowRight, then ArrowLeft twice --
#            the same open lightbox throughout, so the ONLY thing that
#            differs between these three sets is the key that was pressed.
#            `key-down("ArrowLeft") / ("ArrowRight")` are declared on the
#            `modal-scope` container and the key walk is upward-only, so
#            they are reachable only because entry moved focus to a
#            descendant -- these frames are the rendered half of that.
#
#     caption  a0 vs a3  : THE DIFFERENCE LEG -- the identity of the
#                          clicked thumbnail must reach the rendered
#                          caption.
#     caption  a0 vs a0b : AN AGREEMENT LEG -- the same thumbnail clicked
#                          twice must render the same caption. Without
#                          it, "a0 and a3 differ" would be satisfied by
#                          any per-open variation (a different window
#                          position, a redraw artefact) and would say
#                          nothing about identity.
#     photo    a0 vs a3  : THE SECOND AGREEMENT LEG -- the lightbox's
#                          `[photo]` Box is a placeholder that does not
#                          depend on `selected_index` (the real image and
#                          the photo *name* need index reads, which are
#                          M4-Phase 3), so it must NOT differ between a0
#                          and a3. This is what localises the difference
#                          to the caption rather than to "the lightbox
#                          repainted".
#
# Both sample regions are derived from the lightbox `Grid`'s own track
# spec in `examples/gallery/gallery.ui`
# (`columns: 1* 56 400 56 1*`, `rows: 1* 44 300 64 1*`), which fixes each
# cell's fraction of the client rectangle independent of the absolute
# window size -- the same principle capture-t7-scope-entry.ps1 uses for
# its "<" / ">" search rectangles, and never a hand-worked-out pixel
# constant. The thumbnail click points are derived the same way, from the
# main `Grid`'s `rows: 56 1* 28` plus the `VStack`'s `padding: 12px` and
# the `WrapPanel`'s `item-cross-size: 88` / `item-spacing: 12`.
#
# Capture mechanics follow docs/notes/verification-environments.md
# Observation 4: PMv2 declared AND read back, CopyFromScreen (never
# PrintWindow) over the CLIENT rectangle mapped with ClientToScreen,
# window raised foreground+topmost, at least two frames per side, and --
# because this control sends a real Escape key -- foreground activation
# is earned with a real click and read back before any key is sent, with
# the input path used recorded in the output (capture-t5-focus.ps1's
# working shape).
param(
  [switch]$Capture,
  [switch]$Compare,
  [string]$OutDir = $PSScriptRoot,
  [string]$OutputPrefix = "t10-item-identity"
)

if (-not $Capture -and -not $Compare) {
  throw "pass -Capture, -Compare, or both"
}

if (-not ('WinT10Cap' -as [type])) {
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class WinT10Cap {
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
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
}
'@
}

# Per-Monitor-Aware V2, declared AND verified: declaring is not evidence
# that the declaration took effect (Observation 4 / Phase 1 F-48).
$PMV2 = [IntPtr](-4)
[WinT10Cap]::SetProcessDpiAwarenessContext($PMV2) | Out-Null
if (-not [WinT10Cap]::AreDpiAwarenessContextsEqual([WinT10Cap]::GetThreadDpiAwarenessContext(), $PMV2)) {
  throw "capture tool is not Per-Monitor-Aware V2; every rectangle below would be virtualized"
}
Add-Type -AssemblyName System.Drawing
New-Item -ItemType Directory -Force $OutDir | Out-Null

function Find-GalleryWindow($ProcessId) {
  $script:found = [IntPtr]::Zero
  [WinT10Cap]::EnumWindows({
    param($h, $l)
    $owner = 0
    [WinT10Cap]::GetWindowThreadProcessId($h, [ref]$owner) | Out-Null
    if ($owner -ne $ProcessId -or -not [WinT10Cap]::IsWindowVisible($h)) { return $true }
    $sb = [Text.StringBuilder]::new(256)
    [WinT10Cap]::GetWindowTextW($h, $sb, 256) | Out-Null
    if ($sb.ToString() -eq "Gallery") { $script:found = $h; return $false }
    return $true
  }, [IntPtr]::Zero) | Out-Null
  return $script:found
}

# The client rectangle in screen coordinates: both corners mapped with
# ClientToScreen, never derived from GetWindowRect minus a guessed frame.
function Client-ScreenRect($Handle) {
  $c = New-Object WinT10Cap+RECT
  [WinT10Cap]::GetClientRect($Handle, [ref]$c) | Out-Null
  $tl = New-Object WinT10Cap+POINT; $tl.X = $c.Left;  $tl.Y = $c.Top
  $br = New-Object WinT10Cap+POINT; $br.X = $c.Right; $br.Y = $c.Bottom
  [WinT10Cap]::ClientToScreen($Handle, [ref]$tl) | Out-Null
  [WinT10Cap]::ClientToScreen($Handle, [ref]$br) | Out-Null
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
  [WinT10Cap]::SetCursorPos($r.X + $ClientX, $r.Y + $ClientY) | Out-Null
  Start-Sleep -Milliseconds 400
}

function Click-At($Handle, $ClientX, $ClientY) {
  Move-To $Handle $ClientX $ClientY
  [WinT10Cap]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)   # LEFTDOWN
  Start-Sleep -Milliseconds 60
  [WinT10Cap]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)   # LEFTUP
  Start-Sleep -Milliseconds 900   # let the click's drain + re-layout settle
}

# Keyboard input goes to the focused window of the FOREGROUND thread, so
# activation is earned with a real click and read back, never requested
# and assumed (capture-t5-focus.ps1's header states the full reasoning).
# The activation click lands on the dark scroll-area background, whose
# `Box` carries no handler and is not focusable, so it changes nothing
# this capture measures.
function Try-Activate($Handle, $ClientX, $ClientY, $Attempts = 5) {
  for ($i = 1; $i -le $Attempts; $i++) {
    Click-At $Handle $ClientX $ClientY
    [WinT10Cap]::SetForegroundWindow($Handle) | Out-Null
    Start-Sleep -Milliseconds 300
    if ([WinT10Cap]::GetForegroundWindow() -eq $Handle) {
      if ($i -gt 1) { Write-Host "foreground acquired on attempt $i" }
      return $true
    }
    Write-Host ("attempt {0}/{1}: foreground is {2}, retrying" -f $i, $Attempts, [WinT10Cap]::GetForegroundWindow())
    Start-Sleep -Milliseconds 700
  }
  return $false
}

# A key press through the strongest input path this environment supports.
# The path actually used is printed, because the frames look identical
# either way and a silent fallback would be invisible in the artefact.
function Send-Key($Handle, [byte]$Vk) {
  if ($script:RealKeys) {
    if ([WinT10Cap]::GetForegroundWindow() -ne $Handle) {
      throw "the Gallery window lost foreground mid-capture; a real key press would go somewhere else"
    }
    [WinT10Cap]::keybd_event($Vk, 0, 0, [UIntPtr]::Zero)       # key down
    Start-Sleep -Milliseconds 40
    [WinT10Cap]::keybd_event($Vk, 0, 2, [UIntPtr]::Zero)       # KEYEVENTF_KEYUP
  } else {
    [WinT10Cap]::PostMessageW($Handle, 0x0100, [IntPtr]$Vk, [IntPtr]0) | Out-Null  # WM_KEYDOWN
  }
  Start-Sleep -Milliseconds 900
}

function Send-Escape($Handle) { Send-Key $Handle 0x1B }

# ── Capture mode ───────────────────────────────────────────────────────────

function Do-Capture() {
  $repo = (Resolve-Path "$PSScriptRoot\..\..\..\..\..").Path
  $exe = Join-Path $repo "target\release\gallery-rust.exe"
  if (-not (Test-Path $exe)) { throw "missing $exe (run cargo build --release --workspace first)" }

  $p = Start-Process -FilePath $exe -PassThru
  try {
    Start-Sleep -Seconds 3
    $h = Find-GalleryWindow $p.Id
    if ($h -eq [IntPtr]::Zero) { throw "no visible Gallery HWND (alive=$(-not $p.HasExited))" }

    [WinT10Cap]::ShowWindow($h, 9) | Out-Null
    [WinT10Cap]::SetWindowPos($h, [IntPtr](-1), 120, 120, 1000, 750, 0x0040) | Out-Null
    Start-Sleep -Milliseconds 1200

    $dpi = [WinT10Cap]::GetDpiForWindow($h)
    $scale = $dpi / 96.0
    $cr = Client-ScreenRect $h
    $cwDip = $cr.W / $scale
    $chDip = $cr.H / $scale
    Write-Host "Gallery HWND=$h dpi=$dpi scale=$scale client=$($cr.W)x$($cr.H) px = ${cwDip}x${chDip} DIP at ($($cr.X),$($cr.Y))"

    # Thumbnail centres, from the `.ui`'s own numbers (see the header):
    # main Grid `rows: 56 1* 28` puts the scroll area's top at y=56 DIP;
    # the VStack's `padding: 12px` and the WrapPanel's
    # `item-cross-size: 88` / `item-spacing: 12` put thumbnail i's centre
    # at (12 + i*100 + 44, 68 + 44) DIP on the first line.
    function Thumb-Point($i) {
      $x = [int]((12 + $i * 100 + 44) * $scale)
      $y = [int]((68 + 44) * $scale)
      return @($x, $y)
    }
    # The park / activation point: low in the dark scroll area, below the
    # thumbnail lines this window height shows, and clear of the status
    # strip. Its `Box` carries no handler and is not focusable.
    $parkX = [int]($cr.W * 0.5)
    $parkY = [int]($cr.H - 28 * $scale - 30 * $scale)

    $script:RealKeys = Try-Activate $h $parkX $parkY
    if ($script:RealKeys) {
      Write-Host "input path: REAL KEY PRESSES (keybd_event); foreground activation acquired and read back"
    } else {
      Write-Host "input path: POSTED WM_KEYDOWN (weaker claim: bypasses the OS input queue); foreground could not be acquired"
    }

    function Save-Set([string]$Tag, [int]$Count = 2) {
      for ($i = 0; $i -lt $Count; $i++) {
        $bmp = Capture-Client $h
        $out = Join-Path $OutDir "$OutputPrefix-$Tag-$i.png"
        $bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
        $bmp.Dispose()
        Write-Host "saved $out"
        Start-Sleep -Milliseconds 250
      }
    }

    # closed: the before-state, captured rather than assumed.
    Move-To $h $parkX $parkY
    Save-Set "closed"

    foreach ($step in @(@("a0", 0), @("a3", 3), @("a0b", 0))) {
      $tag = $step[0]; $idx = $step[1]
      $pt = Thumb-Point $idx
      Write-Host "clicking thumbnail $idx at client ($($pt[0]),$($pt[1])) px -> set '$tag'"
      Click-At $h $pt[0] $pt[1]
      # Park over the scrim, far left of the lightbox Grid's fixed
      # columns, so no lightbox control is hovered while frames are taken.
      Move-To $h ([int]($cr.W * 0.05)) ([int]($cr.H * 0.5))
      Save-Set $tag
      Send-Escape $h
      Move-To $h $parkX $parkY
    }

    # The arrow leg. `key-down("ArrowLeft")` / `("ArrowRight")` are
    # declared on the lightbox's `modal-scope` container, and the key walk
    # is upward-only, so they are reachable only because entry moved focus
    # to a descendant (the "<" Button). The rendered caption is the only
    # visible result this phase can produce -- the photo itself needs
    # index reads, which are M4-Phase 3 -- so it is what these frames
    # sample. Opened on thumbnail 5 so both directions stay in range.
    $pt5 = Thumb-Point 5
    Write-Host "clicking thumbnail 5 at client ($($pt5[0]),$($pt5[1])) px -> set 'k5'"
    Click-At $h $pt5[0] $pt5[1]
    Move-To $h ([int]($cr.W * 0.05)) ([int]($cr.H * 0.5))
    Save-Set "k5"
    Send-Key $h 0x27                       # VK_RIGHT
    Save-Set "k6"
    Send-Key $h 0x25                       # VK_LEFT
    Send-Key $h 0x25                       # VK_LEFT
    Save-Set "k4"
    Send-Escape $h
    Move-To $h $parkX $parkY

    # The lightbox must be closed again at the end: the same before-state
    # the run started from, which is what shows Escape actually closed it
    # rather than the frames merely differing.
    Save-Set "closed-after"

    $meta = "$scale`n$($cr.W)`n$($cr.H)`n$($script:RealKeys)"
    Set-Content -Path (Join-Path $OutDir "$OutputPrefix-meta.txt") -Value $meta
  } finally {
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
  }
}

# ── Compare mode ───────────────────────────────────────────────────────────

function Do-Compare() {
  function Load($Tag, $i) {
    $path = Join-Path $OutDir "$OutputPrefix-$Tag-$i.png"
    if (-not (Test-Path $path)) { throw "missing $path -- run -Capture first" }
    return New-Object System.Drawing.Bitmap $path
  }
  $metaLines = Get-Content (Join-Path $OutDir "$OutputPrefix-meta.txt")
  $scale = [double]$metaLines[0]
  $cw = [int]$metaLines[1]
  $ch = [int]$metaLines[2]
  Write-Host "Compare: scale=$scale client=${cw}x${ch} real-keys=$($metaLines[3])"

  $tags = @("closed", "a0", "a3", "a0b", "k5", "k6", "k4", "closed-after")
  $data = @{}
  $bmps = @()
  foreach ($t in $tags) {
    for ($i = 0; $i -lt 2; $i++) {
      $b = Load $t $i
      if ($b.Width -ne $cw -or $b.Height -ne $ch) {
        throw "$t-$i is $($b.Width)x$($b.Height), expected ${cw}x${ch} -- the window moved or resized mid-capture, so pixel regions would not line up"
      }
      $bmps += $b
      $data["$t$i"] = Bitmap-Bytes $b
    }
  }

  # Both sample regions come from the lightbox Grid's own track spec
  # (`columns: 1* 56 400 56 1*`, `rows: 1* 44 300 64 1*`), converted to
  # client pixels with the measured scale.
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

  Write-Host "caption region x[$col2x0..$col2x1] y[$capY0..$capY1]; photo region x[$col2x0..$col2x1] y[$photoY0..$photoY1]"

  # Count pixels whose summed channel difference exceeds a threshold well
  # above the text-pixel jitter F-33 measured (up to 13 per channel, so up
  # to 39 summed); 60 is comfortably above it and far below a glyph
  # appearing or disappearing.
  function Diff-Count($a, $b, $x0, $x1, $y0, $y1) {
    $n = 0
    for ($y = $y0; $y -lt $y1; $y++) {
      $row = $y * $a.Stride
      for ($x = $x0; $x -lt $x1; $x++) {
        $i = $row + $x * 4
        $d = [Math]::Abs([int]$a.Bytes[$i] - [int]$b.Bytes[$i]) +
             [Math]::Abs([int]$a.Bytes[$i+1] - [int]$b.Bytes[$i+1]) +
             [Math]::Abs([int]$a.Bytes[$i+2] - [int]$b.Bytes[$i+2])
        if ($d -gt 60) { $n++ }
      }
    }
    return $n
  }

  function Report($label, $a, $b, $x0, $x1, $y0, $y1) {
    $n = Diff-Count $data[$a] $data[$b] $x0 $x1 $y0 $y1
    Write-Host ("  {0,-46} {1,-10} vs {2,-10} : {3} differing px" -f $label, $a, $b, $n)
    return $n
  }

  Write-Host ""
  Write-Host "within-set jitter (same set, two frames -- the noise floor):"
  $jit = @()
  foreach ($t in @("a0", "a3", "a0b")) {
    $jit += Report "caption jitter" "${t}0" "${t}1" $col2x0 $col2x1 $capY0 $capY1
  }
  $noise = ($jit | Measure-Object -Maximum).Maximum
  Write-Host "  noise floor (max within-set caption jitter): $noise px"

  Write-Host ""
  Write-Host "legs:"
  $diff = Report "DIFFERENCE  caption, thumbnail 0 vs thumbnail 3" "a00" "a30" $col2x0 $col2x1 $capY0 $capY1
  $agree1 = Report "AGREEMENT   caption, thumbnail 0 twice" "a00" "a0b0" $col2x0 $col2x1 $capY0 $capY1
  $agree2 = Report "AGREEMENT   photo box, thumbnail 0 vs 3" "a00" "a30" $col2x0 $col2x1 $photoY0 $photoY1

  Write-Host ""
  Write-Host "arrow legs (same open lightbox, only the key pressed differs):"
  $kRight = Report "DIFFERENCE  caption, ArrowRight from #5" "k50" "k60" $col2x0 $col2x1 $capY0 $capY1
  $kLeft = Report "DIFFERENCE  caption, ArrowLeft x2 from #6" "k60" "k40" $col2x0 $col2x1 $capY0 $capY1
  $kPhoto = Report "AGREEMENT   photo box across the arrow keys" "k50" "k60" $col2x0 $col2x1 $photoY0 $photoY1

  # The lightbox itself must actually open and close: a whole-client
  # comparison of the closed before-state against an open one must differ
  # a lot, and the closed state before and after the whole run must agree.
  Write-Host ""
  Write-Host "open/close control (whole client):"
  $openDiff = Report "DIFFERENCE  closed vs lightbox open" "closed0" "a00" 0 $cw 0 $ch
  $closeAgree = Report "AGREEMENT   closed before vs closed after Esc" "closed0" "closed-after0" 0 $cw 0 $ch

  Write-Host ""
  $ok = $true
  if ($diff -le [Math]::Max($noise * 4, 40)) {
    Write-Host "FAIL: the caption does not differ between thumbnail 0 and thumbnail 3 -- item identity does not reach the rendered caption"; $ok = $false
  }
  if ($agree1 -gt [Math]::Max($noise * 4, 40)) {
    Write-Host "FAIL: the caption differs between two clicks of the SAME thumbnail -- the difference above cannot be attributed to identity"; $ok = $false
  }
  if ($agree2 -gt [Math]::Max($noise * 4, 40)) {
    Write-Host "FAIL: the photo box differs between thumbnail 0 and 3 -- the difference is not localised to the caption"; $ok = $false
  }
  if ($kRight -le [Math]::Max($noise * 4, 40)) {
    Write-Host "FAIL: ArrowRight did not change the caption -- the scope container's key-down handler did not reach the bound state"; $ok = $false
  }
  if ($kLeft -le [Math]::Max($noise * 4, 40)) {
    Write-Host "FAIL: ArrowLeft did not change the caption"; $ok = $false
  }
  if ($kPhoto -gt [Math]::Max($noise * 4, 40)) {
    Write-Host "FAIL: the photo box differs across the arrow keys -- the change is not localised to the caption"; $ok = $false
  }
  if ($openDiff -lt ($cw * $ch / 20)) {
    Write-Host "FAIL: the 'open' frames barely differ from the closed one -- the lightbox may not have opened at all"; $ok = $false
  }
  if ($closeAgree -gt ($cw * $ch / 200)) {
    Write-Host "FAIL: the client does not return to its closed appearance after Escape -- the lightbox did not close"; $ok = $false
  }
  if ($ok) { Write-Host "PASS: caption differs by thumbnail and by arrow key, agrees for the same thumbnail, photo box agrees, lightbox opens and closes" }

  foreach ($b in $bmps) { $b.Dispose() }
}

if ($Capture) { Do-Capture }
if ($Compare) { Do-Compare }
