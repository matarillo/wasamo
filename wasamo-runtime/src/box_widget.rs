// Box-widget internal domain types (M3-Phase 2 DD-M3-P2-002 / DD-M3-P2-003,
// variant strategy Option A).
//
// `Ratio` and `Color` are populated by `ir_loader::build_node` from
// `IrLiteral::Ratio` / `IrLiteral::Color` and stored as Box-internal fields
// on `WidgetData::Box`. They deliberately do **not** become `PropertyValue`
// variants in Phase 2 (DD-M3-P2-004 keeps `aspect` / `fill` constant-only),
// and there is no matching `WASAMO_VALUE_*` ABI tag. The seams here are the
// types a future bindable-`fill` / bindable-`aspect` phase would wrap, not
// revise.

/// Aspect ratio carried by `aspect: <num>:<den>` literals.
///
/// `num` and `den` are guaranteed positive by `wasamoc check` (T3) and the
/// runtime IR loader (T7); the runtime never sees zero or negative sides.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Ratio {
    pub num: i32,
    pub den: i32,
}

/// Packed RGBA colour carried by `fill: #RRGGBB` / `fill: #RRGGBBAA` literals.
///
/// The packed `u32` is laid out as `0xAARRGGBB` (alpha in the most significant
/// byte) per dsl_spec §8.2. `#RRGGBB` surface input materialises with implicit
/// alpha `0xFF`; `#RRGGBBAA` carries explicit alpha.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Color(pub u32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratio_construction_and_equality() {
        let a = Ratio { num: 16, den: 9 };
        let b = Ratio { num: 16, den: 9 };
        let c = Ratio { num: 4, den: 3 };
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.num, 16);
        assert_eq!(a.den, 9);
    }

    #[test]
    fn color_packs_alpha_in_msb() {
        // #cccccc → implicit alpha 0xFF → 0xFFCCCCCC
        let opaque = Color(0xFF_CC_CC_CC);
        // #00000080 → explicit alpha 0x80 → 0x80000000
        let translucent = Color(0x80_00_00_00);
        assert_eq!(opaque.0 >> 24, 0xFF);
        assert_eq!(translucent.0 >> 24, 0x80);
        assert_ne!(opaque, translucent);
    }

    #[test]
    fn color_equality_distinguishes_alpha() {
        // Same RGB, different alpha must compare unequal — alpha is part of
        // the packed value, not stripped on comparison.
        let opaque_white = Color(0xFF_FF_FF_FF);
        let translucent_white = Color(0x80_FF_FF_FF);
        assert_ne!(opaque_white, translucent_white);
    }
}
