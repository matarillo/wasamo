# M4-Phase 1 T11 — compare two single frames with a uniform inset, and locate
# whatever differs.
#
# `compare-frames.ps1` compares six-frame *directories* and carries default
# insets derived from the development machine's 96-DPI non-client frame. T11's
# frames are single owner-captured windows from another machine, so neither the
# directory shape nor those insets apply; this script is the single-file form
# and takes the inset explicitly.
#
# -Map lists where the differences are (bounding box, worst rows, 50-row bands)
# instead of only how many there are. That is what turns "8323 pixels differ"
# into "every differing pixel is on the outermost two pixels of the frame and
# the four corner radii" — a fact about the capture, not about the render.
#
# As in `compare-frames.ps1`: the pixel count and the max per-channel delta
# describe a difference, they do not classify it. A window captured at two
# different desktop positions blends its rounded corners with whatever is
# behind it, which is a large per-channel delta and no regression at all.
param(
  [Parameter(Mandatory=$true)][string]$A,
  [Parameter(Mandatory=$true)][string]$B,
  [int]$Inset = 0,
  [switch]$Map
)
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
$x = [System.Drawing.Bitmap]::FromFile((Resolve-Path $A).Path)
$y = [System.Drawing.Bitmap]::FromFile((Resolve-Path $B).Path)
try {
  if (($x.Width -ne $y.Width) -or ($x.Height -ne $y.Height)) {
    Write-Output ("size mismatch: {0}x{1} vs {2}x{3} — these are not comparable frames" -f `
      $x.Width, $x.Height, $y.Width, $y.Height)
    exit 2
  }
  [int]$diff = 0
  [int]$maxd = 0
  [int]$tot = 0
  [int]$minx = [int]::MaxValue; [int]$maxx = -1
  [int]$miny = [int]::MaxValue; [int]$maxy = -1
  $rows = [int[]]::new($x.Height)
  for ($j = $Inset; $j -lt ($x.Height - $Inset); $j++) {
    for ($i = $Inset; $i -lt ($x.Width - $Inset); $i++) {
      $tot = $tot + 1
      $p = $x.GetPixel($i, $j)
      $q = $y.GetPixel($i, $j)
      if (($p.R -ne $q.R) -or ($p.G -ne $q.G) -or ($p.B -ne $q.B)) {
        $diff = $diff + 1
        $rows[$j] = $rows[$j] + 1
        if ($i -lt $minx) { $minx = $i }
        if ($i -gt $maxx) { $maxx = $i }
        if ($j -lt $miny) { $miny = $j }
        if ($j -gt $maxy) { $maxy = $j }
        $m = [Math]::Max([Math]::Abs($p.R - $q.R), [Math]::Max([Math]::Abs($p.G - $q.G), [Math]::Abs($p.B - $q.B)))
        if ($m -gt $maxd) { $maxd = $m }
      }
    }
  }
  Write-Output ("{0}x{1}  inset {2}  differing {3} of {4}  max per-channel delta {5}" -f `
    $x.Width, $x.Height, $Inset, $diff, $tot, $maxd)
  if ($Map -and ($diff -gt 0)) {
    Write-Output ("  bounding box  x {0}..{1}  y {2}..{3}" -f $minx, $maxx, $miny, $maxy)
    Write-Output "  worst rows:"
    ($Inset)..($x.Height - $Inset - 1) | Sort-Object { -$rows[$_] } | Select-Object -First 8 | ForEach-Object {
      if ($rows[$_] -gt 0) { Write-Output ("    y={0,4}  {1,5} px" -f $_, $rows[$_]) }
    }
  }
  if ($diff -gt 0) { exit 1 }
}
finally { $x.Dispose(); $y.Dispose() }
