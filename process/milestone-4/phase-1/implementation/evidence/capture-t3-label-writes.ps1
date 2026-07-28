# M4-Phase 1 T3 GUI evidence capture — Button / ToggleButton label Visual
# writes relocated into the sync pass.
#
# Run once on the unmodified tree (-Tag before) and once on the changed tree
# (-Tag after). The pair is the positive control; the "after" frames alone
# prove nothing.
#
# Requires a visible Windows desktop. CopyFromScreen over the window rect —
# DirectComposition content reads back blank under PrintWindow
# (docs/notes/verification-environments.md Observation 4).
#
# Frames captured, and what each is for:
#   gallery-default     3 ToggleButton labels + 3 Button labels, placed at
#                       construction and laid out by set_root's first pass.
#   gallery-tab-albums  the same labels after a click re-runs layout through
#                       the reactive drain (flush_layout -> run_layout).
#   gallery-lightbox    the "<" / ">" / "x" Buttons of the conditional
#                       subtree: constructed *after* the tree was attached,
#                       so their labels are placed by the drain's layout pass
#                       rather than by set_root.
#   labelupdate-initial the label-update evidence .ui (a Button whose text is
#   labelupdate-clicked bound to state), before and after a click. This is the
#                       only coverage of the second relocated write site —
#                       no shipped example binds a Button's text.
#
# The evidence .ui starts its counter at 9 on purpose, so the first click
# crosses 9 -> 10 and the label gains a digit. At 0 -> 1 -> 2 the digits have
# the same advance width, the Button never changes size, and an implementation
# that wrote a stale width into `label_size` would pass every frame in the set
# (independent review finding R-4).
param(
  [Parameter(Mandatory = $true)][ValidateSet("before", "after")][string]$Tag
)

$ErrorActionPreference = "Stop"

if (-not ('WinT3Cap' -as [type])) {
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class WinT3Cap {
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr ctx);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int hgt, bool repaint);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, IntPtr extraInfo);
}
'@
}

[WinT3Cap]::SetProcessDpiAwarenessContext([IntPtr](-4)) | Out-Null
Add-Type -AssemblyName System.Drawing

$repo = (Resolve-Path "$PSScriptRoot\..\..\..\..\..").Path
$exe = Join-Path $repo "target\release\gallery-rust.exe"
if (-not (Test-Path $exe)) { throw "missing $exe — run cargo build --release --workspace first" }

# The host reads its .uic from an absolute path baked in at build time
# (examples/gallery-rust/build.rs writes it to OUT_DIR and exports
# WASAMO_GALLERY_IR). Swapping the file at that path lets the same executable
# load the label-update evidence UI without touching the repo.
$uic = Get-ChildItem -Path (Join-Path $repo "target\release\build") -Filter "gallery.uic" -Recurse |
       Select-Object -First 1
if ($null -eq $uic) { throw "gallery.uic not found under target/release/build" }
$uicPath = $uic.FullName
$backup = "$uicPath.t3-backup"

# Recover from an interrupted earlier run. The swap below is undone in a
# `finally`, but a hard kill between the swap and the restore leaves the
# evidence IR sitting at the gallery's path — and cargo will not regenerate
# it, because `build.rs` only re-runs when `gallery.ui` changes. A later
# gallery capture would then photograph the evidence UI and report success.
# Restoring here makes that state self-healing on the next run rather than
# silent until someone looks closely at a frame.
if (Test-Path $backup) {
  Write-Warning "leftover backup from an interrupted run — restoring $uicPath before capturing"
  Copy-Item $backup $uicPath -Force
  Remove-Item $backup -Force
}

$outDir = Join-Path $PSScriptRoot $Tag
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

$script:proc = $null
$script:hwnd = [IntPtr]::Zero

function LaunchHost {
  $script:proc = Start-Process -FilePath $exe -PassThru
  Start-Sleep -Seconds 3
  $script:proc.Refresh()
  $script:hwnd = $script:proc.MainWindowHandle
  if ($script:hwnd -eq [IntPtr]::Zero) {
    throw "no MainWindowHandle (alive=$(-not $script:proc.HasExited))"
  }
  [WinT3Cap]::ShowWindow($script:hwnd, 9) | Out-Null
}

function StopHost {
  if ($null -ne $script:proc) {
    Stop-Process -Id $script:proc.Id -Force -ErrorAction SilentlyContinue
    $script:proc = $null
  }
}

function PositionWindow($width, $height) {
  [WinT3Cap]::MoveWindow($script:hwnd, 0, 0, $width, $height, $true) | Out-Null
  [WinT3Cap]::SetWindowPos($script:hwnd, [IntPtr](-1), 0, 0, $width, $height, 0x0040) | Out-Null
  [WinT3Cap]::SetForegroundWindow($script:hwnd) | Out-Null
  Start-Sleep -Milliseconds 1000
}

function CaptureFrame($name) {
  $r = New-Object WinT3Cap+RECT
  [WinT3Cap]::GetWindowRect($script:hwnd, [ref]$r) | Out-Null
  $w = $r.Right - $r.Left
  $h = $r.Bottom - $r.Top
  if ($w -le 0 -or $h -le 0) { throw "invalid window size ${w}x${h}" }
  $out = Join-Path $outDir "$name.png"
  Write-Host ("[{0}] dpi={1} rect=({2},{3}) {4}x{5}" -f $name, [WinT3Cap]::GetDpiForWindow($script:hwnd), $r.Left, $r.Top, $w, $h)

  $bmp = New-Object System.Drawing.Bitmap $w, $h
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($r.Left, $r.Top, 0, 0, (New-Object System.Drawing.Size($w, $h)))
  $bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
  $g.Dispose()
  $bmp.Dispose()
  Write-Host "  -> $out"
}

function ClickAt($x, $y) {
  [WinT3Cap]::SetCursorPos($x, $y) | Out-Null
  [WinT3Cap]::mouse_event(0x0002, 0, 0, 0, [IntPtr]::Zero)
  [WinT3Cap]::mouse_event(0x0004, 0, 0, 0, [IntPtr]::Zero)
  Start-Sleep -Milliseconds 800
}

# Click points, read off a probe capture of the 1200x760 gallery window
# placed at the desktop origin (so window coordinates are screen coordinates).
$AlbumsTab    = @(138, 74)   # the second ToggleButton
$OpenLightbox = @(1104, 74)  # the accent Button that inserts the conditional subtree
$CounterBtn   = @(320, 128)  # the label-update UI's bound-text Button

# `*.uic` is gitignored, so the compiled evidence IR is regenerated from the
# committed `.ui` with the workspace's own compiler.
$labelUpdateUi = Join-Path $PSScriptRoot "t3-label-update.ui"
$labelUpdateUic = Join-Path $PSScriptRoot "t3-label-update.uic"
& (Join-Path $repo "target\release\wasamoc.exe") build $labelUpdateUi $labelUpdateUic
if ($LASTEXITCODE -ne 0) { throw "wasamoc build failed for $labelUpdateUi" }

try {
  # ── Gallery frames (construction-time write site) ──────────────────────
  LaunchHost
  PositionWindow 1200 760
  CaptureFrame "gallery-default"
  ClickAt $AlbumsTab[0] $AlbumsTab[1]
  CaptureFrame "gallery-tab-albums"
  ClickAt $OpenLightbox[0] $OpenLightbox[1]
  CaptureFrame "gallery-lightbox"
  StopHost

  # ── Label-update frames (label-update write site) ──────────────────────
  Copy-Item $uicPath $backup -Force
  Copy-Item $labelUpdateUic $uicPath -Force
  LaunchHost
  PositionWindow 640 420
  CaptureFrame "labelupdate-initial"
  ClickAt $CounterBtn[0] $CounterBtn[1]
  CaptureFrame "labelupdate-clicked"
  ClickAt $CounterBtn[0] $CounterBtn[1]
  CaptureFrame "labelupdate-clicked-twice"
  StopHost
}
finally {
  StopHost
  if (Test-Path $backup) {
    Copy-Item $backup $uicPath -Force
    Remove-Item $backup -Force
    Write-Host "restored $uicPath"
  }
}
