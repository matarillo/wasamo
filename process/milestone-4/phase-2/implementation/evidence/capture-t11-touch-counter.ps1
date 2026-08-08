# M4-Phase 2 T11 shipped-runtime touch evidence (assistant-automated GUI
# evidence; the important artifact for framing agreement (6)).
#
# What this measures, and why it is not the same claim as
# probe-t11-touch-injection.ps1. That probe shows Windows' own promotion
# behaviour on a plain, wasamo-free window. It does NOT show that the
# wasamo runtime receives a real OS touch contact and routes it to an
# authored handler. This script drives the shipped counter-rust.exe host
# (examples/counter/counter.ui: a Text bound to `Count: \{root.count}` and
# a Button whose `clicked` handler does `root.count += 1`) with a real OS
# input contact and reads the result back as rendered pixels -- the same
# assistant-automated-GUI-evidence shape capture-t10-item-identity.ps1
# uses (PMv2 declared and read back, CopyFromScreen over the CLIENT
# rectangle, window raised foreground+topmost, >=2 frames per step,
# display scale recorded -- docs/notes/verification-environments.md
# Observation 4).
#
# The control. The SAME script drives the SAME app the SAME number of
# times (a fresh launch each run, both starting at Count: 0, moved to the
# SAME fixed window rectangle), differing ONLY in which input family
# activates the Button:
#   -Input mouse : SetCursorPos + mouse_event (T10's shape)
#   -Input touch : InitializeTouchInjection + InjectTouchInput
#                  (probe-t11-touch-injection.ps1's shape)
# at the SAME screen point in both cases. Two legs follow from that:
#
#   THE DIFFERENCE LEG. Step 0 (before any input) and step 1 (after one
#   activation) must differ -- the count changed at all. Checked live,
#   during capture, not only at compare time: if a step's frame does not
#   differ from the previous one, the script throws immediately rather
#   than saving a silently-identical frame set (see "Finding the Button"
#   below for why this matters).
#
#   THE AGREEMENT LEG -- the actual claim this script exists for. `touch`
#   step N must agree with `mouse` step N for N = 0,1,2,3, within the
#   text-pixel jitter tolerance Phase 1 F-33 measured: antialiased text
#   pixels drift session to session, bounded at up to 13 per channel,
#   never bit-identical (see AGENTS.md `Testing rules` / handoff.md's F-33
#   row). If a touch contact were delivered TWICE -- exactly the state
#   claiming the pointer messages exists to prevent (WM_POINTER*'s
#   suppression of mouse promotion, DD-M4-P2-001 SS Touch) -- touch step 1
#   would read "Count: 2" against the mouse run's "Count: 1", and every
#   pixel in the digit would differ by far more than 13 per channel: the
#   comparison would FAIL, loudly and specifically. That is why the
#   agreement leg is the evidence here, not a formality alongside the
#   difference leg.
#
# Finding the Button. counter.ui gives the Button no explicit size or
# position -- it is sized by its own text and Fluent "accent" style, and
# positioned by the VStack's `padding: 24px` / `spacing: 12px`, so its
# rectangle cannot be read off the `.ui` source the way T10's Grid-track
# arithmetic could. It was instead measured directly: with the window at
# this script's fixed outer rectangle (WINDOW_X/Y/W/H below, physical
# px), one frame of a freshly launched counter-rust.exe was captured and
# scanned pixel-by-pixel for the Fluent accent-blue fill (R<100, G in
# 80..190, B>150); the resulting bounding box was x[282..399] y[92..134]
# physical px on a 682x453 physical client at 120 DPI (scale 1.25),
# giving a pixel centroid of (340.5, 113.0) physical, i.e. (272.4, 90.4)
# DIP once divided by the measured scale. Those two DIP numbers are
# hard-coded below as BUTTON_CENTRE_DIP_X / BUTTON_CENTRE_DIP_Y; every run
# re-multiplies them by whatever scale it measures at capture time (T10's
# Thumb-Point pattern), so a same-DPI, same-window-rectangle re-run stays
# correct without re-deriving anything.
#
# This derivation is machine- and rectangle-specific (the Button's DIP
# position may depend on the window's DIP width, since the observed X
# sits almost exactly at half the client width -- consistent with the
# VStack's content being centred against the full available width). A
# wrong point is therefore a real risk on a different DPI or a different
# fixed rectangle, which is exactly why every activation below is followed
# by a live check that the count actually changed, and the script THROWS
# (does not silently save a flat frame set) if it did not.
#
# Both runs precede capture with `cargo build --release --workspace`
# (Phase 1 finding F-21: a host-package-only build can relink wasamo.dll
# around a stale uplifted wasamo-runtime rlib, silently and with a fresh
# timestamp).
#
# Usage:
#   .\capture-t11-touch-counter.ps1 -Capture -Input mouse
#   .\capture-t11-touch-counter.ps1 -Capture -Input touch
#   .\capture-t11-touch-counter.ps1 -Compare
param(
  [switch]$Capture,
  [ValidateSet('touch', 'mouse')]
  [string]$Input,
  [switch]$Compare,
  [string]$OutDir = (Join-Path $PSScriptRoot "t11-frames"),
  [string]$OutputPrefix = "t11-touch-counter"
)

$ErrorActionPreference = "Stop"

# `$Input` is a PowerShell reserved automatic variable (the pipeline
# enumerator): `-Input mouse` binds correctly into $PSBoundParameters, but
# the `$Input` variable itself cannot be assigned and reads back empty.
# Read the bound value out of $PSBoundParameters instead, so the external
# parameter name stays `-Input` while the script body uses a real variable.
$InputFamily = $PSBoundParameters['Input']

if (-not $Capture -and -not $Compare) {
  throw "pass -Capture -Input touch|mouse, and/or -Compare"
}
if ($Capture -and -not $InputFamily) {
  throw "-Capture requires -Input touch|mouse"
}

# The fixed OUTER window rectangle (physical screen px) every run targets.
# Both legs use this exact rectangle -- otherwise the two frame sets are
# not comparable (Phase 1 T10 finding: frame-set shape is part of
# identity) -- and it is also the rectangle the Button's DIP centre above
# was derived against.
$WINDOW_X = 150
$WINDOW_Y = 150
$WINDOW_W = 700
$WINDOW_H = 500

# See "Finding the Button" above.
$BUTTON_CENTRE_DIP_X = 272.4
$BUTTON_CENTRE_DIP_Y = 90.4

# An in-client point in the dark background, well clear of the Button and
# of the "Count: N" text, used only to earn foreground activation
# (SetForegroundWindow is refused unless the caller is already foreground
# -- Observation 4 / T10's Try-Activate). It must be INSIDE the window
# (a foreground-earning click has to land on the target), unlike
# CURSOR_PARK_SCREEN_X/Y below.
$ACTIVATE_FRACTION_X = 0.5
$ACTIVATE_FRACTION_Y = 0.85

# A fixed OFF-WINDOW screen position (outside WINDOW_X..WINDOW_X+WINDOW_W /
# WINDOW_Y..WINDOW_Y+WINDOW_H) the physical cursor is moved to before
# every capture, in both input families. This is deliberately off the
# window rather than merely off the Button: a cursor still inside the
# client area keeps `tracking_mouse` engaged (WM_MOUSELEAVE only fires
# once the cursor truly leaves the client rectangle), and the two input
# families must differ ONLY in which one activates the Button -- not
# additionally in where the physical cursor happens to sit afterward,
# which the mouse leg controls directly (SetCursorPos as part of
# clicking) and the touch leg does not touch at all. Parking both legs at
# the same fixed off-window point removes that asymmetry and removes any
# dependence on incidental cursor state left over from a previous run.
$CURSOR_PARK_SCREEN_X = 5
$CURSOR_PARK_SCREEN_Y = 5

if (-not ('WinT11Cap' -as [type])) {
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class WinT11Cap {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }

    [StructLayout(LayoutKind.Sequential)]
    public struct POINTER_INFO {
        public uint pointerType;
        public uint pointerId;
        public uint frameId;
        public uint pointerFlags;
        public IntPtr sourceDevice;
        public IntPtr hwndTarget;
        public POINT ptPixelLocation;
        public POINT ptHimetricLocation;
        public POINT ptPixelLocationRaw;
        public POINT ptHimetricLocationRaw;
        public uint dwTime;
        public uint historyCount;
        public int inputData;
        public uint dwKeyStates;
        public ulong PerformanceCount;
        public int ButtonChangeType;
    }
    [StructLayout(LayoutKind.Sequential)]
    public struct POINTER_TOUCH_INFO {
        public POINTER_INFO pointerInfo;
        public uint touchFlags;
        public uint touchMask;
        public RECT rcContact;
        public RECT rcContactRaw;
        public uint orientation;
        public uint pressure;
    }

    [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr ctx);
    [DllImport("user32.dll")] public static extern IntPtr GetThreadDpiAwarenessContext();
    [DllImport("user32.dll")] public static extern bool AreDpiAwarenessContextsEqual(IntPtr a, IntPtr b);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc cb, IntPtr lp);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr h, StringBuilder sb, int max);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(POINT p);

    [DllImport("user32.dll", SetLastError=true)] public static extern bool InitializeTouchInjection(uint maxCount, uint dwMode);
    [DllImport("user32.dll", SetLastError=true)] static extern bool InjectTouchInput(uint count, [In] POINTER_TOUCH_INFO[] contacts);

    public const uint PT_TOUCH = 2;
    public const uint TOUCH_DOWN = 0x00010000 | 0x2 | 0x4; // DOWN | INRANGE | INCONTACT
    public const uint TOUCH_UP   = 0x00040000;             // UP

    // Built entirely in C#: PowerShell assignment to a *nested* value-type
    // field mutates a copy, so a struct built from PowerShell reaches the
    // OS still zeroed (probe-t11-touch-injection.ps1's header, trap 2).
    static POINTER_TOUCH_INFO MakeContact(int x, int y, uint flags) {
        POINTER_TOUCH_INFO c = new POINTER_TOUCH_INFO();
        c.pointerInfo.pointerType = PT_TOUCH;
        c.pointerInfo.pointerId = 0;
        c.pointerInfo.pointerFlags = flags;
        c.pointerInfo.ptPixelLocation.X = x;
        c.pointerInfo.ptPixelLocation.Y = y;
        c.touchFlags = 0;
        c.touchMask = 0x00000007; // CONTACTAREA | ORIENTATION | PRESSURE
        c.rcContact.Left = x - 2; c.rcContact.Top = y - 2;
        c.rcContact.Right = x + 2; c.rcContact.Bottom = y + 2;
        c.orientation = 90;
        c.pressure = 512; // touch range is 0..1024 (trap 1)
        return c;
    }

    public static string InjectTouch(int x, int y, uint flags) {
        POINTER_TOUCH_INFO[] a = new POINTER_TOUCH_INFO[1];
        a[0] = MakeContact(x, y, flags);
        bool ok = InjectTouchInput(1, a);
        // GetLastWin32Error is only meaningful on failure -- printed
        // beside a success it is frequently a stale code left over from
        // an unrelated earlier call, and reads like a defect to a
        // reviewer even though nothing failed.
        if (ok) { return "ok=True"; }
        int err = Marshal.GetLastWin32Error();
        return "ok=False err=0x" + err.ToString("X8");
    }
}
'@
}

# Per-Monitor-Aware V2, declared AND verified (Observation 4 / Phase 1
# F-48): declaring is not evidence that the declaration took effect.
$PMV2 = [IntPtr](-4)
[WinT11Cap]::SetProcessDpiAwarenessContext($PMV2) | Out-Null
if (-not [WinT11Cap]::AreDpiAwarenessContextsEqual([WinT11Cap]::GetThreadDpiAwarenessContext(), $PMV2)) {
  throw "capture tool is not Per-Monitor-Aware V2; every rectangle below would be virtualized"
}
Add-Type -AssemblyName System.Drawing
New-Item -ItemType Directory -Force $OutDir | Out-Null

function Find-CounterWindow($ProcessId) {
  $script:found = [IntPtr]::Zero
  [WinT11Cap]::EnumWindows({
    param($h, $l)
    $owner = 0
    [WinT11Cap]::GetWindowThreadProcessId($h, [ref]$owner) | Out-Null
    if ($owner -ne $ProcessId -or -not [WinT11Cap]::IsWindowVisible($h)) { return $true }
    $sb = [Text.StringBuilder]::new(256)
    [WinT11Cap]::GetWindowTextW($h, $sb, 256) | Out-Null
    if ($sb.ToString() -eq "Counter") { $script:found = $h; return $false }
    return $true
  }, [IntPtr]::Zero) | Out-Null
  return $script:found
}

# Every top-level window the process owns, for a failure message that says
# what was actually there. A bare "not found" cannot distinguish "the host
# died", "the host is alive but has not shown its window yet" and "the
# title is not what this script looks for".
function Describe-ProcessWindows($ProcessId) {
  $script:rows = New-Object System.Collections.ArrayList
  [WinT11Cap]::EnumWindows({
    param($h, $l)
    $owner = 0
    [WinT11Cap]::GetWindowThreadProcessId($h, [ref]$owner) | Out-Null
    if ($owner -ne $ProcessId) { return $true }
    $sb = [Text.StringBuilder]::new(256)
    [WinT11Cap]::GetWindowTextW($h, $sb, 256) | Out-Null
    $null = $script:rows.Add("hwnd=$h visible=$([WinT11Cap]::IsWindowVisible($h)) title='$($sb.ToString())'")
    return $true
  }, [IntPtr]::Zero) | Out-Null
  if ($script:rows.Count -eq 0) { return "(no top-level windows)" }
  return ($script:rows -join "; ")
}

# Poll for the host's window rather than sleeping a fixed interval and
# looking once.
#
# **Measured, not defensive.** A single look after 3 s failed on this
# machine under load: the process was alive, an untitled visible window
# already existed at 3 s, and the titled "Counter" window only appeared
# between 3 s and 5 s. The same script had succeeded on the same machine
# earlier the same day, so a fixed wait encodes the machine's mood at the
# moment it was written. This is the discipline
# docs/notes/verification-environments.md Observation 4 already states for
# foreground activation -- "a single refusal is not an environment
# verdict ... retry before concluding anything" -- applied to the step
# before it, which had been left on a fixed sleep.
function Wait-ForCounterWindow($ProcessId, $TimeoutSeconds = 30) {
  $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
  while ((Get-Date) -lt $deadline) {
    $h = Find-CounterWindow $ProcessId
    if ($h -ne [IntPtr]::Zero) { return $h }
    Start-Sleep -Milliseconds 250
  }
  return [IntPtr]::Zero
}

# The client rectangle in screen coordinates: both corners mapped with
# ClientToScreen, never derived from GetWindowRect minus a guessed frame
# (Observation 4).
function Client-ScreenRect($Handle) {
  $c = New-Object WinT11Cap+RECT
  [WinT11Cap]::GetClientRect($Handle, [ref]$c) | Out-Null
  $tl = New-Object WinT11Cap+POINT; $tl.X = $c.Left;  $tl.Y = $c.Top
  $br = New-Object WinT11Cap+POINT; $br.X = $c.Right; $br.Y = $c.Bottom
  [WinT11Cap]::ClientToScreen($Handle, [ref]$tl) | Out-Null
  [WinT11Cap]::ClientToScreen($Handle, [ref]$br) | Out-Null
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

# Parks the physical cursor at the same FIXED off-window screen position
# in both input families, before every capture (including step 0). This
# is what keeps the two runs differing ONLY in which input family
# activates the Button: without it, the mouse leg's own SetCursorPos (as
# part of clicking) would leave the cursor sitting on or near the Button
# for that leg's captures, while the touch leg -- which never calls
# SetCursorPos -- would leave the physical cursor wherever it happened to
# be beforehand (including, incidentally, wherever the mouse leg's own
# run last parked it, since cursor position is desktop-global and
# persists across separate process launches). Neither is a controlled,
# reproducible state for a pixel comparison.
function Park-CursorOffWindow() {
  [WinT11Cap]::SetCursorPos($CURSOR_PARK_SCREEN_X, $CURSOR_PARK_SCREEN_Y) | Out-Null
  Start-Sleep -Milliseconds 200
}

# Injection (both mouse and touch) is desktop-scoped, not window-scoped:
# it goes to whatever window is at the screen point
# (probe-t11-touch-injection.ps1's header). FAILS LOUDLY rather than
# continuing if another window is on top.
function Assert-WindowUnderPoint($Handle, $ScreenX, $ScreenY, [string]$What) {
  $pt = New-Object WinT11Cap+POINT; $pt.X = $ScreenX; $pt.Y = $ScreenY
  $under = [WinT11Cap]::WindowFromPoint($pt)
  if ($under -ne $Handle) {
    throw "$What -- WindowFromPoint($ScreenX,$ScreenY) returned $under, not the Counter window ($Handle): another window is on top of the activation point"
  }
}

function Click-At($Handle, $ClientX, $ClientY) {
  $r = Client-ScreenRect $Handle
  $sx = $r.X + $ClientX; $sy = $r.Y + $ClientY
  Assert-WindowUnderPoint $Handle $sx $sy "mouse activation"
  [WinT11Cap]::SetCursorPos($sx, $sy) | Out-Null
  Start-Sleep -Milliseconds 200
  [WinT11Cap]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)   # LEFTDOWN
  Start-Sleep -Milliseconds 60
  [WinT11Cap]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)   # LEFTUP
  Start-Sleep -Milliseconds 900   # let the click's drain + re-layout settle
}

function Touch-At($Handle, $ClientX, $ClientY) {
  $r = Client-ScreenRect $Handle
  $sx = $r.X + $ClientX; $sy = $r.Y + $ClientY
  Assert-WindowUnderPoint $Handle $sx $sy "touch activation"
  $down = [WinT11Cap]::InjectTouch($sx, $sy, [WinT11Cap]::TOUCH_DOWN)
  if ($down -ne "ok=True") { throw "InjectTouchInput (down) failed: $down" }
  Start-Sleep -Milliseconds 150
  $up = [WinT11Cap]::InjectTouch($sx, $sy, [WinT11Cap]::TOUCH_UP)
  if ($up -ne "ok=True") { throw "InjectTouchInput (up) failed: $up" }
  Start-Sleep -Milliseconds 900   # let the tap's drain + re-layout settle
}

# Foreground activation is earned with a real click, never requested and
# assumed (Observation 4 / T10's Try-Activate). Lands on the in-client
# activate point (ACTIVATE_FRACTION_X/Y), which touches nothing this
# script measures; the cursor is parked off-window separately
# (Park-CursorOffWindow) before any frame is captured.
function Try-Activate($Handle, $ClientX, $ClientY, $Attempts = 5) {
  for ($i = 1; $i -le $Attempts; $i++) {
    Click-At $Handle $ClientX $ClientY
    [WinT11Cap]::SetForegroundWindow($Handle) | Out-Null
    Start-Sleep -Milliseconds 300
    if ([WinT11Cap]::GetForegroundWindow() -eq $Handle) {
      if ($i -gt 1) { Write-Host "foreground acquired on attempt $i" }
      return $true
    }
    Write-Host ("attempt {0}/{1}: foreground is {2}, retrying" -f $i, $Attempts, [WinT11Cap]::GetForegroundWindow())
    Start-Sleep -Milliseconds 700
  }
  return $false
}

# Count pixels whose summed channel difference exceeds a threshold well
# above the text-pixel jitter F-33 measured (up to 13 per channel, so up
# to 39 summed); 60 is comfortably above it and far below a digit glyph
# appearing or disappearing -- the same threshold
# capture-t10-item-identity.ps1 uses.
function Diff-Count($a, $b) {
  if ($a.W -ne $b.W -or $a.H -ne $b.H) { throw "frame size mismatch: $($a.W)x$($a.H) vs $($b.W)x$($b.H)" }
  $n = 0
  for ($y = 0; $y -lt $a.H; $y++) {
    $row = $y * $a.Stride
    for ($x = 0; $x -lt $a.W; $x++) {
      $i = $row + $x * 4
      $d = [Math]::Abs([int]$a.Bytes[$i] - [int]$b.Bytes[$i]) +
           [Math]::Abs([int]$a.Bytes[$i+1] - [int]$b.Bytes[$i+1]) +
           [Math]::Abs([int]$a.Bytes[$i+2] - [int]$b.Bytes[$i+2])
      if ($d -gt 60) { $n++ }
    }
  }
  return $n
}

# Pixels that differ AT ALL, at any magnitude. Reported beside
# `Diff-Count` because the two answer different questions and only this
# one is the noise floor F-33 is about: `Diff-Count`'s 60-per-channel-sum
# threshold exists to count *visible* change (a digit appearing), and
# reporting its zero as "differing px" reads as "the frames are
# identical" when they can differ by 1 on thousands of pixels. Measured
# on this artifact's own frames: a mouse set's two back-to-back captures
# of the same state differ by max 1 per channel over 4,638 px, which
# `Diff-Count` reports as 0.
function Diff-CountAny($a, $b) {
  if ($a.W -ne $b.W -or $a.H -ne $b.H) { throw "frame size mismatch: $($a.W)x$($a.H) vs $($b.W)x$($b.H)" }
  $n = 0
  for ($y = 0; $y -lt $a.H; $y++) {
    $row = $y * $a.Stride
    for ($x = 0; $x -lt $a.W; $x++) {
      $i = $row + $x * 4
      if ($a.Bytes[$i] -ne $b.Bytes[$i] -or
          $a.Bytes[$i+1] -ne $b.Bytes[$i+1] -or
          $a.Bytes[$i+2] -ne $b.Bytes[$i+2]) { $n++ }
    }
  }
  return $n
}

# The measured MAXIMUM per-channel delta across the whole frame -- the
# number the agreement leg's tolerance (F-33's "up to 13 per channel") is
# checked against. Never a bit-identity check.
function Max-ChannelDiff($a, $b) {
  if ($a.W -ne $b.W -or $a.H -ne $b.H) { throw "frame size mismatch: $($a.W)x$($a.H) vs $($b.W)x$($b.H)" }
  $max = 0
  for ($y = 0; $y -lt $a.H; $y++) {
    $row = $y * $a.Stride
    for ($x = 0; $x -lt $a.W; $x++) {
      $i = $row + $x * 4
      for ($c = 0; $c -lt 3; $c++) {
        $d = [Math]::Abs([int]$a.Bytes[$i + $c] - [int]$b.Bytes[$i + $c])
        if ($d -gt $max) { $max = $d }
      }
    }
  }
  return $max
}

$F33_TOLERANCE = 13  # measured maximum, Phase 1 F-33; see this script's header

# ── Capture mode ───────────────────────────────────────────────────────────

function Do-Capture([string]$InputFamily) {
  # Recorded before anything else runs, so it is this run's true start --
  # not the moment the first frame lands -- and so two separate
  # `-Capture -Input mouse` / `-Capture -Input touch` invocations leave two
  # visibly distinct timestamps in their respective run-meta files
  # (independent-review finding D8: the artifact must carry its own
  # evidence that two separate runs happened, not just assert it).
  $runStart = Get-Date -Format "o"

  $repo = (Resolve-Path "$PSScriptRoot\..\..\..\..\..").Path
  $exe = Join-Path $repo "target\release\counter-rust.exe"

  Write-Host "cargo build --release --workspace (F-21: must precede any capture of a runtime change)"
  Push-Location $repo
  try {
    & cargo build --release --workspace
    if ($LASTEXITCODE -ne 0) { throw "cargo build --release --workspace failed with exit code $LASTEXITCODE" }
  } finally {
    Pop-Location
  }
  if (-not (Test-Path $exe)) { throw "missing $exe after build" }

  $commit = (& git -C $repo rev-parse HEAD).Trim()
  Write-Host "commit=$commit"

  if ($InputFamily -eq "touch") {
    # InjectTouchInput requires InitializeTouchInjection to have been
    # called on this thread first, or it fails with ok=False and no
    # further diagnostic (the same trap probe-t11-touch-injection.ps1's
    # header records).
    if (-not [WinT11Cap]::InitializeTouchInjection(1, 3)) {
      throw "InitializeTouchInjection failed -- touch injection is unavailable in this session"
    }
  }

  $p = Start-Process -FilePath $exe -PassThru
  try {
    $h = Wait-ForCounterWindow $p.Id 30
    if ($h -eq [IntPtr]::Zero) {
      throw ("no visible Counter HWND after 30s (alive=$(-not $p.HasExited)); " +
             "windows owned by pid $($p.Id): $(Describe-ProcessWindows $p.Id)")
    }

    [WinT11Cap]::ShowWindow($h, 9) | Out-Null   # SW_RESTORE
    # HWND_TOPMOST = -1: raised foreground + topmost (Observation 4).
    [WinT11Cap]::SetWindowPos($h, [IntPtr](-1), $WINDOW_X, $WINDOW_Y, $WINDOW_W, $WINDOW_H, 0x0040) | Out-Null
    Start-Sleep -Milliseconds 1200

    $outer = New-Object WinT11Cap+RECT
    [WinT11Cap]::GetWindowRect($h, [ref]$outer) | Out-Null
    if ($outer.Left -ne $WINDOW_X -or $outer.Top -ne $WINDOW_Y -or
        ($outer.Right - $outer.Left) -ne $WINDOW_W -or ($outer.Bottom - $outer.Top) -ne $WINDOW_H) {
      throw "window did not land at the fixed target rectangle: got ($($outer.Left),$($outer.Top)) $($outer.Right - $outer.Left)x$($outer.Bottom - $outer.Top), wanted ($WINDOW_X,$WINDOW_Y) ${WINDOW_W}x${WINDOW_H}"
    }

    $dpi = [WinT11Cap]::GetDpiForWindow($h)
    $scale = $dpi / 96.0
    $cr = Client-ScreenRect $h
    Write-Host "Counter HWND=$h dpi=$dpi scale=$scale client=$($cr.W)x$($cr.H) px at ($($cr.X),$($cr.Y))"

    $btnClientX = [int]($BUTTON_CENTRE_DIP_X * $scale)
    $btnClientY = [int]($BUTTON_CENTRE_DIP_Y * $scale)
    Write-Host "derived Button centre: ($BUTTON_CENTRE_DIP_X,$BUTTON_CENTRE_DIP_Y) DIP -> ($btnClientX,$btnClientY) client px at scale $scale"

    $activateX = [int]($cr.W * $ACTIVATE_FRACTION_X)
    $activateY = [int]($cr.H * $ACTIVATE_FRACTION_Y)

    $activated = Try-Activate $h $activateX $activateY
    if (-not $activated) { throw "could not acquire foreground for the Counter window" }
    Write-Host "foreground acquired and read back"

    function Save-Set([int]$Step) {
      $frames = @()
      for ($i = 0; $i -lt 2; $i++) {
        $bmp = Capture-Client $h
        $out = Join-Path $OutDir "$OutputPrefix-$InputFamily-step$Step-$i.png"
        $bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
        Write-Host "saved $out"
        $frames += , (Bitmap-Bytes $bmp)
        $bmp.Dispose()
        Start-Sleep -Milliseconds 250
      }
      return $frames
    }

    # Step 0: before any input. The cursor is parked off-window here too
    # (not just after each activation), so step 0 starts from the same
    # controlled cursor state as every later step, in both input families.
    Park-CursorOffWindow
    $step0 = Save-Set 0
    $prev = $step0[0]

    for ($step = 1; $step -le 3; $step++) {
      Write-Host "activation $step ($InputFamily) at client ($btnClientX,$btnClientY) px"
      if ($InputFamily -eq "mouse") {
        Click-At $h $btnClientX $btnClientY
      } else {
        Touch-At $h $btnClientX $btnClientY
      }
      Park-CursorOffWindow   # same fixed off-window position in both input families
      $frames = Save-Set $step

      # FAIL LOUDLY if the count did not visibly change: a wrong Button
      # point must be a red run, never a silently identical frame set
      # (this script's header, "Finding the Button").
      $changed = Diff-Count $prev $frames[0]
      Write-Host "  step $($step-1) -> step $step ($InputFamily): $changed differing px"
      if ($changed -le 40) {
        throw "activation $step ($InputFamily) produced only $changed differing px between step $($step-1) and step $step -- the count did not visibly change. The derived Button point may be wrong, or the activation did not land on the Button. Re-derive BUTTON_CENTRE_DIP_X/Y (see this script's header) before trusting any comparison."
      }
      $prev = $frames[0]
    }

    $meta = @(
      "commit=$commit"
      "input=$InputFamily"
      "run_start=$runStart"
      "dpi=$dpi"
      "scale=$scale"
      "client_x=$($cr.X)"
      "client_y=$($cr.Y)"
      "client_w=$($cr.W)"
      "client_h=$($cr.H)"
      "button_centre_dip_x=$BUTTON_CENTRE_DIP_X"
      "button_centre_dip_y=$BUTTON_CENTRE_DIP_Y"
      "button_centre_client_x=$btnClientX"
      "button_centre_client_y=$btnClientY"
      "cursor_park_screen_x=$CURSOR_PARK_SCREEN_X"
      "cursor_park_screen_y=$CURSOR_PARK_SCREEN_Y"
    ) -join "`n"
    Set-Content -Path (Join-Path $OutDir "$OutputPrefix-$InputFamily-run-meta.txt") -Value $meta

    Write-Host "$InputFamily capture complete: all three activations changed the rendered frame."
  } finally {
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
  }
}

# ── Compare mode ───────────────────────────────────────────────────────────

function Do-Compare() {
  function Load($InputFamily, $Step, $Index) {
    $path = Join-Path $OutDir "$OutputPrefix-$InputFamily-step$Step-$Index.png"
    if (-not (Test-Path $path)) { throw "missing $path -- run -Capture -Input $InputFamily first" }
    return New-Object System.Drawing.Bitmap $path
  }
  function ReadMeta($InputFamily) {
    $path = Join-Path $OutDir "$OutputPrefix-$InputFamily-run-meta.txt"
    if (-not (Test-Path $path)) { throw "missing $path -- run -Capture -Input $InputFamily first" }
    $h = @{}
    foreach ($line in Get-Content $path) {
      $kv = $line -split '=', 2
      $h[$kv[0]] = $kv[1]
    }
    return $h
  }

  $mouseMeta = ReadMeta "mouse"
  $touchMeta = ReadMeta "touch"

  if ($mouseMeta.commit -ne $touchMeta.commit) {
    throw "the mouse and touch captures were built at different commits ($($mouseMeta.commit) vs $($touchMeta.commit)) -- re-capture both against the same commit"
  }
  if ($mouseMeta.client_w -ne $touchMeta.client_w -or $mouseMeta.client_h -ne $touchMeta.client_h -or
      $mouseMeta.client_x -ne $touchMeta.client_x -or $mouseMeta.client_y -ne $touchMeta.client_y) {
    throw "the mouse and touch runs landed at different client rectangles (mouse=$($mouseMeta.client_x),$($mouseMeta.client_y) $($mouseMeta.client_w)x$($mouseMeta.client_h); touch=$($touchMeta.client_x),$($touchMeta.client_y) $($touchMeta.client_w)x$($touchMeta.client_h)) -- the two frame sets are not comparable"
  }
  if ($mouseMeta.cursor_park_screen_x -ne $touchMeta.cursor_park_screen_x -or
      $mouseMeta.cursor_park_screen_y -ne $touchMeta.cursor_park_screen_y) {
    throw "the mouse and touch runs parked the cursor at different screen positions (mouse=$($mouseMeta.cursor_park_screen_x),$($mouseMeta.cursor_park_screen_y); touch=$($touchMeta.cursor_park_screen_x),$($touchMeta.cursor_park_screen_y)) -- the two frame sets are not comparable"
  }

  Write-Host "commit=$($mouseMeta.commit)"
  Write-Host "display: dpi=$($mouseMeta.dpi) scale=$($mouseMeta.scale)"
  Write-Host "client rect: ($($mouseMeta.client_x),$($mouseMeta.client_y)) $($mouseMeta.client_w)x$($mouseMeta.client_h)"
  Write-Host "derived Button centre: ($($mouseMeta.button_centre_dip_x),$($mouseMeta.button_centre_dip_y)) DIP"
  Write-Host "cursor parked at screen ($($mouseMeta.cursor_park_screen_x),$($mouseMeta.cursor_park_screen_y)) before every capture, both input families"
  Write-Host "tolerance (F-33 measured maximum): $F33_TOLERANCE per channel"
  Write-Host ""

  $bmps = @()
  $data = @{}
  $dataFrame1 = @{}
  foreach ($fam in @("mouse", "touch")) {
    for ($step = 0; $step -le 3; $step++) {
      $b0 = Load $fam $step 0
      $b1 = Load $fam $step 1
      $bmps += $b0
      $bmps += $b1
      $data["$fam$step"] = Bitmap-Bytes $b0
      $dataFrame1["$fam$step"] = Bitmap-Bytes $b1
    }
  }

  $ok = $true
  $metaLines = New-Object System.Collections.Generic.List[string]

  # Within-set jitter (Observation 4: require at least two agreeing
  # captures per side before comparing across the change; T10 precedent).
  # Each set's two frames are the SAME rendered state, captured back to
  # back, so any difference here is pure capture noise, not signal.
  Write-Host "within-set jitter (same set, two frames of the same rendered state -- the noise floor):"
  $noise = 0
  $noiseAny = 0
  $noiseMax = 0
  foreach ($fam in @("mouse", "touch")) {
    for ($step = 0; $step -le 3; $step++) {
      $n = Diff-Count $data["$fam$step"] $dataFrame1["$fam$step"]
      $any = Diff-CountAny $data["$fam$step"] $dataFrame1["$fam$step"]
      $mx = Max-ChannelDiff $data["$fam$step"] $dataFrame1["$fam$step"]
      if ($n -gt $noise) { $noise = $n }
      if ($any -gt $noiseAny) { $noiseAny = $any }
      if ($mx -gt $noiseMax) { $noiseMax = $mx }
      Write-Host ("  {0,-5} step {1}: max_channel={2,3}, {3,7} px differ at all, {4,7} px over the 60/channel-sum visible-change threshold" -f $fam, $step, $mx, $any, $n)
    }
  }
  Write-Host "  noise floor: max_channel=$noiseMax, $noiseAny px differ at all, $noise px over the visible-change threshold"
  Write-Host ""
  $metaLines.Add("commit=$($mouseMeta.commit)")
  $metaLines.Add("dpi=$($mouseMeta.dpi)")
  $metaLines.Add("scale=$($mouseMeta.scale)")
  $metaLines.Add("client_rect=$($mouseMeta.client_x),$($mouseMeta.client_y),$($mouseMeta.client_w),$($mouseMeta.client_h)")
  $metaLines.Add("button_centre_dip=$($mouseMeta.button_centre_dip_x),$($mouseMeta.button_centre_dip_y)")
  $metaLines.Add("button_centre_derivation=one frame scanned for the Fluent accent-blue fill (R<100, 80<G<190, B>150); pixel bbox x[282..399] y[92..134] physical at 682x453 client, 120 DPI (scale 1.25); centroid (340.5,113.0) physical / (272.4,90.4) DIP")
  $metaLines.Add("cursor_park_screen=$($mouseMeta.cursor_park_screen_x),$($mouseMeta.cursor_park_screen_y) (fixed, off-window, both input families, before every capture)")
  $metaLines.Add("input_path_mouse=SetCursorPos + mouse_event (LEFTDOWN/LEFTUP)")
  $metaLines.Add("input_path_touch=InitializeTouchInjection + InjectTouchInput (DOWN then UP flags)")
  $metaLines.Add("tolerance_per_channel=$F33_TOLERANCE (Phase 1 F-33 measured maximum; never bit-identity)")
  $metaLines.Add("noise_floor_within_set_max_channel=$noiseMax")
  $metaLines.Add("noise_floor_within_set_px_differing_at_all=$noiseAny")
  $metaLines.Add("noise_floor_within_set_px_over_visible_change_threshold=$noise")
  $metaLines.Add("")
  $metaLines.Add("--- difference leg (step 0 vs step 1, same input family; the count changed at all) ---")

  foreach ($fam in @("mouse", "touch")) {
    $n = Diff-Count $data["${fam}0"] $data["${fam}1"]
    $verdict = if ($n -gt 40) { "PASS" } else { "FAIL" }
    if ($verdict -eq "FAIL") { $ok = $false }
    Write-Host ("DIFFERENCE ({0,-5}) step0 vs step1: {1,7} differing px -> {2}" -f $fam, $n, $verdict)
    $metaLines.Add("difference_${fam}_step0_vs_step1=$n px, verdict=$verdict")
  }

  Write-Host ""
  Write-Host "--- agreement leg (touch vs mouse, same step N; the actual claim) ---"
  $metaLines.Add("")
  $metaLines.Add("--- agreement leg (touch step N vs mouse step N) ---")
  for ($step = 0; $step -le 3; $step++) {
    $maxDiff = Max-ChannelDiff $data["mouse$step"] $data["touch$step"]
    $diffCount = Diff-Count $data["mouse$step"] $data["touch$step"]
    $diffAny = Diff-CountAny $data["mouse$step"] $data["touch$step"]
    $verdict = if ($maxDiff -le $F33_TOLERANCE) { "PASS" } else { "FAIL" }
    if ($verdict -eq "FAIL") { $ok = $false }
    Write-Host ("AGREEMENT  step {0}: max_channel_diff={1,3} (tolerance {2}), {3,7} px differ at all, {4,7} px over the visible-change threshold -> {5}" -f $step, $maxDiff, $F33_TOLERANCE, $diffAny, $diffCount, $verdict)
    $metaLines.Add("agreement_step${step}_max_channel_diff=$maxDiff, px_differing_at_all=$diffAny, px_over_visible_change_threshold=$diffCount, verdict=$verdict")
  }

  Write-Host ""
  if ($ok) {
    Write-Host "PASS: touch and mouse agree at every step within the F-33 tolerance, and both input families visibly change the count -- a touch contact reaches the shipped counter-rust.exe host exactly once per activation."
  } else {
    Write-Host "FAIL: see the verdict lines above. If step 1 touch disagreed with step 1 mouse specifically (not the later steps), suspect a double-delivered touch contact -- exactly what claiming the pointer messages exists to prevent."
  }
  $metaLines.Add("")
  $metaLines.Add("overall=$(if ($ok) { 'PASS' } else { 'FAIL' })")

  # ── Provenance (independent-review finding D8) ──────────────────────────
  # The touch and mouse frames are expected to be byte-identical (that
  # identity IS the agreement leg's result -- see t11-frames/README.md), so
  # nothing about the PNG bytes themselves can show two separate runs
  # happened. What can: the two runs' own start times (recorded by
  # Do-Capture, read back here from each run-meta file, so a mouse run and
  # a touch run started at visibly different wall-clock moments), and a
  # SHA-256 + capture (last-write) timestamp for every retained file --
  # computed here, at compare time, straight off disk, not carried through
  # from Do-Capture, so this is provenance for exactly what is committed,
  # not for whatever a capture run claimed it wrote.
  Write-Host ""
  Write-Host "--- provenance: two independent runs, and this artifact's own file hashes ---"
  $metaLines.Add("")
  $metaLines.Add("--- provenance (two independent runs; every retained file's SHA-256 and capture timestamp) ---")
  $metaLines.Add("run_start_mouse=$($mouseMeta.run_start)")
  $metaLines.Add("run_start_touch=$($touchMeta.run_start)")
  Write-Host "run_start_mouse=$($mouseMeta.run_start)"
  Write-Host "run_start_touch=$($touchMeta.run_start)"
  $provenanceFiles = New-Object System.Collections.Generic.List[string]
  foreach ($fam in @("mouse", "touch")) {
    for ($step = 0; $step -le 3; $step++) {
      for ($i = 0; $i -le 1; $i++) {
        $provenanceFiles.Add("$OutputPrefix-$fam-step$step-$i.png")
      }
    }
    $provenanceFiles.Add("$OutputPrefix-$fam-run-meta.txt")
  }
  foreach ($fileName in $provenanceFiles) {
    $path = Join-Path $OutDir $fileName
    $hash = (Get-FileHash -Path $path -Algorithm SHA256).Hash
    $captured = (Get-Item $path).LastWriteTimeUtc.ToString("o")
    $line = "file=$fileName sha256=$hash captured_utc=$captured"
    $metaLines.Add($line)
    Write-Host "  $line"
  }

  Set-Content -Path (Join-Path $OutDir "meta.txt") -Value $metaLines
  Write-Host ""
  Write-Host "wrote $(Join-Path $OutDir 'meta.txt')"

  foreach ($b in $bmps) { $b.Dispose() }
  if (-not $ok) { exit 1 }
}

if ($Capture) { Do-Capture $InputFamily }
if ($Compare) { Do-Compare }
# Both Do-Capture and Do-Compare terminate the script (throw, or an
# explicit `exit 1`) on any failure; reaching here means every check
# passed. Exit 0 explicitly rather than leaving $LASTEXITCODE at whatever
# a prior external command set it to -- the same false-green class
# Phase 1 findings F-5/F-21 and compare-frames.ps1's R-4 record.
exit 0
