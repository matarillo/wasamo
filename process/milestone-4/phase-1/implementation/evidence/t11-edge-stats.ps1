# M4-Phase 1 T11 — glyph-edge statistics over a crop of a captured frame.
#
# What this measures, and what it does not.
#
# Crispness is a glyph-shape judgement and the judgement is made by looking at
# the magnified pair (plan.md §T10 control A, and the crop-*-x6.png artifacts
# in t11-owner-smoke/). This script does not replace that. It reports three
# numbers that describe how an edge is *shaped*, so the visual reading has
# something falsifiable next to it:
#
#   mean|dx|   mean absolute horizontal step in luminance between neighbouring
#              pixels, over the crop
#   max|dx|    the largest such step
#   mid-band   the share of pixels whose luminance sits between 25% and 75% of
#              the crop's own range — the "neither background nor glyph"
#              population
#
# A natively rasterized glyph has its transition inside roughly one pixel: high
# max|dx|, few mid-band pixels. The same glyph rasterized at a lower resolution
# and then resampled up by the compositor has its transition spread over two or
# more pixels: lower max|dx|, more mid-band pixels. That is the difference the
# T11 control is about.
#
# The metric is a proxy and is treated as one. Two things make it usable here
# rather than merely suggestive, and both are properties of the run, not of the
# script:
#
#   1. It was exercised on a leg where it *must* read equal — the 100% pair,
#      where every conversion is the identity — and it read equal to the last
#      digit on both windows. A metric that separated them there would have
#      been measuring window identity, position or capture, not rasterization.
#   2. The aware side matches an independently known-aware reference (leg 0,
#      the window launched normally before the control was set up) at the same
#      scale.
#
# Neither number says *why* an edge is soft. Use it to corroborate a reading of
# the magnified crops, never to stand in for one.
param(
  [Parameter(Mandatory=$true)][string]$In,
  [Parameter(Mandatory=$true)][int]$X,
  [Parameter(Mandatory=$true)][int]$Y,
  [Parameter(Mandatory=$true)][int]$W,
  [Parameter(Mandatory=$true)][int]$H,
  [string]$Label = ""
)
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
$bmp = [System.Drawing.Bitmap]::FromFile((Resolve-Path $In).Path)
try {
  if (($X + $W -gt $bmp.Width) -or ($Y + $H -gt $bmp.Height)) {
    throw "crop ($X,$Y,$W,$H) falls outside $($bmp.Width)x$($bmp.Height)"
  }
  $lum = [double[,]]::new($W, $H)
  [double]$min = 255.0
  [double]$max = 0.0
  for ($j = 0; $j -lt $H; $j++) {
    for ($i = 0; $i -lt $W; $i++) {
      $c = $bmp.GetPixel($X + $i, $Y + $j)
      [double]$v = 0.299 * $c.R + 0.587 * $c.G + 0.114 * $c.B
      $lum[$i, $j] = $v
      if ($v -lt $min) { $min = $v }
      if ($v -gt $max) { $max = $v }
    }
  }
  [double]$range = $max - $min
  if ($range -le 0) { throw "crop is a flat colour; there is no edge to measure" }
  [double]$gsum = 0.0
  [double]$gmax = 0.0
  [int]$gcount = 0
  [int]$mid = 0
  [int]$tot = 0
  for ($j = 0; $j -lt $H; $j++) {
    for ($i = 0; $i -lt ($W - 1); $i++) {
      [double]$g = [Math]::Abs($lum[($i + 1), $j] - $lum[$i, $j])
      $gsum = $gsum + $g
      $gcount = $gcount + 1
      if ($g -gt $gmax) { $gmax = $g }
    }
    for ($i = 0; $i -lt $W; $i++) {
      $tot = $tot + 1
      [double]$n = ($lum[$i, $j] - $min) / $range
      if (($n -gt 0.25) -and ($n -lt 0.75)) { $mid = $mid + 1 }
    }
  }
  Write-Output ("{0,-20} lum {1,5:N1}..{2,5:N1}  mean|dx| {3,5:N2}  max|dx| {4,5:N1}  mid-band {5,5:N1}%" -f `
    $Label, $min, $max, ($gsum / $gcount), $gmax, (100.0 * $mid / $tot))
}
finally { $bmp.Dispose() }
