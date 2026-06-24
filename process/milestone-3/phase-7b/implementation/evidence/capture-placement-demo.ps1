# M3-Phase 7b T5 assistant GUI-evidence capture (DPI-aware).
#
# Re-tuned for the CURRENT gallery layout: the proven Phase 6/7
# capture-lightbox.ps1 click coordinates are stale (the inherited "Open
# lightbox" coordinate no longer hits the button after the placement-demo
# button was inserted). Captures the live Gallery HWND via CopyFromScreen over
# GetWindowRect (PrintWindow reads back blank under DirectComposition).
#
# Layout assumption: the button coordinates passed below are derived against the
# WxH size set here. A later gallery layout change re-staleness them — re-derive
# from a fresh -CaptureHomeOnly frame.
#
# Synthetic input (SetCursorPos+mouse_event and posted WM_LBUTTON*) drives the
# wasamo Composition app's buttons ONLY on a real / elevated desktop session;
# inside a restricted (sandboxed) session the injected input is dropped and the
# button never fires. Run this capture on a visible, non-sandboxed desktop.
param(
  [string]$OutDir = $PSScriptRoot,
  [string]$OutputPrefix = "t5",
  # Override the gallery-rust.exe path. Defaults to this repo's release build;
  # set it to a worktree build to capture a baseline from another commit (e.g.
  # the T4-pre bare-syntax lightbox for the same-position proof).
  [string]$ExePath = "",
  [int]$Width = 820,
  [int]$Height = 720,
  # "x,y" window-relative point of the "Open placement demo" button. When set,
  # the script clicks it after the home capture and captures the demo overlay
  # as "<prefix>-placement-demo.png".
  [string]$OpenDemoAt = "",
  # "x,y" window-relative point of the demo "Close demo" button. When set (with
  # -OpenDemoAt), the script clicks it after the demo capture and captures the
  # closed-again frame (the toggle positive control: the overlay disappears).
  [string]$CloseDemoAt = "",
  # "x,y" window-relative point of the "Open lightbox" button. When set, the
  # script clicks it after the home capture and captures the lightbox overlay as
  # "<prefix>-lightbox-slot-current.png" (the same-position re-render proof:
  # compare scrim/photo placement against the Phase 6/7 lightbox evidence).
  [string]$OpenLightboxAt = "",
  [switch]$CaptureHomeOnly
)

if (-not ('WinDemoCap' -as [type])) {
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class WinDemoCap {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr ctx);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc cb, IntPtr lp);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr h, StringBuilder sb, int max);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
    [DllImport("user32.dll")] public static extern bool ScreenToClient(IntPtr h, ref POINT p);
    [DllImport("user32.dll")] public static extern IntPtr SendMessageW(IntPtr h, uint msg, IntPtr wParam, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr h, uint msg, IntPtr wParam, IntPtr lParam);
    [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
}
'@
}

[WinDemoCap]::SetProcessDpiAwarenessContext([IntPtr](-4)) | Out-Null
Add-Type -AssemblyName System.Drawing

if ($ExePath -ne "") {
  $exe = $ExePath
} else {
  $repo = (Resolve-Path "$PSScriptRoot\..\..\..\..\..").Path
  $exe = Join-Path $repo "target\release\gallery-rust.exe"
}
if (-not (Test-Path $exe)) { throw "missing $exe" }
New-Item -ItemType Directory -Force $OutDir | Out-Null

function Find-GalleryWindow($ProcessId) {
  $script:found = [IntPtr]::Zero
  [WinDemoCap]::EnumWindows({
    param($h, $l)
    $owner = 0
    [WinDemoCap]::GetWindowThreadProcessId($h, [ref]$owner) | Out-Null
    if ($owner -ne $ProcessId -or -not [WinDemoCap]::IsWindowVisible($h)) { return $true }
    $sb = [Text.StringBuilder]::new(256)
    [WinDemoCap]::GetWindowTextW($h, $sb, 256) | Out-Null
    if ($sb.ToString() -eq "Gallery") { $script:found = $h; return $false }
    return $true
  }, [IntPtr]::Zero) | Out-Null
  return $script:found
}

function Window-Rect($Handle) {
  $r = New-Object WinDemoCap+RECT
  [WinDemoCap]::GetWindowRect($Handle, [ref]$r) | Out-Null
  return $r
}

function Capture-Window($Handle, $Name) {
  $r = Window-Rect $Handle
  $w = $r.Right - $r.Left
  $h = $r.Bottom - $r.Top
  if ($w -le 0 -or $h -le 0) { throw "invalid capture rect ${w}x${h}" }
  $bmp = New-Object System.Drawing.Bitmap $w, $h
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($r.Left, $r.Top, 0, 0, (New-Object System.Drawing.Size($w, $h)))
  $out = Join-Path $OutDir $Name
  $bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
  $g.Dispose(); $bmp.Dispose()
  Write-Host "saved $out (${w}x${h})"
}

# Post a WM_MOUSEMOVE + WM_LBUTTONDOWN/UP straight to the HWND in client
# coordinates. Foreground/cursor-independent: drives the single-HWND Composition
# app's internal hit-testing without relying on the window holding focus.
function Click-Message($Handle, $X, $Y) {
  $r = Window-Rect $Handle
  $pt = New-Object WinDemoCap+POINT
  $pt.X = $r.Left + $X
  $pt.Y = $r.Top + $Y
  [WinDemoCap]::ScreenToClient($Handle, [ref]$pt) | Out-Null
  $lp = [IntPtr]((($pt.Y -band 0xFFFF) -shl 16) -bor ($pt.X -band 0xFFFF))
  [WinDemoCap]::SetForegroundWindow($Handle) | Out-Null
  [WinDemoCap]::SendMessageW($Handle, 0x0200, [IntPtr]0, $lp) | Out-Null      # WM_MOUSEMOVE
  Start-Sleep -Milliseconds 60
  [WinDemoCap]::SendMessageW($Handle, 0x0201, [IntPtr]1, $lp) | Out-Null      # WM_LBUTTONDOWN (MK_LBUTTON)
  Start-Sleep -Milliseconds 90
  [WinDemoCap]::SendMessageW($Handle, 0x0202, [IntPtr]0, $lp) | Out-Null      # WM_LBUTTONUP
  Start-Sleep -Milliseconds 120
}

function Click-WindowPoint($Handle, $X, $Y) {
  $r = Window-Rect $Handle
  [WinDemoCap]::SetForegroundWindow($Handle) | Out-Null
  # A real cursor move (neutral point first) so the app sees WM_MOUSEMOVE before
  # the click, then a down/up with a press gap. Synthetic-input apps that track
  # hover state can drop a zero-gap click.
  [WinDemoCap]::SetCursorPos(($r.Left + $X), ($r.Top + $Y - 40)) | Out-Null
  Start-Sleep -Milliseconds 120
  [WinDemoCap]::SetCursorPos(($r.Left + $X), ($r.Top + $Y)) | Out-Null
  Start-Sleep -Milliseconds 180
  [WinDemoCap]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
  Start-Sleep -Milliseconds 90
  [WinDemoCap]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
  Start-Sleep -Milliseconds 120
}

$p = Start-Process -FilePath $exe -PassThru
try {
  Start-Sleep -Seconds 3
  $h = Find-GalleryWindow $p.Id
  if ($h -eq [IntPtr]::Zero) { throw "no visible Gallery HWND (alive=$(-not $p.HasExited))" }

  [WinDemoCap]::ShowWindow($h, 9) | Out-Null
  [WinDemoCap]::SetWindowPos($h, [IntPtr](-1), 130, 130, $Width, $Height, 0x0040) | Out-Null
  [WinDemoCap]::SetForegroundWindow($h) | Out-Null
  Start-Sleep -Milliseconds 900

  $r = Window-Rect $h
  Write-Host "Gallery HWND=$h rect=($($r.Left),$($r.Top)) $($r.Right - $r.Left)x$($r.Bottom - $r.Top)"
  Capture-Window $h "$OutputPrefix-home.png"

  if (-not $CaptureHomeOnly -and $OpenDemoAt -ne "") {
    $od = $OpenDemoAt.Split(',')
    Click-Message $h ([int]$od[0]) ([int]$od[1])
    Start-Sleep -Milliseconds 900
    Capture-Window $h "$OutputPrefix-placement-demo.png"

    if ($CloseDemoAt -ne "") {
      $cd = $CloseDemoAt.Split(',')
      Click-Message $h ([int]$cd[0]) ([int]$cd[1])
      Start-Sleep -Milliseconds 900
      Capture-Window $h "$OutputPrefix-demo-closed.png"
    }
  }

  if (-not $CaptureHomeOnly -and $OpenLightboxAt -ne "") {
    $ol = $OpenLightboxAt.Split(',')
    Click-Message $h ([int]$ol[0]) ([int]$ol[1])
    Start-Sleep -Milliseconds 900
    Capture-Window $h "$OutputPrefix-lightbox-slot-current.png"
  }
} finally {
  Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
}
