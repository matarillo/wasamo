# Verify the gallery window re-LAYOUTS on resize (C0 fixed stays constant, stars
# absorb slack) rather than bitmap-stretching uniformly. Captures two widths.
#
# The interop class is named WinResize (distinct from capture-smoke.ps1's WinCap)
# and guarded so both scripts can run in one PowerShell session and either can be
# re-run without a duplicate Add-Type failure.
if (-not ('WinResize' -as [type])) {
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class WinResize {
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr ctx);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
}
'@
}
[WinResize]::SetProcessDpiAwarenessContext([IntPtr](-4)) | Out-Null
Add-Type -AssemblyName System.Drawing

$exe = Join-Path (Resolve-Path "$PSScriptRoot\..\..\..\..\..").Path "target\release\gallery-rust.exe"
$p = Start-Process -FilePath $exe -PassThru
Start-Sleep -Seconds 3
$p.Refresh(); $h = $p.MainWindowHandle
if ($h -eq [IntPtr]::Zero) { throw "no handle" }
[WinResize]::ShowWindow($h, 9) | Out-Null

function Grab($w, $ht, $tag) {
  # SWP_NOMOVE=0x2, keep position; resize to w x ht (physical, caller is DPI-aware)
  [WinResize]::SetWindowPos($h, [IntPtr](-1), 0, 0, $w, $ht, 0x2) | Out-Null  # HWND_TOPMOST + size
  [WinResize]::SetForegroundWindow($h) | Out-Null
  Start-Sleep -Milliseconds 1200
  $r = New-Object WinResize+RECT; [WinResize]::GetWindowRect($h, [ref]$r) | Out-Null
  $aw = $r.Right - $r.Left; $ah = $r.Bottom - $r.Top
  $bmp = New-Object System.Drawing.Bitmap $aw, $ah
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($r.Left, $r.Top, 0, 0, (New-Object System.Drawing.Size($aw, $ah)))
  $out = "$PSScriptRoot\t6-resize-$tag.png"
  $bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
  $g.Dispose(); $bmp.Dispose()
  Write-Host "[$tag] window ${aw}x${ah} -> $out"
}
Grab 820 760 "narrow"
Grab 1500 760 "wide"
Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
