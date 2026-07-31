# M4-Phase 1 T10 — crop a region out of a captured frame and magnify it with
# nearest-neighbour so glyph shape survives the enlargement.
#
# Crispness is a glyph-shape judgement — stems, counters, fringing — and no
# pixel count substitutes for looking at it (plan.md §T10 control A). A
# smoothing interpolation would manufacture exactly the softness the comparison
# is about, so the interpolation mode is pinned to NearestNeighbor and the
# pixel-offset mode to Half; every output pixel is then a whole source pixel
# repeated $Factor times, and nothing in this script can make either side look
# sharper or softer than it was captured.
param(
  [Parameter(Mandatory = $true)][string]$In,
  [Parameter(Mandatory = $true)][string]$Out,
  [Parameter(Mandatory = $true)][int]$X,
  [Parameter(Mandatory = $true)][int]$Y,
  [Parameter(Mandatory = $true)][int]$W,
  [Parameter(Mandatory = $true)][int]$H,
  [int]$Factor = 5
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$src = [System.Drawing.Bitmap]::FromFile((Resolve-Path $In).Path)
try {
  if ($X + $W -gt $src.Width -or $Y + $H -gt $src.Height) {
    throw "crop ($X,$Y,$W,$H) falls outside $($src.Width)x$($src.Height)"
  }
  $dst = New-Object System.Drawing.Bitmap ($W * $Factor), ($H * $Factor)
  $g = [System.Drawing.Graphics]::FromImage($dst)
  $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
  $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
  $g.DrawImage($src,
    (New-Object System.Drawing.Rectangle 0, 0, ($W * $Factor), ($H * $Factor)),
    (New-Object System.Drawing.Rectangle $X, $Y, $W, $H),
    [System.Drawing.GraphicsUnit]::Pixel)
  $g.Dispose()
  $dst.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
  $dst.Dispose()
  Write-Host ("{0} [{1},{2} {3}x{4}] x{5} -> {6}" -f (Split-Path -Leaf $In), $X, $Y, $W, $H, $Factor, $Out)
}
finally { $src.Dispose() }
