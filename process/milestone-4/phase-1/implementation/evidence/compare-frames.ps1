# Pixel-compare two frame sets captured by capture-t3-label-writes.ps1.
#
# Introduced at T5. T3 performed the same comparison inline; it is a script
# here because T5 runs it several times (its own baseline against T3's committed
# set, then its baseline against its post-change set, then the F-23 pair) and a
# comparison rule written out that many times is the second-home-for-a-rule
# class T2 finding F-14 exists to prevent.
#
# **The comparison is over the client interior, not the captured rectangle.**
# `CopyFromScreen` photographs the whole window rect, whose outer frame is
# alpha-blended against whatever is behind it, so desktop bleed at the border
# can differ between two runs of an identical build. The inset below clears the
# non-client frame at 96 DPI with room to spare — measured at T4:
# 2 x (SM_CXSIZEFRAME + SM_CXPADDEDBORDER) = 16 px of width and
# 2 x 8 + SM_CYCAPTION = 39 px of height.
#
# ── What the max-delta column is, and what it is not ─────────────────────────
#
# T5 finding F-33 measured that two captures of the **identical build** are not
# always identical: same session but the first launch, 149 of 827,904 pixels;
# same code a day apart, 25. Classified, every differing pixel was a text pixel,
# none flipped between background and covered, and the intensity change was
# bounded at **13** per channel.
#
# **The delta does not classify a difference.** Three counterexamples fix its
# limits:
#
#   * A rasterization defect changes intensity **without** moving geometry. A
#     wrong D2D context DPI is exactly that — T6's defining failure.
#   * A sub-pixel positional error changes edge coverage without necessarily
#     flipping any pixel between covered and uncovered.
#   * Contrast is a property of the edge, not of the change. Two adjacent
#     colours less than the threshold apart give a geometry move a small delta,
#     so the reading is not palette-independent.
#
# So the column is **asymmetric evidence** and is reported as such:
#
#   * A **large** delta proves the difference is **outside the measured drift
#     bound**, and that is all it proves. It does not say *what* moved: an
#     intensity-only rasterization defect can exceed the bound too, so
#     "geometry moved" cannot be inferred from a large delta either.
#   * A **small** delta proves nothing. It is consistent with the measured
#     capture drift and equally consistent with a real intensity-only change.
#
# The default is therefore **any difference exits non-zero.** `-AllowDrift`
# opts into treating a small-delta difference as a pass — a judgement the
# runner makes deliberately and records, never a default a gate can inherit.
param(
  [Parameter(Mandatory = $true)][string]$Left,
  [Parameter(Mandatory = $true)][string]$Right,
  [int]$InsetX = 12,
  [int]$InsetTop = 44,
  [int]$InsetBottom = 12,
  [int]$MaxDriftDelta = 16,
  [switch]$AllowDrift
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$leftDir = Resolve-Path $Left
$rightDir = Resolve-Path $Right

# Compare the file *sets* first. Enumerating only the left side would let a
# frame added on the right pass unnoticed.
$leftNames = @(Get-ChildItem -Path $leftDir -Filter *.png | ForEach-Object { $_.Name })
$rightNames = @(Get-ChildItem -Path $rightDir -Filter *.png | ForEach-Object { $_.Name })
$onlyLeft = @($leftNames | Where-Object { $rightNames -notcontains $_ })
$onlyRight = @($rightNames | Where-Object { $leftNames -notcontains $_ })

# A comparison with nothing in it is a mistake in the invocation, not a pass.
if ($leftNames.Count -eq 0 -or $rightNames.Count -eq 0) {
  Write-Host "no PNGs on the $(if ($leftNames.Count -eq 0) { 'left' } else { 'right' }) side — nothing was compared."
  exit 1
}

$material = 0
$inconclusive = 0

foreach ($n in $onlyLeft) { Write-Host ("{0,-28} MISSING on the right" -f $n); $material++ }
foreach ($n in $onlyRight) { Write-Host ("{0,-28} EXTRA on the right" -f $n); $material++ }

foreach ($name in ($leftNames | Where-Object { $rightNames -contains $_ } | Sort-Object)) {
  $imgA = [System.Drawing.Bitmap]::FromFile((Join-Path $leftDir $name))
  $imgB = [System.Drawing.Bitmap]::FromFile((Join-Path $rightDir $name))
  try {
    if ($imgA.Width -ne $imgB.Width -or $imgA.Height -ne $imgB.Height) {
      Write-Host ("{0,-28} SIZE {1}x{2} vs {3}x{4}" -f $name, $imgA.Width, $imgA.Height, $imgB.Width, $imgB.Height)
      $material++
      continue
    }
    $x0 = $InsetX; $x1 = $imgA.Width - $InsetX
    $y0 = $InsetTop; $y1 = $imgA.Height - $InsetBottom
    $diff = 0
    $maxDelta = 0
    $total = ($x1 - $x0) * ($y1 - $y0)
    for ($y = $y0; $y -lt $y1; $y++) {
      for ($x = $x0; $x -lt $x1; $x++) {
        $ca = $imgA.GetPixel($x, $y); $cb = $imgB.GetPixel($x, $y)
        if ($ca.ToArgb() -eq $cb.ToArgb()) { continue }
        $diff++
        $d = [Math]::Max([Math]::Abs($ca.R - $cb.R),
             [Math]::Max([Math]::Abs($ca.G - $cb.G), [Math]::Abs($ca.B - $cb.B)))
        if ($d -gt $maxDelta) { $maxDelta = $d }
      }
    }
    if ($diff -eq 0) {
      Write-Host ("{0,-28} {1,8} of {2}   identical" -f $name, $diff, $total)
    }
    elseif ($maxDelta -gt $MaxDriftDelta) {
      Write-Host ("{0,-28} {1,8} of {2}   max delta {3,3}  MATERIAL — exceeds the measured drift bound" -f `
        $name, $diff, $total, $maxDelta)
      $material++
    }
    elseif ($AllowDrift) {
      Write-Host ("{0,-28} {1,8} of {2}   max delta {3,3}  small-delta, allowed by -AllowDrift" -f `
        $name, $diff, $total, $maxDelta)
      $inconclusive++
    }
    else {
      Write-Host ("{0,-28} {1,8} of {2}   max delta {3,3}  DIFFERS, INCONCLUSIVE — consistent with capture drift *and* with an intensity-only real change" -f `
        $name, $diff, $total, $maxDelta)
      $material++
    }
  }
  finally {
    $imgA.Dispose()
    $imgB.Dispose()
  }
}

if ($material -ne 0) {
  Write-Host "`n$material frame(s) differ. A large max delta proves only that the difference exceeds the measured drift bound, not what caused it; a small one proves nothing at all. Do not read this as a pass."
  exit 1
}
elseif ($inconclusive -ne 0) {
  Write-Host "`n$inconclusive frame(s) differ by no more than $MaxDriftDelta per channel and were allowed by -AllowDrift. This is a judgement, not a measurement: record the counts and the max delta with the evidence, and say that the allowance was taken."
  exit 0
}
else {
  Write-Host "`nall frames identical over the client interior."
  exit 0
}
