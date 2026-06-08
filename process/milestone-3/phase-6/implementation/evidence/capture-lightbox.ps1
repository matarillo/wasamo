# Assistant lightbox capture (DPI-aware). Originally a T7 single-shot; extended
# in T8 with resize positive-control and collaborative owner-smoke modes (see
# the param comments for -ResizeTo / -ResizeCloseAt / -KeepAlive / -RaiseOnly /
# -ClickAt).
#
# Captures the real Gallery HWND by enumerating top-level windows for the
# launched process and matching the static title. Process.MainWindowHandle can
# pick a small helper HWND for this host, which is not the render target.
param(
  [int]$ProcessId = 0,
  [string]$OutDir = $PSScriptRoot,
  [string]$OutputPrefix = "t7-lightbox",
  # When set (e.g. "1280x960"), captures a second open-lightbox frame after
  # resizing the still-open window. This is the resize positive control for the
  # scrim's Fill/Fill behaviour: a fixed-rect scrim fails to cover the grown
  # viewport, a Fill/Fill scrim still covers it. When set, the trailing
  # close-after-click frame is skipped (the close button moves under resize).
  [string]$ResizeTo = "",
  # With -ResizeTo: after capturing open@resized, click this window-relative
  # "x,y" (the lightbox close button at the resized size) and capture
  # closed@resized. This makes the resize evidence a SAME-SIZE open-vs-closed
  # positive control: the scrim's dimming is judged against the closed frame at
  # the identical size, not against the smaller launch size (the scrim alpha is
  # subtle, so a same-size reference is required to read dim vs no-dim).
  [string]$ResizeCloseAt = "",
  # Collaborative owner-smoke modes (interactive resize positive control):
  #   -KeepAlive : launch + capture closed/open, then leave the process running
  #                and print "PID=<n>" so a later -RaiseOnly run can attach. Used
  #                for steps 1-2 (agent launches, opens lightbox).
  #   -RaiseOnly : attach to -ProcessId, raise the window to front WITHOUT
  #                resizing or moving it (SWP_NOSIZE|SWP_NOMOVE; SW_SHOW, never
  #                SW_RESTORE so an owner-maximized window stays maximized), then
  #                capture a single frame at the CURRENT size. Used for step 4
  #                (agent re-captures after the owner's interactive resize). The
  #                captured size is embedded in the filename.
  [switch]$KeepAlive,
  [switch]$RaiseOnly,
  # With -RaiseOnly: click this window-relative "x,y" point (e.g. the lightbox
  # close button) AFTER raising but BEFORE capturing, without resizing/moving.
  # Used to obtain a closed-state reference at the owner's resized size so the
  # scrim dim/no-dim judgement is a same-size open-vs-closed positive control.
  [string]$ClickAt = ""
)

if (-not ('WinLightboxCap' -as [type])) {
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class WinLightboxCap {
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
}
'@
}

[WinLightboxCap]::SetProcessDpiAwarenessContext([IntPtr](-4)) | Out-Null
Add-Type -AssemblyName System.Drawing

$repo = (Resolve-Path "$PSScriptRoot\..\..\..\..\..").Path
$exe = Join-Path $repo "target\release\gallery-rust.exe"
if (-not (Test-Path $exe)) { throw "missing $exe" }
New-Item -ItemType Directory -Force $OutDir | Out-Null

function Find-GalleryWindow($ProcessId) {
  $script:found = [IntPtr]::Zero
  [WinLightboxCap]::EnumWindows({
    param($h, $l)
    $owner = 0
    [WinLightboxCap]::GetWindowThreadProcessId($h, [ref]$owner) | Out-Null
    if ($owner -ne $ProcessId -or -not [WinLightboxCap]::IsWindowVisible($h)) { return $true }
    $sb = [Text.StringBuilder]::new(256)
    [WinLightboxCap]::GetWindowTextW($h, $sb, 256) | Out-Null
    if ($sb.ToString() -eq "Gallery") {
      $script:found = $h
      return $false
    }
    return $true
  }, [IntPtr]::Zero) | Out-Null
  return $script:found
}

function Window-Rect($Handle) {
  $r = New-Object WinLightboxCap+RECT
  [WinLightboxCap]::GetWindowRect($Handle, [ref]$r) | Out-Null
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
  $g.Dispose()
  $bmp.Dispose()
  Write-Host "saved $out (${w}x${h})"
}

function Click-WindowPoint($Handle, $X, $Y) {
  $r = Window-Rect $Handle
  $sx = $r.Left + $X
  $sy = $r.Top + $Y
  [WinLightboxCap]::SetCursorPos($sx, $sy) | Out-Null
  Start-Sleep -Milliseconds 100
  [WinLightboxCap]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
  [WinLightboxCap]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
}

$launched = $false
if ($ProcessId -ne 0) {
  $p = Get-Process -Id $ProcessId -ErrorAction Stop
} else {
  $p = Start-Process -FilePath $exe -PassThru
  $launched = $true
}
try {
  # Attaching to a live process needs only a short settle; a fresh launch needs
  # time to create the render window.
  if ($ProcessId -ne 0) { Start-Sleep -Milliseconds 600 } else { Start-Sleep -Seconds 3 }
  $h = Find-GalleryWindow $p.Id
  if ($h -eq [IntPtr]::Zero) { throw "no visible Gallery HWND (alive=$(-not $p.HasExited))" }

  if ($RaiseOnly) {
    # Step 4: re-capture after the owner's interactive resize. Preserve the
    # owner's size/position absolutely — no SetWindowPos sizing, no SW_RESTORE.
    # SW_SHOW (5) keeps a maximized window maximized; HWND_TOP raise with
    # SWP_NOSIZE|SWP_NOMOVE|SWP_SHOWWINDOW only changes z-order.
    [WinLightboxCap]::ShowWindow($h, 5) | Out-Null
    [WinLightboxCap]::SetWindowPos($h, [IntPtr]0, 0, 0, 0, 0, 0x0043) | Out-Null
    [WinLightboxCap]::SetForegroundWindow($h) | Out-Null
    Start-Sleep -Milliseconds 700
    if ($ClickAt -ne "") {
      $cp = $ClickAt.Split(',')
      Click-WindowPoint $h ([int]$cp[0]) ([int]$cp[1])
      Start-Sleep -Milliseconds 900
    }
    $r = Window-Rect $h
    $w = $r.Right - $r.Left
    $hh = $r.Bottom - $r.Top
    Write-Host "RaiseOnly: Gallery HWND=$h rect=($($r.Left),$($r.Top)) ${w}x${hh}"
    Capture-Window $h "$OutputPrefix-${w}x${hh}.png"
  } else {
    [WinLightboxCap]::ShowWindow($h, 9) | Out-Null
    [WinLightboxCap]::SetWindowPos($h, [IntPtr](-1), 130, 130, 800, 600, 0x0040) | Out-Null
    [WinLightboxCap]::SetForegroundWindow($h) | Out-Null
    Start-Sleep -Milliseconds 900

    $r = Window-Rect $h
    Write-Host "Gallery HWND=$h title=Gallery rect=($($r.Left),$($r.Top)) $($r.Right - $r.Left)x$($r.Bottom - $r.Top)"
    Capture-Window $h "$OutputPrefix-closed.png"

    # The Open button is placed between the top thumbnail strip and the scroll
    # view, near the left edge of the default 800x600 window.
    Click-WindowPoint $h 400 382
    Start-Sleep -Milliseconds 900
    Capture-Window $h "$OutputPrefix-open.png"

    if ($ResizeTo -ne "") {
      $parts = $ResizeTo.Split('x')
      $rw = [int]$parts[0]
      $rh = [int]$parts[1]
      # Resize the still-open window and re-capture: the scrim must still cover the
      # full client area at the new size (Fill/Fill positive control).
      [WinLightboxCap]::SetWindowPos($h, [IntPtr](-1), 130, 130, $rw, $rh, 0x0040) | Out-Null
      Start-Sleep -Milliseconds 900
      Capture-Window $h "$OutputPrefix-open-resized.png"
      if ($ResizeCloseAt -ne "") {
        # Close the lightbox at the resized size for a same-size closed reference.
        $rc = $ResizeCloseAt.Split(',')
        Click-WindowPoint $h ([int]$rc[0]) ([int]$rc[1])
        Start-Sleep -Milliseconds 900
        Capture-Window $h "$OutputPrefix-closed-resized.png"
      }
    } elseif (-not $KeepAlive) {
      # The close button is the rightmost nav button in the centered lightbox.
      Click-WindowPoint $h 462 523
      Start-Sleep -Milliseconds 900
      Capture-Window $h "$OutputPrefix-closed-after-click.png"
    }
  }

  if ($KeepAlive) {
    Write-Host "PID=$($p.Id)"
  }
} finally {
  if ($launched -and -not $KeepAlive) {
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
  }
}
