# M4-Phase 2 T4 hover positive control (assistant-automated GUI evidence).
#
# What it discriminates: whether hover / pressed is computed against the ONE
# resolved hit-test target (T4) or by the pre-T4 whole-tree walk that sets
# state on every Button-family widget whose own rectangle contains the point.
#
# The gallery already has a reachable overlap: the `if is_lightbox_open`
# branch is a later child of the root `ZStack` than the main `Grid`, and its
# scrim `Box` is stretch/stretch, so with the lightbox open the scrim covers
# the toolbar. Under the pre-T4 walk the toolbar's checked `ToggleButton`
# ("All") still enters its Hovered colour when the cursor sits over the scrim
# above it; under T4 the scrim is the resolved target and no Button reacts.
#
# The four frame sets, all at one window size and position, all sampling the
# SAME pixel mask (derived once from the closed / cursor-away frame so the
# scrim's own dimming cannot move the sample):
#
#   B  closed, cursor parked away  -> "All" at its Normal colour
#   A  closed, cursor over "All"   -> "All" at its Hovered colour
#        A vs B is the difference leg: it shows hover is observable here at all.
#   D  open,   cursor parked away  -> "All" Normal, dimmed by the scrim
#   C  open,   cursor over "All"   -> the discriminating frame
#        pre-T4:  C ~ A dimmed  (the button hovers THROUGH the scrim)
#        post-T4: C ~ D         (agreement leg: the scrim owns the point)
#
# Capture mechanics follow docs/notes/verification-environments.md
# Observation 4: PMv2 declared AND read back, CopyFromScreen (never
# PrintWindow) over the CLIENT rectangle mapped with ClientToScreen, window
# raised foreground+topmost, and at least two frames per side.
param(
  [string]$OutDir = $PSScriptRoot,
  [string]$OutputPrefix = "t4-hover",
  [switch]$KeepFrames
)

if (-not ('WinT4Cap' -as [type])) {
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class WinT4Cap {
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
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
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
[WinT4Cap]::SetProcessDpiAwarenessContext($PMV2) | Out-Null
if (-not [WinT4Cap]::AreDpiAwarenessContextsEqual([WinT4Cap]::GetThreadDpiAwarenessContext(), $PMV2)) {
  throw "capture tool is not Per-Monitor-Aware V2; every rectangle below would be virtualized"
}
Add-Type -AssemblyName System.Drawing

$repo = (Resolve-Path "$PSScriptRoot\..\..\..\..\..").Path
$exe = Join-Path $repo "target\release\gallery-rust.exe"
if (-not (Test-Path $exe)) { throw "missing $exe (run cargo build --release --workspace first)" }
New-Item -ItemType Directory -Force $OutDir | Out-Null

function Find-GalleryWindow($ProcessId) {
  $script:found = [IntPtr]::Zero
  [WinT4Cap]::EnumWindows({
    param($h, $l)
    $owner = 0
    [WinT4Cap]::GetWindowThreadProcessId($h, [ref]$owner) | Out-Null
    if ($owner -ne $ProcessId -or -not [WinT4Cap]::IsWindowVisible($h)) { return $true }
    $sb = [Text.StringBuilder]::new(256)
    [WinT4Cap]::GetWindowTextW($h, $sb, 256) | Out-Null
    if ($sb.ToString() -eq "Gallery") { $script:found = $h; return $false }
    return $true
  }, [IntPtr]::Zero) | Out-Null
  return $script:found
}

# The client rectangle in screen coordinates: both corners mapped with
# ClientToScreen, never derived from GetWindowRect minus a guessed frame.
function Client-ScreenRect($Handle) {
  $c = New-Object WinT4Cap+RECT
  [WinT4Cap]::GetClientRect($Handle, [ref]$c) | Out-Null
  $tl = New-Object WinT4Cap+POINT; $tl.X = $c.Left;  $tl.Y = $c.Top
  $br = New-Object WinT4Cap+POINT; $br.X = $c.Right; $br.Y = $c.Bottom
  [WinT4Cap]::ClientToScreen($Handle, [ref]$tl) | Out-Null
  [WinT4Cap]::ClientToScreen($Handle, [ref]$br) | Out-Null
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

# Lock the bitmap once and return a byte[] of 32bpp BGRA rows, so per-pixel
# reads are not a GetPixel call each.
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
  [WinT4Cap]::SetCursorPos($r.X + $ClientX, $r.Y + $ClientY) | Out-Null
  Start-Sleep -Milliseconds 450   # let the hover-in / hover-out animation settle
}

function Click-At($Handle, $ClientX, $ClientY) {
  Move-To $Handle $ClientX $ClientY
  [WinT4Cap]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
  Start-Sleep -Milliseconds 60
  [WinT4Cap]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
  Start-Sleep -Milliseconds 900
}

$p = Start-Process -FilePath $exe -PassThru
try {
  Start-Sleep -Seconds 3
  $h = Find-GalleryWindow $p.Id
  if ($h -eq [IntPtr]::Zero) { throw "no visible Gallery HWND (alive=$(-not $p.HasExited))" }

  [WinT4Cap]::ShowWindow($h, 9) | Out-Null
  [WinT4Cap]::SetWindowPos($h, [IntPtr](-1), 120, 120, 1000, 750, 0x0040) | Out-Null
  [WinT4Cap]::SetForegroundWindow($h) | Out-Null
  Start-Sleep -Milliseconds 1200

  $dpi = [WinT4Cap]::GetDpiForWindow($h)
  $scale = $dpi / 96.0
  $cr = Client-ScreenRect $h
  Write-Host "Gallery HWND=$h dpi=$dpi scale=$scale client=$($cr.W)x$($cr.H) at ($($cr.X),$($cr.Y))"

  # A parking spot inside the dark scroll area: a `Box`, never a hit target
  # that paints anything, and far from every Button.
  $parkX = [int]($cr.W * 0.5)
  $parkY = [int]($cr.H * 0.75)

  # ---- B: closed, cursor away (two frames) ------------------------------
  Move-To $h $parkX $parkY
  $frames = @{}
  $frames["B0"] = Capture-Client $h
  Start-Sleep -Milliseconds 250
  $frames["B1"] = Capture-Client $h

  # The sample mask: the checked "All" ToggleButton's own blue, found in the
  # left half of the 56-DIP toolbar band of the closed / cursor-away frame.
  # Derived once, reused for every frame, so the scrim can never move it.
  $b0 = Bitmap-Bytes $frames["B0"]
  $bandH = [Math]::Min([int](56 * $scale), $b0.H)
  $halfW = [int]($b0.W / 2)
  $mask = New-Object System.Collections.Generic.List[int]
  $minX = [int]::MaxValue; $maxX = -1; $minY = [int]::MaxValue; $maxY = -1
  for ($y = 0; $y -lt $bandH; $y++) {
    $row = $y * $b0.Stride
    for ($x = 0; $x -lt $halfW; $x++) {
      $i = $row + $x * 4
      $bl = $b0.Bytes[$i]; $gr = $b0.Bytes[$i+1]; $rd = $b0.Bytes[$i+2]
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

  # ---- A: closed, cursor over "All" -------------------------------------
  Move-To $h $hoverX $hoverY
  $frames["A0"] = Capture-Client $h
  Start-Sleep -Milliseconds 250
  $frames["A1"] = Capture-Client $h

  # ---- open the lightbox -------------------------------------------------
  # The "Open lightbox" Button is the rightmost item of the top-right toolbar
  # HStack (8 DIP padding), vertically centred in the 56-DIP row.
  $openX = $cr.W - [int](30 * $scale)
  $openY = [int](28 * $scale)
  Click-At $h $openX $openY
  Move-To $h $parkX $parkY

  # ---- D: open, cursor away ---------------------------------------------
  $frames["D0"] = Capture-Client $h
  Start-Sleep -Milliseconds 250
  $frames["D1"] = Capture-Client $h

  # ---- C: open, cursor over the same "All" pixels -----------------------
  Move-To $h $hoverX $hoverY
  $frames["C0"] = Capture-Client $h
  Start-Sleep -Milliseconds 250
  $frames["C1"] = Capture-Client $h

  $means = @{}
  foreach ($k in @("B0","B1","A0","A1","D0","D1","C0","C1")) { $means[$k] = Mask-Mean $frames[$k] }

  # The lightbox must actually be open, or C and D are two closed frames and
  # the control proves nothing. The scrim is #101820cc, so the sampled blue
  # loses most of its intensity.
  $closedB = ($means["B0"].B + $means["B1"].B) / 2
  $openB   = ($means["D0"].B + $means["D1"].B) / 2
  if ($openB -ge $closedB * 0.75) {
    throw "the lightbox does not appear to be open: sampled blue went $closedB -> $openB; the click at ($openX,$openY) probably missed"
  }

  foreach ($k in @("B0","B1","A0","A1","D0","D1","C0","C1")) {
    $m = $means[$k]
    Write-Host ("{0}  R={1,7} G={2,7} B={3,7}" -f $k, $m.R, $m.G, $m.B)
  }
  function Delta($p, $q) {
    return @{
      R = [Math]::Round((($means["$($p)0"].R + $means["$($p)1"].R) - ($means["$($q)0"].R + $means["$($q)1"].R)) / 2, 2)
      G = [Math]::Round((($means["$($p)0"].G + $means["$($p)1"].G) - ($means["$($q)0"].G + $means["$($q)1"].G)) / 2, 2)
      B = [Math]::Round((($means["$($p)0"].B + $means["$($p)1"].B) - ($means["$($q)0"].B + $means["$($q)1"].B)) / 2, 2)
    }
  }
  $noiseClosed = Delta "B" "B"   # zero by construction; printed as the shape check
  $dAB = Delta "A" "B"
  $dCD = Delta "C" "D"
  $jitterB = @{ R = [Math]::Round($means["B0"].R - $means["B1"].R, 2); G = [Math]::Round($means["B0"].G - $means["B1"].G, 2); B = [Math]::Round($means["B0"].B - $means["B1"].B, 2) }
  $jitterD = @{ R = [Math]::Round($means["D0"].R - $means["D1"].R, 2); G = [Math]::Round($means["D0"].G - $means["D1"].G, 2); B = [Math]::Round($means["D0"].B - $means["D1"].B, 2) }
  Write-Host ""
  Write-Host ("within-side jitter B0-B1 : R={0} G={1} B={2}" -f $jitterB.R, $jitterB.G, $jitterB.B)
  Write-Host ("within-side jitter D0-D1 : R={0} G={1} B={2}" -f $jitterD.R, $jitterD.G, $jitterD.B)
  Write-Host ("A-B (closed: hover vs no-hover, difference leg) : R={0} G={1} B={2}" -f $dAB.R, $dAB.G, $dAB.B)
  Write-Host ("C-D (open:   hover vs no-hover, THE CONTROL)    : R={0} G={1} B={2}" -f $dCD.R, $dCD.G, $dCD.B)
  Write-Host ""
  Write-Host "pre-T4 expectation : C-D is a scrim-attenuated copy of A-B (the button hovers through the scrim)"
  Write-Host "post-T4 expectation: C-D collapses into the within-side jitter (only the scrim is the target)"

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
