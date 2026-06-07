# T7 assistant lightbox capture (DPI-aware).
#
# Captures the real Gallery HWND by enumerating top-level windows for the
# launched process and matching the static title. Process.MainWindowHandle can
# pick a small helper HWND for this host, which is not the render target.
param(
  [int]$ProcessId = 0,
  [string]$OutDir = $PSScriptRoot
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
  Start-Sleep -Seconds 3
  $h = Find-GalleryWindow $p.Id
  if ($h -eq [IntPtr]::Zero) { throw "no visible Gallery HWND (alive=$(-not $p.HasExited))" }
  [WinLightboxCap]::ShowWindow($h, 9) | Out-Null
  [WinLightboxCap]::SetWindowPos($h, [IntPtr](-1), 130, 130, 800, 600, 0x0040) | Out-Null
  [WinLightboxCap]::SetForegroundWindow($h) | Out-Null
  Start-Sleep -Milliseconds 900

  $r = Window-Rect $h
  Write-Host "Gallery HWND=$h title=Gallery rect=($($r.Left),$($r.Top)) $($r.Right - $r.Left)x$($r.Bottom - $r.Top)"
  Capture-Window $h "t7-lightbox-closed.png"

  # The Open button is placed between the top thumbnail strip and the scroll
  # view, near the left edge of the default 800x600 window.
  Click-WindowPoint $h 400 382
  Start-Sleep -Milliseconds 900
  Capture-Window $h "t7-lightbox-open.png"

  # The close button is the rightmost nav button in the centered lightbox.
  Click-WindowPoint $h 462 523
  Start-Sleep -Milliseconds 900
  Capture-Window $h "t7-lightbox-closed-after-click.png"
} finally {
  if ($launched) {
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
  }
}
