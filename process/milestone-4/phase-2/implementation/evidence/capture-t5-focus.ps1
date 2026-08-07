# M4-Phase 2 T5 focus-indicator positive control (assistant-automated GUI
# evidence).
#
# What it discriminates: whether the focus indicator is **visibly distinct**
# from the two other states that share its only means. Until M5's theming
# surface there are no borders and no focus rings (docs/dsl_spec.md §4.18),
# so M4 paints focus as a background change - and so do hover and the
# ToggleButton selected state. DD-M4-P2-003: "The focused state must be
# visibly distinct from both, or a captured frame cannot say which stop
# holds focus and the traversal-order control degrades to 'something
# changed'."
#
# No state read-back can answer this. `__button_focused_for_test` reports
# the same boolean whatever colour the brush ends up at - measured: the
# mutation that makes `focus_indicator_color` the identity function reddens
# only the pure colour unit test, and none of the five integration
# fixtures. Only a captured frame can.
#
# The sample is the gallery's **first Tab stop**, which is the checked
# "All" ToggleButton - the hardest of the three comparisons rather than a
# constructed one, because a checked ToggleButton is already painting the
# selected colour before focus arrives.
#
# The four frame sets, all at one window size and position, all sampling
# the SAME pixel mask (derived once from the baseline frame by the same
# blue predicate T4's capture used, so no frame that shows the effect can
# influence where the effect is measured):
#
#   N  nothing focused, cursor parked away  -> "All" selected, plain
#   H  nothing focused, cursor over "All"   -> "All" selected + hovered
#   FA Tab x1, cursor parked away           -> "All" selected + FOCUSED
#   FB Tab x4, cursor parked away           -> focus has moved on
#
#     FA vs N : focused must differ from selected      (DD-M4-P2-003)
#     FA vs H : focused must differ from hovered       (DD-M4-P2-003)
#     FB vs N : THE AGREEMENT LEG - when focus moves to the fourth stop,
#               "All" must return to exactly the colour it had at N. An
#               implementation that painted the indicator but never cleared
#               it would give FB ~ FA here and pass every difference leg
#               above; this is the leg that fails against it.
#
# Capture mechanics follow docs/notes/verification-environments.md
# Observation 4: PMv2 declared AND read back, CopyFromScreen (never
# PrintWindow) over the CLIENT rectangle mapped with ClientToScreen, window
# raised foreground+topmost, and at least two frames per side.
param(
  [string]$OutDir = $PSScriptRoot,
  [string]$OutputPrefix = "t5-focus",
  [switch]$KeepFrames
)

if (-not ('WinT5Cap' -as [type])) {
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class WinT5Cap {
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

# Per-Monitor-Aware V2, declared AND verified: declaring is not evidence that
# the declaration took effect (Observation 4 / Phase 1 F-48).
$PMV2 = [IntPtr](-4)
[WinT5Cap]::SetProcessDpiAwarenessContext($PMV2) | Out-Null
if (-not [WinT5Cap]::AreDpiAwarenessContextsEqual([WinT5Cap]::GetThreadDpiAwarenessContext(), $PMV2)) {
  throw "capture tool is not Per-Monitor-Aware V2; every rectangle below would be virtualized"
}
Add-Type -AssemblyName System.Drawing

$repo = (Resolve-Path "$PSScriptRoot\..\..\..\..\..").Path
$exe = Join-Path $repo "target\release\gallery-rust.exe"
if (-not (Test-Path $exe)) { throw "missing $exe (run cargo build --release --workspace first)" }
New-Item -ItemType Directory -Force $OutDir | Out-Null

function Find-GalleryWindow($ProcessId) {
  $script:found = [IntPtr]::Zero
  [WinT5Cap]::EnumWindows({
    param($h, $l)
    $owner = 0
    [WinT5Cap]::GetWindowThreadProcessId($h, [ref]$owner) | Out-Null
    if ($owner -ne $ProcessId -or -not [WinT5Cap]::IsWindowVisible($h)) { return $true }
    $sb = [Text.StringBuilder]::new(256)
    [WinT5Cap]::GetWindowTextW($h, $sb, 256) | Out-Null
    if ($sb.ToString() -eq "Gallery") { $script:found = $h; return $false }
    return $true
  }, [IntPtr]::Zero) | Out-Null
  return $script:found
}

# The client rectangle in screen coordinates: both corners mapped with
# ClientToScreen, never derived from GetWindowRect minus a guessed frame.
function Client-ScreenRect($Handle) {
  $c = New-Object WinT5Cap+RECT
  [WinT5Cap]::GetClientRect($Handle, [ref]$c) | Out-Null
  $tl = New-Object WinT5Cap+POINT; $tl.X = $c.Left;  $tl.Y = $c.Top
  $br = New-Object WinT5Cap+POINT; $br.X = $c.Right; $br.Y = $c.Bottom
  [WinT5Cap]::ClientToScreen($Handle, [ref]$tl) | Out-Null
  [WinT5Cap]::ClientToScreen($Handle, [ref]$br) | Out-Null
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
  [WinT5Cap]::SetCursorPos($r.X + $ClientX, $r.Y + $ClientY) | Out-Null
  Start-Sleep -Milliseconds 450   # let the hover-in / hover-out animation settle
}

# Try to earn foreground activation the way a user does, and REPORT whether
# it worked rather than assuming either answer.
#
# `SetForegroundWindow` alone is refused by Windows unless the caller is
# already foreground, so a real click inside the client area is attempted
# first - that is what a user does and what the OS accepts. The park
# position is used deliberately: it is inside the dark scroll area, whose
# `Box` carries no handler and is not focusable, so the click changes
# nothing this capture measures (docs/dsl_spec.md §4.19 "clicking
# background never clears focus").
#
# Returns $true when the window really is foreground afterwards. A `$false`
# here is an environment fact, not a failure: a session with **no**
# foreground window at all (`GetForegroundWindow` returns 0) cannot deliver
# synthesized keyboard input to any window, because `keybd_event` is
# delivered to the focused window of the foreground thread.
function Try-Activate($Handle, $ClientX, $ClientY) {
  Move-To $Handle $ClientX $ClientY
  [WinT5Cap]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)   # LEFTDOWN
  Start-Sleep -Milliseconds 60
  [WinT5Cap]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)   # LEFTUP
  Start-Sleep -Milliseconds 600
  [WinT5Cap]::SetForegroundWindow($Handle) | Out-Null
  Start-Sleep -Milliseconds 300
  return ([WinT5Cap]::GetForegroundWindow() -eq $Handle)
}

# Advance the focus by `Times` Tab presses, through the strongest input path
# this environment actually supports.
#
# `$script:RealKeys` decides, and it is set from the measurement above
# rather than predicted:
#
#   $true  - `keybd_event`, a real OS key press, delivered by the input
#            system to the foreground window. This exercises the whole
#            path: keyboard -> queue -> `GetMessage` -> `DispatchMessage`
#            -> `wnd_proc`'s WM_KEYDOWN arm.
#   $false - `PostMessageW(WM_KEYDOWN, VK_TAB)`, which still goes through
#            the window's real message loop (`wasamo_runtime::run`'s
#            `GetMessage` / `DispatchMessage`, hence the message-loop drain
#            boundary) but is **posted by this tool rather than produced by
#            the OS input stack**. That is a WEAKER claim and is printed as
#            such: it says nothing about OS keyboard delivery. It is
#            adequate for what this capture is for - the indicator's
#            appearance - because the key path itself is pinned by
#            `tests/focus_traversal_integration.rs`, which drives the same
#            arm through real messages.
function Press-Tab($Handle, $Times) {
  for ($i = 0; $i -lt $Times; $i++) {
    if ($script:RealKeys) {
      if ([WinT5Cap]::GetForegroundWindow() -ne $Handle) {
        throw "the Gallery window lost foreground mid-capture; a real Tab press would go somewhere else"
      }
      [WinT5Cap]::keybd_event(0x09, 0, 0, [UIntPtr]::Zero)      # VK_TAB down
      Start-Sleep -Milliseconds 40
      [WinT5Cap]::keybd_event(0x09, 0, 2, [UIntPtr]::Zero)      # KEYEVENTF_KEYUP
    } else {
      [WinT5Cap]::PostMessageW($Handle, 0x0100, [IntPtr]0x09, [IntPtr]0) | Out-Null  # WM_KEYDOWN VK_TAB
    }
    Start-Sleep -Milliseconds 400                               # settle the colour animation
  }
}

$p = Start-Process -FilePath $exe -PassThru
try {
  Start-Sleep -Seconds 3
  $h = Find-GalleryWindow $p.Id
  if ($h -eq [IntPtr]::Zero) { throw "no visible Gallery HWND (alive=$(-not $p.HasExited))" }

  [WinT5Cap]::ShowWindow($h, 9) | Out-Null
  [WinT5Cap]::SetWindowPos($h, [IntPtr](-1), 120, 120, 1000, 750, 0x0040) | Out-Null
  Start-Sleep -Milliseconds 1200

  $dpi = [WinT5Cap]::GetDpiForWindow($h)
  $scale = $dpi / 96.0
  $cr = Client-ScreenRect $h
  Write-Host "Gallery HWND=$h dpi=$dpi scale=$scale client=$($cr.W)x$($cr.H) at ($($cr.X),$($cr.Y))"

  # A parking spot inside the dark scroll area: a `Box`, never a hit target
  # that paints anything, and far from every Button.
  $parkX = [int]($cr.W * 0.5)
  $parkY = [int]($cr.H * 0.75)

  # Measure the input capability before choosing how to press Tab, rather
  # than predicting it (the T4 retrospective's start-gate corrective applied
  # to the capture tool).
  $fgBefore = [WinT5Cap]::GetForegroundWindow()
  $script:RealKeys = Try-Activate $h $parkX $parkY
  if ($script:RealKeys) {
    Write-Host "input path: REAL KEY PRESSES (keybd_event); the window holds foreground activation"
  } else {
    Write-Host ("input path: POSTED WM_KEYDOWN (weaker claim). The window could not take foreground " +
                "activation even after a real click; GetForegroundWindow was $fgBefore before and " +
                "$([WinT5Cap]::GetForegroundWindow()) after. A session whose foreground window is 0 has no " +
                "foreground thread for synthesized keyboard input to reach, so keybd_event would go " +
                "nowhere. The posted message still travels the window's real message loop and the real " +
                "WM_KEYDOWN arm, but this capture makes NO claim about OS keyboard delivery.")
  }

  # ---- N: nothing focused, cursor away (two frames) ---------------------
  Move-To $h $parkX $parkY
  $frames = @{}
  $frames["N0"] = Capture-Client $h
  Start-Sleep -Milliseconds 250
  $frames["N1"] = Capture-Client $h

  # The sample mask: the checked "All" ToggleButton's own blue, found in the
  # left half of the 56-DIP toolbar band of the BASELINE frame - the frame in
  # which no effect under test is present. Derived once and reused for every
  # frame, so no later state can move where it is measured.
  $n0 = Bitmap-Bytes $frames["N0"]
  $bandH = [Math]::Min([int](56 * $scale), $n0.H)
  $halfW = [int]($n0.W / 2)
  $mask = New-Object System.Collections.Generic.List[int]
  $minX = [int]::MaxValue; $maxX = -1; $minY = [int]::MaxValue; $maxY = -1
  for ($y = 0; $y -lt $bandH; $y++) {
    $row = $y * $n0.Stride
    for ($x = 0; $x -lt $halfW; $x++) {
      $i = $row + $x * 4
      $bl = $n0.Bytes[$i]; $gr = $n0.Bytes[$i+1]; $rd = $n0.Bytes[$i+2]
      if ($bl -gt ($rd + 60) -and $bl -gt 120 -and $gr -gt $rd) {
        $mask.Add($i) | Out-Null
        if ($x -lt $minX) { $minX = $x }; if ($x -gt $maxX) { $maxX = $x }
        if ($y -lt $minY) { $minY = $y }; if ($y -gt $maxY) { $maxY = $y }
      }
    }
  }
  if ($mask.Count -lt 200) {
    throw "the checked ToggleButton's blue was not found in the toolbar band (mask=$($mask.Count) px); the sample region cannot be established"
  }
  Write-Host "mask: $($mask.Count) px, bbox client x[$minX..$maxX] y[$minY..$maxY]"
  $hoverX = [int](($minX + $maxX) / 2)
  $hoverY = [int](($minY + $maxY) / 2)

  function Mask-Mean($bmp) {
    $d = Bitmap-Bytes $bmp
    $sb = 0.0; $sg = 0.0; $sr = 0.0
    foreach ($i in $mask) { $sb += $d.Bytes[$i]; $sg += $d.Bytes[$i+1]; $sr += $d.Bytes[$i+2] }
    $n = $mask.Count
    return @{ R = [Math]::Round($sr / $n, 2); G = [Math]::Round($sg / $n, 2); B = [Math]::Round($sb / $n, 2) }
  }

  # ---- H: nothing focused, cursor over "All" ----------------------------
  Move-To $h $hoverX $hoverY
  $frames["H0"] = Capture-Client $h
  Start-Sleep -Milliseconds 250
  $frames["H1"] = Capture-Client $h

  # Park the cursor again before any Tab: "All" must be focused-and-not-hovered
  # for FA to be a comparison against H rather than a mixture of the two.
  Move-To $h $parkX $parkY

  # ---- FA: Tab x1 -> the first stop is focused --------------------------
  Press-Tab $h 1
  $frames["FA0"] = Capture-Client $h
  Start-Sleep -Milliseconds 250
  $frames["FA1"] = Capture-Client $h

  # ---- FB: three more Tabs -> focus has moved off "All" -----------------
  Press-Tab $h 3
  $frames["FB0"] = Capture-Client $h
  Start-Sleep -Milliseconds 250
  $frames["FB1"] = Capture-Client $h

  $means = @{}
  foreach ($k in @("N0","N1","H0","H1","FA0","FA1","FB0","FB1")) { $means[$k] = Mask-Mean $frames[$k] }
  foreach ($k in @("N0","N1","H0","H1","FA0","FA1","FB0","FB1")) {
    $m = $means[$k]
    Write-Host ("{0,-4} R={1,7} G={2,7} B={3,7}" -f $k, $m.R, $m.G, $m.B)
  }

  function Delta($p, $q) {
    return @{
      R = [Math]::Round((($means["$($p)0"].R + $means["$($p)1"].R) - ($means["$($q)0"].R + $means["$($q)1"].R)) / 2, 2)
      G = [Math]::Round((($means["$($p)0"].G + $means["$($p)1"].G) - ($means["$($q)0"].G + $means["$($q)1"].G)) / 2, 2)
      B = [Math]::Round((($means["$($p)0"].B + $means["$($p)1"].B) - ($means["$($q)0"].B + $means["$($q)1"].B)) / 2, 2)
    }
  }
  function Jitter($p) {
    return @{
      R = [Math]::Round($means["$($p)0"].R - $means["$($p)1"].R, 2)
      G = [Math]::Round($means["$($p)0"].G - $means["$($p)1"].G, 2)
      B = [Math]::Round($means["$($p)0"].B - $means["$($p)1"].B, 2)
    }
  }
  function MaxAbs($d) { return [Math]::Max([Math]::Abs($d.R), [Math]::Max([Math]::Abs($d.G), [Math]::Abs($d.B))) }

  $jN = Jitter "N"; $jH = Jitter "H"; $jFA = Jitter "FA"; $jFB = Jitter "FB"
  $noise = [Math]::Max([Math]::Max((MaxAbs $jN), (MaxAbs $jH)), [Math]::Max((MaxAbs $jFA), (MaxAbs $jFB)))
  $dHN  = Delta "H"  "N"
  $dFAN = Delta "FA" "N"
  $dFAH = Delta "FA" "H"
  $dFBN = Delta "FB" "N"

  Write-Host ""
  Write-Host ("within-side jitter N / H / FA / FB (max abs channel) : {0} / {1} / {2} / {3}" -f (MaxAbs $jN), (MaxAbs $jH), (MaxAbs $jFA), (MaxAbs $jFB))
  Write-Host ("H  - N  (hover vs selected, context)                 : R={0} G={1} B={2}" -f $dHN.R, $dHN.G, $dHN.B)
  Write-Host ("FA - N  (FOCUSED vs selected, DD-M4-P2-003)          : R={0} G={1} B={2}" -f $dFAN.R, $dFAN.G, $dFAN.B)
  Write-Host ("FA - H  (FOCUSED vs hovered,  DD-M4-P2-003)          : R={0} G={1} B={2}" -f $dFAH.R, $dFAH.G, $dFAH.B)
  Write-Host ("FB - N  (AGREEMENT LEG: focus moved on, must be ~0)  : R={0} G={1} B={2}" -f $dFBN.R, $dFBN.G, $dFBN.B)
  Write-Host ""

  # Guard, not a leg: if Tab never reached the window, FA would equal N and
  # every comparison below would be vacuous.
  if ((MaxAbs $dFAN) -le [Math]::Max($noise, 1.0)) {
    throw "FA is indistinguishable from N (max abs channel $(MaxAbs $dFAN) vs noise $noise): either the Tab key never reached the window or the first stop is not the sampled ToggleButton. The comparisons below would be vacuous."
  }

  $ok = $true
  if ((MaxAbs $dFAN) -lt 8) { Write-Host "FAIL: focused is not distinct from selected"; $ok = $false }
  if ((MaxAbs $dFAH) -lt 8) { Write-Host "FAIL: focused is not distinct from hovered"; $ok = $false }
  if ((MaxAbs $dFBN) -gt [Math]::Max($noise, 1.0)) { Write-Host "FAIL: 'All' did not return to its unfocused colour after focus moved on"; $ok = $false }
  if ($ok) { Write-Host "PASS: focused is distinct from both selected and hovered, and clears when focus moves on" }

  if ($KeepFrames) {
    foreach ($k in $frames.Keys) {
      $out = Join-Path $OutDir "$OutputPrefix-$k.png"
      $frames[$k].Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
      Write-Host "saved $out"
    }
  }
  foreach ($k in $frames.Keys) { $frames[$k].Dispose() }
} finally {
  Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
}
