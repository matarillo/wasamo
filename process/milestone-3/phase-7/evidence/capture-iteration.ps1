# T8 assistant-visible iteration capture (DPI-aware). Adapted from the Phase 6
# capture-lightbox.ps1 pattern: enumerate the real "Gallery" HWND, drive the
# body-external Add / Remove Buttons with SetCursorPos + mouse_event at
# window-relative coordinates, and CopyFromScreen over GetWindowRect (PrintWindow
# reads back blank under DirectComposition).
#
# Positive control: initial N -> Add (N+1, prefix undisturbed) -> Remove (N).
param(
  [string]$OutDir = $PSScriptRoot,
  [string]$OutputPrefix = "t8-iteration",
  [int]$Width = 760,
  [int]$Height = 1180,
  # "x,y;x,y;..." window-relative click points applied between frames. Each click
  # is followed by a capture. Empty = capture launch frame only (recon).
  [string]$ClicksThenCapture = "",
  [string]$Labels = "init,add1,add2,remove"
)

if (-not ('WinIterCap' -as [type])) {
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class WinIterCap {
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

[WinIterCap]::SetProcessDpiAwarenessContext([IntPtr](-4)) | Out-Null
Add-Type -AssemblyName System.Drawing

$repo = (Resolve-Path "$PSScriptRoot\..\..\..\..").Path
$exe = Join-Path $repo "target\release\gallery-rust.exe"
if (-not (Test-Path $exe)) { throw "missing $exe" }
New-Item -ItemType Directory -Force $OutDir | Out-Null

function Find-GalleryWindow($ProcessId) {
  $script:found = [IntPtr]::Zero
  [WinIterCap]::EnumWindows({
    param($h, $l)
    $owner = 0
    [WinIterCap]::GetWindowThreadProcessId($h, [ref]$owner) | Out-Null
    if ($owner -ne $ProcessId -or -not [WinIterCap]::IsWindowVisible($h)) { return $true }
    $sb = [Text.StringBuilder]::new(256)
    [WinIterCap]::GetWindowTextW($h, $sb, 256) | Out-Null
    if ($sb.ToString() -eq "Gallery") { $script:found = $h; return $false }
    return $true
  }, [IntPtr]::Zero) | Out-Null
  return $script:found
}

function Window-Rect($Handle) {
  $r = New-Object WinIterCap+RECT
  [WinIterCap]::GetWindowRect($Handle, [ref]$r) | Out-Null
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

function Click-WindowPoint($Handle, $X, $Y) {
  $r = Window-Rect $Handle
  [WinIterCap]::SetCursorPos(($r.Left + $X), ($r.Top + $Y)) | Out-Null
  Start-Sleep -Milliseconds 120
  [WinIterCap]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
  [WinIterCap]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
}

$p = Start-Process -FilePath $exe -PassThru
try {
  Start-Sleep -Seconds 3
  $h = Find-GalleryWindow $p.Id
  if ($h -eq [IntPtr]::Zero) { throw "no visible Gallery HWND (alive=$(-not $p.HasExited))" }

  [WinIterCap]::ShowWindow($h, 9) | Out-Null
  [WinIterCap]::SetWindowPos($h, [IntPtr](-1), 80, 40, $Width, $Height, 0x0040) | Out-Null
  [WinIterCap]::SetForegroundWindow($h) | Out-Null
  Start-Sleep -Milliseconds 900

  $r = Window-Rect $h
  Write-Host "Gallery HWND=$h rect=($($r.Left),$($r.Top)) $($r.Right - $r.Left)x$($r.Bottom - $r.Top)"

  $labelArr = $Labels.Split(',')
  Capture-Window $h "$OutputPrefix-$($labelArr[0]).png"

  if ($ClicksThenCapture -ne "") {
    $i = 1
    foreach ($pt in $ClicksThenCapture.Split(';')) {
      $xy = $pt.Split(',')
      Click-WindowPoint $h ([int]$xy[0]) ([int]$xy[1])
      Start-Sleep -Milliseconds 900
      $name = if ($i -lt $labelArr.Count) { $labelArr[$i] } else { "f$i" }
      Capture-Window $h "$OutputPrefix-$name.png"
      $i++
    }
  }
} finally {
  Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
}
