# Pixel-compare two frame sets captured by capture-t3-label-writes.ps1.
#
# Introduced at T5. T3 performed the same comparison inline; it is a script
# here because T5 runs it three times (its own baseline against T3's committed
# set, then its baseline against its post-change set, then the F-23 pair) and a
# comparison rule written out three times is the second-home-for-a-rule class
# T2 finding F-14 exists to prevent.
#
# **The comparison is over the client interior, not the captured rectangle.**
# `CopyFromScreen` photographs the whole window rect, whose outer frame is
# alpha-blended against whatever is behind it, so desktop bleed at the border
# can differ between two runs of an identical build. The inset below clears the
# non-client frame at 96 DPI with room to spare — measured at T4:
# 2 x (SM_CXSIZEFRAME + SM_CXPADDEDBORDER) = 16 px of width and
# 2 x 8 + SM_CYCAPTION = 39 px of height.
param(
  [Parameter(Mandatory = $true)][string]$Left,
  [Parameter(Mandatory = $true)][string]$Right,
  [int]$InsetX = 12,
  [int]$InsetTop = 44,
  [int]$InsetBottom = 12
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$leftDir = Resolve-Path $Left
$rightDir = Resolve-Path $Right
$failures = 0

foreach ($file in Get-ChildItem -Path $leftDir -Filter *.png | Sort-Object Name) {
  $other = Join-Path $rightDir $file.Name
  if (-not (Test-Path $other)) {
    Write-Host ("{0,-28} MISSING on the right" -f $file.Name)
    $failures++
    continue
  }
  $a = [System.Drawing.Bitmap]::FromFile($file.FullName)
  $b = [System.Drawing.Bitmap]::FromFile($other)
  try {
    if ($a.Width -ne $b.Width -or $a.Height -ne $b.Height) {
      Write-Host ("{0,-28} SIZE {1}x{2} vs {3}x{4}" -f $file.Name, $a.Width, $a.Height, $b.Width, $b.Height)
      $failures++
      continue
    }
    $x0 = $InsetX; $x1 = $a.Width - $InsetX
    $y0 = $InsetTop; $y1 = $a.Height - $InsetBottom
    $diff = 0
    $total = ($x1 - $x0) * ($y1 - $y0)
    for ($y = $y0; $y -lt $y1; $y++) {
      for ($x = $x0; $x -lt $x1; $x++) {
        if ($a.GetPixel($x, $y).ToArgb() -ne $b.GetPixel($x, $y).ToArgb()) { $diff++ }
      }
    }
    $verdict = if ($diff -eq 0) { "identical" } else { "DIFFER"; }
    Write-Host ("{0,-28} {1,8} of {2} differing pixels over the client interior  {3}" -f `
      $file.Name, $diff, $total, $verdict)
    if ($diff -ne 0) { $failures++ }
  }
  finally {
    $a.Dispose()
    $b.Dispose()
  }
}

if ($failures -ne 0) {
  Write-Host "`n$failures frame(s) differ."
} else {
  Write-Host "`nall frames identical over the client interior."
}
