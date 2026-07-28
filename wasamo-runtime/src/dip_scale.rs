// The DIP ↔ physical-pixel conversion carrier (M4-Phase 1 DD-M4-P1-002
// §The carrier of the arithmetic, option U2). Pure logic — no Win32 or WinRT
// dependency — so the rounding contract lives in one unit-testable place
// instead of being re-derived at each conversion seam.
//
// `WindowState` holds the authoritative value and `window::create` realises the
// DIP window size through `window_size_to_physical` (T4). The remaining
// operations are still unreached: the conversion seams call them at T5, and the
// text-surface allocation calls `surface_pixels` at T6. The module-level allow
// below is that forward-pointer and goes away as those tasks land.
#![allow(dead_code)]

/// The DPI at which one DIP is one physical pixel — the "100%" reference, and
/// the value the OS reports unconditionally to a process that has declared no
/// DPI awareness (DD-M4-P1-001).
pub const REFERENCE_DPI: u32 = 96;

/// A window's DIP → physical-pixel conversion factor.
///
/// Layout, authored `.ui` dimensions, `TextRenderer::measure` results and the
/// typography ramp are DIP, where 1 DIP is 1/96 inch. The Composition visual
/// tree and the Win32 pointer message stream are the physical pixels of the
/// window's client area. This type is the only place the two spaces meet
/// (DD-M4-P1-002); the layout engine never receives one.
///
/// **The originating DPI is what is retained, and the factor is derived from
/// it.** T2 landed this the other way round — factor stored, DPI discarded, on
/// the grounds that "every consumer wants the factor rather than the DPI". T4's
/// independent review falsified that premise: [`Self::window_size_to_physical`]
/// wants the DPI, because an `f32` factor cannot express `dpi / 96` exactly for
/// every DPI a custom scaling can produce, and a rounding rule stated on a
/// value the type has already approximated is a rule the type does not keep.
/// Storing the DPI is also strictly *less* drift-prone than the alternative of
/// carrying both: there is one representation, and it is the one from which
/// both answers are exactly derivable.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DipScale {
    dpi: u32,
}

impl DipScale {
    /// 100% — every conversion is the identity. This is the value in force on
    /// an undeclared process and, until T9, on every process.
    pub const IDENTITY: Self = Self { dpi: REFERENCE_DPI };

    /// Construct from an OS-reported DPI — `GetDpiForWindow` at window
    /// creation, or `HIWORD(wParam)` of `WM_DPICHANGED`.
    ///
    /// A DPI of zero is not a scale anything can be converted with: it would
    /// make [`Self::to_dip`] divide by zero and poison every coordinate
    /// downstream with an infinity, far from the cause. It falls back to
    /// [`Self::IDENTITY`].
    ///
    /// This is a floor on the arithmetic, not the scale-1 assumption
    /// DD-M4-P1-001 rejects as option F3. That option guesses at a scale that
    /// is unknown but *real*, and is wrong whenever the guess misses; here
    /// there is no scale to be had at all. Neither documented source can
    /// produce a zero in the first place — `GetDpiForWindow` returns 0 only
    /// for an invalid window handle, and T4 seeds it from a window
    /// `CreateWindowExW` has just returned.
    pub fn from_dpi(dpi: u32) -> Self {
        if dpi == 0 {
            return Self::IDENTITY;
        }
        Self { dpi }
    }

    /// The raw factor, for the consumers that need the number itself rather
    /// than a conversion — the D2D context resolution `96 × s` at T6, and the
    /// scale-ratio assertions at T8.
    ///
    /// Derived rather than stored, and exactly as before: this is the same
    /// `dpi as f32 / 96.0` expression the factor field used to be initialised
    /// with, so every `f32` conversion below is bit-identical to what T2
    /// landed. What changed is that the *integer* rule no longer has to start
    /// from this value.
    pub fn factor(self) -> f32 {
        self.dpi as f32 / REFERENCE_DPI as f32
    }

    // ── Outbound: DIP → physical ─────────────────────────────────────────────

    /// A single length.
    pub fn to_physical(self, dip: f32) -> f32 {
        dip * self.factor()
    }

    /// A Visual's physical extent (`SetSize`).
    ///
    /// The result depends only on the DIP extent, so two widgets of equal DIP
    /// size receive bit-identical physical sizes wherever they sit — the
    /// property that keeps a WrapPanel's tiles uniform across a row. Deriving
    /// an extent by converting two edges and subtracting would not have it.
    pub fn extent_to_physical(self, dip: (f32, f32)) -> (f32, f32) {
        (self.to_physical(dip.0), self.to_physical(dip.1))
    }

    /// A Visual's parent-relative physical offset (`SetOffset`), from the
    /// node's and its parent's absolute DIP positions.
    ///
    /// **Converts once, on the difference** (DD-M4-P1-002 rows 4/5/6): the
    /// subtraction happens in DIP and only its result is multiplied. The two
    /// orders agree in exact arithmetic and differ in `f32` — taking the
    /// difference first has one rounding instead of two. The signature exists
    /// to make that the natural call: a caller holding two absolute DIP
    /// positions cannot reach the two-rounding form without converting each
    /// operand itself.
    pub fn relative_offset_to_physical(
        self,
        abs_dip: (f32, f32),
        parent_abs_dip: (f32, f32),
    ) -> (f32, f32) {
        (
            self.to_physical(abs_dip.0 - parent_abs_dip.0),
            self.to_physical(abs_dip.1 - parent_abs_dip.1),
        )
    }

    // ── Inbound: physical → DIP ──────────────────────────────────────────────

    /// A single length.
    pub fn to_dip(self, px: f32) -> f32 {
        px / self.factor()
    }

    /// A physical pair — a pointer position, a window client extent, or a
    /// `Visual.Offset` / `Visual.Size` readback.
    ///
    /// Inbound needs no difference-taking counterpart to
    /// [`Self::relative_offset_to_physical`]: positions and extents alike are
    /// one componentwise division, because the values arrive absolute and are
    /// consumed absolute. The asymmetry is in the outbound direction only,
    /// where the visual tree wants a parent-relative offset.
    pub fn pair_to_dip(self, px: (f32, f32)) -> (f32, f32) {
        (self.to_dip(px.0), self.to_dip(px.1))
    }

    // ── The surface-allocation rounding rule ─────────────────────────────────

    /// Pixel dimensions for a text rasterization surface covering `dip` DIP.
    ///
    /// Rounds **up**. Truncating clips the final column or row of glyph
    /// coverage — a defect that appears only at non-integer scales and reads
    /// as "the last letter is cut off" (DD-M4-P1-002 §The rounding contract
    /// for surfaces). The Visual's size stays the exact `f32`
    /// [`Self::extent_to_physical`] value; the at-most-one-pixel excess in the
    /// surface is transparent padding, which is what keeps the surface brush's
    /// texel-to-pixel mapping one-to-one.
    ///
    /// The integer return type is part of the contract: this is a pixel count,
    /// not a length a later cast could truncate back. Each axis is floored at
    /// one pixel, preserving `draw_text`'s existing `max(1.0)` — a zero-extent
    /// surface is not allocatable. Because `f32 as u32` saturates, a negative
    /// or non-finite input therefore yields one pixel rather than zero, and an
    /// infinite one yields `u32::MAX`, which the WinRT allocation rejects as it
    /// should rather than silently producing a wrong-sized surface.
    pub fn surface_pixels(self, dip: (f32, f32)) -> (u32, u32) {
        (
            Self::pixel_count(self.to_physical(dip.0)),
            Self::pixel_count(self.to_physical(dip.1)),
        )
    }

    fn pixel_count(physical: f32) -> u32 {
        (physical.ceil() as u32).max(1)
    }

    // ── The window-size rounding rule ────────────────────────────────────────

    /// Physical dimensions for a DIP outer-window rectangle — the argument
    /// `CreateWindowExW` and `SetWindowPos` interpret once the process is
    /// DPI-aware (DD-M4-P1-003 §Initial scale acquisition; DD-M4-P1-004 §What
    /// `width` / `height` denote).
    ///
    /// **Rounds to nearest, resolving an exact half away from zero** — the
    /// `MulDiv(value, dpi, 96)` semantics, computed here in integers rather
    /// than borrowed by name.
    ///
    /// The direction deliberately differs from [`Self::surface_pixels`]. That
    /// rule rounds *up* because a surface is an allocation whose consumer
    /// assumes it, and a truncated one clips the final column of glyph
    /// coverage. A window rectangle is not an allocation of that kind: layout
    /// reads the realised client extent back through `GetClientRect` and
    /// converts it, so a half-pixel-smaller window produces a layout for the
    /// window that exists rather than one overflowing a window that does not.
    /// (It does **not** follow that nothing can ever be clipped — a fixed-size
    /// or overflowing subtree can exceed any client rectangle. The asymmetry
    /// being claimed is only that this quantity carries no
    /// allocate-at-least-as-much obligation, T4 independent review finding
    /// R-7.) What it carries instead is a fidelity contract — an 800 DIP window
    /// is meant to *be* 800 DIP on every monitor — so the physical integer to
    /// pick is the one whose DIP value is nearest what was asked for. `ceil`
    /// and `trunc` each bias the realised logical size in a fixed direction for
    /// no stated reason.
    ///
    /// **The arithmetic is exact for every DPI, which is why this operation is
    /// the reason the type retains the DPI** (see the type's own note). The
    /// obvious implementation — multiply by [`Self::factor`] and round —
    /// silently is not the stated rule: an `f32` cannot hold `dpi / 96` exactly
    /// unless the DPI is a multiple of 24, and Windows custom scaling can
    /// produce others. Measured over `dpi` 1–600 × `dip` 1–4000: the `f32`
    /// route disagrees with this one on 21,190 of 2.4M pairs, always by one
    /// pixel and always on an input whose true product is an exact half — for
    /// instance 804 DIP at 100 DPI, which is exactly 837.5 and which the `f32`
    /// route rounds *down* to 837. Every one of the ten standard Windows
    /// scalings has an exactly-representable factor, so none of them can show
    /// the difference; that is the same shape as F-13 and is precisely why the
    /// rule is implemented rather than approximated.
    ///
    /// Integer in, integer out, so no caller holds an `f32` to cast with
    /// `as i32` — which truncates. At 100% the conversion is the exact identity
    /// for every `i32`, including values above 2^24 that `f32` cannot
    /// represent.
    pub fn window_size_to_physical(self, dip: (i32, i32)) -> (i32, i32) {
        (
            self.length_to_physical_i32(dip.0),
            self.length_to_physical_i32(dip.1),
        )
    }

    fn length_to_physical_i32(self, dip: i32) -> i32 {
        // i64 holds i32::MAX * any plausible DPI with room to spare, so the
        // product cannot overflow before the rounding term is added.
        let scaled = dip as i64 * self.dpi as i64;
        let half = REFERENCE_DPI as i64 / 2;
        // Rust integer division truncates toward zero, so biasing by half in
        // the operand's own direction is round-half-away-from-zero.
        let biased = if scaled >= 0 {
            scaled + half
        } else {
            scaled - half
        };
        // Saturate rather than wrap, matching the `f32 as i32` cast this
        // replaced: a nonsensical input must not become a plausible one.
        (biased / REFERENCE_DPI as i64).clamp(i32::MIN as i64, i32::MAX as i64) as i32
    }
}

/// 100%, so that a not-yet-attached widget tree converts as the identity.
///
/// Hand-written rather than derived: a derived `Default` would produce a zero
/// factor, which is the one value that cannot convert anything.
impl Default for DipScale {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The three scale factors the phase's verification item 1 names, plus the
    /// 100% reference. All four are exactly representable in `f32`, which is
    /// why the construction test below can assert equality rather than a bound.
    const DPI_100: u32 = 96;
    const DPI_125: u32 = 120;
    const DPI_150: u32 = 144;
    const DPI_200: u32 = 192;

    /// Plausible DIP magnitudes: exact and awkward, small and window-sized.
    const SWEEP: [f32; 9] = [0.0, 0.5, 1.0, 13.0, 13.7, 51.3, 100.0, 799.5, 1234.567];

    #[test]
    fn from_dpi_yields_the_documented_factors() {
        assert_eq!(DipScale::from_dpi(DPI_100).factor(), 1.0);
        assert_eq!(DipScale::from_dpi(DPI_125).factor(), 1.25);
        assert_eq!(DipScale::from_dpi(DPI_150).factor(), 1.5);
        assert_eq!(DipScale::from_dpi(DPI_200).factor(), 2.0);
    }

    #[test]
    fn identity_and_default_are_one_hundred_percent() {
        assert_eq!(DipScale::IDENTITY.factor(), 1.0);
        assert_eq!(DipScale::default(), DipScale::IDENTITY);
        assert_eq!(DipScale::default(), DipScale::from_dpi(DPI_100));

        // The world T2–T8 land into: every conversion is bit-identical to its
        // input, in both directions, so an unconverted site is undetectable
        // here and only T8 drives s != 1 (T1 finding F-4).
        let s = DipScale::IDENTITY;
        for dip in SWEEP {
            assert_eq!(s.to_physical(dip), dip);
            assert_eq!(s.to_dip(dip), dip);
        }
        assert_eq!(s.extent_to_physical((13.7, 51.3)), (13.7, 51.3));
        assert_eq!(s.pair_to_dip((13.7, 51.3)), (13.7, 51.3));
    }

    /// Fires `from_dpi`'s zero branch directly (start-gate trap #4).
    #[test]
    fn zero_dpi_falls_back_to_identity() {
        let s = DipScale::from_dpi(0);
        assert_eq!(s, DipScale::IDENTITY);
        // The point of the fallback: no coordinate becomes an infinity.
        assert_eq!(s.to_dip(125.0), 125.0);
        assert!(s.to_dip(125.0).is_finite());
    }

    #[test]
    fn conversion_at_125_150_200_percent() {
        // 100 DIP is the anchor: each factor is exact, so each product is too.
        assert_eq!(DipScale::from_dpi(DPI_125).to_physical(100.0), 125.0);
        assert_eq!(DipScale::from_dpi(DPI_150).to_physical(100.0), 150.0);
        assert_eq!(DipScale::from_dpi(DPI_200).to_physical(100.0), 200.0);

        // Inbound is the exact inverse for these values.
        assert_eq!(DipScale::from_dpi(DPI_125).to_dip(125.0), 100.0);
        assert_eq!(DipScale::from_dpi(DPI_150).to_dip(150.0), 100.0);
        assert_eq!(DipScale::from_dpi(DPI_200).to_dip(200.0), 100.0);

        // DD-M4-P1-004's window-size claim, as arithmetic: 800 x 600 DIP is
        // 1000 x 750 physical at 125%. T10's measurement check asserts the
        // same numbers against a real HWND — where, per T1 finding F-9, they
        // are not a positive control on their own.
        let s125 = DipScale::from_dpi(DPI_125);
        assert_eq!(s125.extent_to_physical((800.0, 600.0)), (1000.0, 750.0));

        // A non-integer factor applied to a non-integer extent still lands
        // where hand-computation says it does.
        assert_eq!(s125.extent_to_physical((13.0, 51.3)), (16.25, 64.125));
        assert_eq!(
            DipScale::from_dpi(DPI_150).extent_to_physical((13.0, 800.0)),
            (19.5, 1200.0)
        );
    }

    #[test]
    fn position_and_extent_convert_separately() {
        let s = DipScale::from_dpi(DPI_125);

        // A child at absolute DIP (130.4, 47.2) inside a parent at
        // (120.0, 40.0), sized 60 x 24 DIP.
        let offset = s.relative_offset_to_physical((130.4, 47.2), (120.0, 40.0));
        let extent = s.extent_to_physical((60.0, 24.0));
        assert_eq!(offset, ((130.4f32 - 120.0) * 1.25, (47.2f32 - 40.0) * 1.25));
        assert_eq!(extent, (75.0, 30.0));

        // The extent is position-independent: the same DIP size anywhere on
        // the surface yields the bit-identical physical size. This is what
        // makes a row of same-sized tiles stay a row of same-sized tiles.
        for i in 0..64u32 {
            let anywhere = i as f32 * 37.3;
            let _ = s.relative_offset_to_physical((anywhere, anywhere), (0.0, 0.0));
            assert_eq!(s.extent_to_physical((60.0, 24.0)), extent);
        }

        // The offset is *not* position-independent in the same way, and the
        // API does not pretend otherwise: the DIP subtraction rounds, so two
        // pairs with the same real-valued gap can differ by an ulp. Recorded
        // as a fact rather than asserted as an invariant — the invariant that
        // matters is one rounding instead of two, pinned below.
        let near = s.relative_offset_to_physical((13.3, 0.0), (7.7, 0.0)).0;
        let far = s
            .relative_offset_to_physical((1013.3, 0.0), (1007.7, 0.0))
            .0;
        assert!((near - far).abs() < 1.0e-3);
    }

    #[test]
    fn round_trip_error_is_bounded_by_one_ulp() {
        // Two roundings, each at most half an ulp, so the relative error of a
        // DIP -> physical -> DIP round trip is at most one `f32::EPSILON`.
        // Non-exactness is the common case, not the exception: a brute-force
        // sweep of two million consecutive `f32` from 0.1 finds 750,000
        // non-exact round trips at 125% and 500,000 at 150%.
        for dpi in [DPI_125, DPI_150, DPI_200] {
            let s = DipScale::from_dpi(dpi);
            for dip in SWEEP {
                let round_tripped = s.to_dip(s.to_physical(dip));
                if dip == 0.0 {
                    assert_eq!(round_tripped, 0.0);
                    continue;
                }
                let relative = ((round_tripped - dip) / dip).abs();
                assert!(
                    relative <= f32::EPSILON,
                    "dpi={dpi} dip={dip} round_tripped={round_tripped} relative={relative:e}"
                );
            }
        }

        // 200% is a power of two, so its round trip is *exactly* the identity.
        // Worth pinning as its own claim: it is the reason a 200% test cannot
        // stand in for a 125% one.
        let s200 = DipScale::from_dpi(DPI_200);
        for dip in SWEEP {
            assert_eq!(s200.to_dip(s200.to_physical(dip)), dip);
        }
    }

    #[test]
    fn round_trip_rounds_to_nearest_in_both_directions() {
        // The rounding *direction* of a value conversion is not one-sided: it
        // is IEEE round-to-nearest, so a round trip can come back below or
        // above its input. No conversion site may lean on an inequality
        // surviving one. The two witnesses are consecutive-ulp neighbours of
        // 0.1 found by brute-force search, written via `from_bits` so the
        // exact inputs are not at the mercy of literal parsing.
        let s = DipScale::from_dpi(DPI_125);
        let base = 0.1f32.to_bits();

        let rounds_down = f32::from_bits(base + 2);
        assert!(s.to_dip(s.to_physical(rounds_down)) < rounds_down);

        let rounds_up = f32::from_bits(base + 4);
        assert!(s.to_dip(s.to_physical(rounds_up)) > rounds_up);

        // The surface rule below is the deliberate contrast: there, and only
        // there, the direction is fixed and upward.
    }

    #[test]
    fn surface_allocation_rounds_up() {
        // 13 DIP at 125% is 16.25 px. Truncation allocates 16 and clips the
        // last column of glyph coverage; the contract allocates 17.
        assert_eq!(
            DipScale::from_dpi(DPI_125).surface_pixels((13.0, 51.3)),
            (17, 65)
        );
        assert_eq!(
            DipScale::from_dpi(DPI_150).surface_pixels((13.0, 13.5)),
            (20, 21)
        );
        // At 200% an integer DIP extent is already an integer pixel count, so
        // this case cannot distinguish ceil from trunc — 13.7 DIP can.
        assert_eq!(
            DipScale::from_dpi(DPI_200).surface_pixels((13.0, 13.7)),
            (26, 28)
        );

        // An exact product is not rounded up past itself: 100 DIP at 125% is
        // 125 px, not 126.
        assert_eq!(
            DipScale::from_dpi(DPI_125).surface_pixels((100.0, 800.0)),
            (125, 1000)
        );

        // The brush mapping stays one-to-one: the surface covers the Visual's
        // exact physical size with at most one pixel of transparent padding
        // per axis, never less than it.
        for dpi in [DPI_100, DPI_125, DPI_150, DPI_200] {
            let s = DipScale::from_dpi(dpi);
            for dip in SWEEP {
                if dip == 0.0 {
                    continue;
                }
                let (px, _) = s.surface_pixels((dip, dip));
                let exact = s.to_physical(dip);
                let excess = px as f32 - exact;
                assert!(
                    (0.0..1.0).contains(&excess),
                    "dpi={dpi} dip={dip} px={px} exact={exact} excess={excess}"
                );
            }
        }
    }

    /// Fires `surface_pixels`' one-pixel floor directly (start-gate trap #4),
    /// on each of the three ways to reach it.
    #[test]
    fn surface_allocation_is_never_zero_pixels() {
        let s = DipScale::from_dpi(DPI_125);
        assert_eq!(s.surface_pixels((0.0, 0.0)), (1, 1));
        assert_eq!(s.surface_pixels((-5.0, -0.25)), (1, 1));
        assert_eq!(s.surface_pixels((f32::NAN, f32::NAN)), (1, 1));
        // A sub-pixel extent still gets a whole pixel, per the same ceil.
        assert_eq!(s.surface_pixels((0.1, 0.7)), (1, 1));
        // Documented rather than defended: an infinite extent saturates, and
        // the WinRT allocation is left to reject it (T6 trap #6) instead of a
        // silently wrong-sized surface being produced here.
        assert_eq!(s.surface_pixels((f32::INFINITY, 1.0)), (u32::MAX, 2));
    }

    #[test]
    fn window_size_converts_at_125_150_200_percent() {
        // DD-M4-P1-004's window-size claim in the integer form the ABI
        // actually carries, where `conversion_at_125_150_200_percent` asserts
        // it as `f32` extents: 800 x 600 DIP is 1000 x 750 physical at 125%.
        let sizes = [
            (DPI_125, (1000, 750)),
            (DPI_150, (1200, 900)),
            (DPI_200, (1600, 1200)),
        ];
        for (dpi, expected) in sizes {
            assert_eq!(
                DipScale::from_dpi(dpi).window_size_to_physical((800, 600)),
                expected,
                "dpi={dpi}"
            );
        }
    }

    #[test]
    fn window_size_rounds_to_nearest_in_both_directions() {
        // The cases that discriminate the rule. Every size whose product is
        // exact -- 800 x 600 at any of the phase's three factors, including
        // T10's window-measurement check -- is satisfied by ceil, trunc and
        // round alike, which is the F-13 shape: the rule is pinned by an
        // awkward number here or it is not pinned at all.
        let s125 = DipScale::from_dpi(DPI_125);
        // 801 DIP at 125% is 1001.25. Nearest is down; `ceil` would give 1002.
        assert_eq!(s125.window_size_to_physical((801, 801)), (1001, 1001));
        // 803 DIP at 125% is 1003.75. Nearest is up; `trunc` would give 1003.
        assert_eq!(s125.window_size_to_physical((803, 803)), (1004, 1004));
        // The one input where the nearest integer is not unique: 801 DIP at
        // 150% is exactly 1201.5, resolved away from zero as `MulDiv` does.
        assert_eq!(
            DipScale::from_dpi(DPI_150).window_size_to_physical((801, 801)),
            (1202, 1202)
        );
        // Each axis is converted independently, so a mutation that reuses one
        // axis for both does not pass unnoticed.
        assert_eq!(s125.window_size_to_physical((801, 803)), (1001, 1004));
    }

    /// The rule is `MulDiv(dip, dpi, 96)` for **every** DPI, not only for the
    /// ones whose factor an `f32` happens to hold exactly (T4 independent
    /// review finding R-1). The witness is the reviewer's: 804 DIP at 100 DPI
    /// is exactly 837.5, and multiplying by the `f32` factor yields
    /// 837.4999680519104, which rounds the wrong way.
    #[test]
    fn window_size_is_exact_at_a_dpi_no_f32_factor_can_hold() {
        assert_eq!(
            DipScale::from_dpi(100).window_size_to_physical((804, 804)),
            (838, 838)
        );

        // The same claim as a property, against exact rational arithmetic in
        // i128, over every DPI a custom scaling can plausibly produce. This is
        // the test that would have caught the `f32` route; the standard-scaling
        // cases above cannot, because their factors are exact.
        fn exact_mul_div(dip: i32, dpi: u32) -> i32 {
            let doubled = dip as i128 * dpi as i128 * 2;
            let biased = if dip >= 0 { doubled + 96 } else { doubled - 96 };
            (biased / 192) as i32
        }
        for dpi in [97u32, 100, 101, 110, 123, 137, 149, 175, 199, 250, 401] {
            let s = DipScale::from_dpi(dpi);
            for dip in [-4000, -804, -1, 0, 1, 13, 96, 240, 799, 800, 804, 4000] {
                assert_eq!(
                    s.window_size_to_physical((dip, dip)),
                    (exact_mul_div(dip, dpi), exact_mul_div(dip, dpi)),
                    "dpi={dpi} dip={dip}"
                );
            }
        }
    }

    /// The integer rule computes in `i64`, so a DIP size whose physical
    /// equivalent leaves `i32` must **saturate**, as the `f32 as i32` cast this
    /// replaced did — not wrap, which would turn an absurd request into a
    /// plausible one of the opposite sign. Added because the mutation run found
    /// the case unguarded: no other test reaches the clamp, since a 100%
    /// identity never leaves the range it started in.
    #[test]
    fn window_size_saturates_rather_than_wrapping() {
        let s400 = DipScale::from_dpi(384);
        assert_eq!(
            s400.window_size_to_physical((i32::MAX, i32::MAX)),
            (i32::MAX, i32::MAX)
        );
        assert_eq!(
            s400.window_size_to_physical((i32::MIN, i32::MIN)),
            (i32::MIN, i32::MIN)
        );
        // Just inside the range is not clamped, so the assertions above are
        // about the boundary rather than about the factor.
        assert_eq!(
            DipScale::from_dpi(192).window_size_to_physical((1_000_000, -1_000_000)),
            (2_000_000, -2_000_000)
        );
    }

    #[test]
    fn window_size_is_the_exact_identity_at_one_hundred_percent() {
        // The world T2-T8 land into, for this operation, stated as a property
        // of every `i32` rather than of the sizes hosts happen to pass.
        // 16_777_217 is 2^24 + 1, the first integer `f32` cannot represent:
        // an `f32` multiplication returns 16_777_216 here even at a factor of
        // exactly one.
        for s in [
            DipScale::IDENTITY,
            DipScale::default(),
            DipScale::from_dpi(DPI_100),
        ] {
            for dip in [0, 1, -1, 800, 16_777_217, i32::MAX, i32::MIN] {
                assert_eq!(
                    s.window_size_to_physical((dip, dip)),
                    (dip, dip),
                    "dip={dip}"
                );
            }
        }
    }

    #[test]
    fn converting_once_on_the_difference_differs_from_converting_twice() {
        // Witness found by brute-force search over plausible DIP coordinate
        // pairs, not chosen by hand: a hand-picked pair that happens to agree
        // would make this test pass against either arithmetic.
        //
        // A node at absolute DIP x = 10.1 inside a parent at 5.7, at 150%.
        let s = DipScale::from_dpi(DPI_150);
        let abs = 10.1f32;
        let parent = 5.7f32;

        let once = s.relative_offset_to_physical((abs, 0.0), (parent, 0.0)).0;
        let twice = s.to_physical(abs) - s.to_physical(parent);

        // 1. The two orders really do differ in `f32` — the premise of the
        //    rule, which would otherwise be a style preference.
        assert_ne!(once, twice);

        // 2. The one-rounding form is the correctly-rounded answer and the
        //    two-rounding form is not: `once` equals the real-valued result
        //    rounded to `f32`, `twice` is an ulp away from it.
        let exact = (abs as f64 - parent as f64) * 1.5f64;
        assert_eq!(once, exact as f32);
        assert_ne!(twice, exact as f32);

        // 3. The type's API yields the correct one without the caller
        //    choosing: `relative_offset_to_physical` takes the two absolute
        //    DIP positions, so reaching `twice` requires converting each
        //    operand by hand first.
    }

    #[test]
    fn the_difference_rule_is_only_observable_at_non_dyadic_scales() {
        // At 200% multiplication by the factor is exact, so both orders agree
        // and no amount of testing at 200% can catch a site that converts
        // twice. The brute-force search that produced the witness above found
        // no disagreeing pair at all at 200%. This is why the phase's
        // verification item names 125% and 150% and not only a round number.
        let abs = 10.1f32;
        let parent = 5.7f32;

        let s200 = DipScale::from_dpi(DPI_200);
        assert_eq!(
            s200.relative_offset_to_physical((abs, 0.0), (parent, 0.0))
                .0,
            s200.to_physical(abs) - s200.to_physical(parent)
        );

        let s100 = DipScale::IDENTITY;
        assert_eq!(
            s100.relative_offset_to_physical((abs, 0.0), (parent, 0.0))
                .0,
            s100.to_physical(abs) - s100.to_physical(parent)
        );
    }
}
