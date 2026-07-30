//! Mock-free Windows integration evidence for M4-Phase 1 T6.
//!
//! Reads the live WinRT brush properties rather than substituting a mock. The
//! default brush is the positive control: it must report `Uniform` / centred,
//! while a Text widget created through the production helper must report the
//! accepted DD-M4-P1-006 mapping (`None` / top-left aligned).

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use wasamo_runtime::{Alignment, TypographyStyle, WidgetNode};
use windows::core::Interface;
use windows::UI::Color;
use windows::UI::Composition::{
    CompositionDrawingSurface, CompositionStretch, CompositionSurfaceBrush, Visual,
};

#[test]
fn text_surface_brush_overrides_the_scaled_centered_default() {
    run_on_owning_runtime_thread_or_skip("text surface brush mapping", move || {
        let compositor = wasamo_runtime::get_compositor();
        let renderer = wasamo_runtime::get_text_renderer();

        let control_surface = renderer
            .draw_text(
                "non-proportional control",
                TypographyStyle::Body,
                37.25,
                19.5,
                Color {
                    A: 255,
                    R: 255,
                    G: 255,
                    B: 255,
                },
            )
            .expect("draw control surface");
        let default_brush = compositor
            .CreateSurfaceBrushWithSurface(&control_surface)
            .expect("create default surface brush");
        assert_eq!(
            default_brush.Stretch().expect("read default stretch"),
            CompositionStretch::Uniform
        );
        assert_eq!(
            default_brush
                .HorizontalAlignmentRatio()
                .expect("read default horizontal alignment"),
            0.5
        );
        assert_eq!(
            default_brush
                .VerticalAlignmentRatio()
                .expect("read default vertical alignment"),
            0.5
        );

        let node = WidgetNode::text(
            compositor,
            renderer,
            "accepted mapping",
            TypographyStyle::Body,
        )
        .expect("create Text widget");
        let production_brush: CompositionSurfaceBrush = node
            .visual
            .Brush()
            .expect("Text visual brush")
            .cast()
            .expect("Text visual uses a surface brush");
        assert_eq!(
            production_brush.Stretch().expect("read stretch"),
            CompositionStretch::None
        );
        assert_eq!(
            production_brush
                .HorizontalAlignmentRatio()
                .expect("read horizontal alignment"),
            0.0
        );
        assert_eq!(
            production_brush
                .VerticalAlignmentRatio()
                .expect("read vertical alignment"),
            0.0
        );
    });
}

#[test]
fn ceil_surface_mapping_is_observed_at_integer_and_fractional_visual_origins() {
    run_on_owning_runtime_thread_or_skip("text surface source/destination mapping", move || {
        let compositor = wasamo_runtime::get_compositor();
        let renderer = wasamo_runtime::get_text_renderer();
        let mut root =
            WidgetNode::hstack(compositor, 0.0, 0.0, Alignment::Leading).expect("create HStack");
        root.append_child(
            WidgetNode::text(compositor, renderer, "fractional", TypographyStyle::Body)
                .expect("create first Text"),
        )
        .expect("append first Text");
        root.append_child(
            WidgetNode::text(compositor, renderer, "origin", TypographyStyle::Body)
                .expect("create second Text"),
        )
        .expect("append second Text");
        root.run_layout(300.0, 100.0).expect("layout HStack");

        let first_visual: Visual = root.children[0].visual.cast().expect("first Text Visual");
        let second_visual: Visual = root.children[1].visual.cast().expect("second Text Visual");
        let first_origin = first_visual.Offset().expect("first Text offset");
        let second_origin = second_visual.Offset().expect("second Text offset");
        assert_eq!(first_origin.X.fract(), 0.0, "first origin is integer");
        assert_ne!(
            second_origin.X.fract(),
            0.0,
            "the first glyph run's fractional measured width must place the second run at a fractional origin"
        );

        let first_size = first_visual.Size().expect("first Text Visual size");
        let first_brush: CompositionSurfaceBrush = root.children[0]
            .visual
            .Brush()
            .expect("first Text brush")
            .cast()
            .expect("surface brush");
        let first_surface: CompositionDrawingSurface = first_brush
            .Surface()
            .expect("surface")
            .cast()
            .expect("drawing surface");
        let surface_size = first_surface.SizeInt32().expect("surface pixel size");
        assert_eq!(
            surface_size.Width,
            first_size.X.ceil() as i32,
            "surface width must be the exact ceil of the Visual width at identity DPI"
        );
        assert_eq!(
            surface_size.Height,
            first_size.Y.ceil() as i32,
            "surface height must be the exact ceil of the Visual height at identity DPI"
        );
        assert!(
            surface_size.Width as f32 >= first_size.X && surface_size.Height as f32 >= first_size.Y,
            "ceil allocation must cover the exact Visual extent"
        );
        assert!(
            surface_size.Width as f32 != first_size.X || surface_size.Height as f32 != first_size.Y,
            "the control requires a source/destination pair whose sizes differ"
        );
        assert_ne!(
            surface_size.Width as f32 * first_size.Y,
            surface_size.Height as f32 * first_size.X,
            "the control requires a non-proportional source/destination pair"
        );
        assert_eq!(
            first_brush.Stretch().expect("stretch"),
            CompositionStretch::None
        );
        assert_eq!(
            first_brush
                .HorizontalAlignmentRatio()
                .expect("horizontal alignment"),
            0.0
        );
        assert_eq!(
            first_brush
                .VerticalAlignmentRatio()
                .expect("vertical alignment"),
            0.0
        );
    });
}

#[test]
fn authoritative_geometry_scale_preserves_a_stale_raster_retry() {
    run_on_owning_runtime_thread_or_skip("geometry/raster scale separation", move || {
        let compositor = wasamo_runtime::get_compositor();
        let renderer = wasamo_runtime::get_text_renderer();
        let mut root =
            WidgetNode::hstack(compositor, 0.0, 0.0, Alignment::Leading).expect("create HStack");
        root.append_child(
            WidgetNode::text(
                compositor,
                renderer,
                "independent raster retry marker",
                TypographyStyle::Body,
            )
            .expect("create Text"),
        )
        .expect("append Text");

        let initial_brush: CompositionSurfaceBrush = root.children[0]
            .visual
            .Brush()
            .expect("initial Text brush")
            .cast()
            .expect("initial surface brush");
        let initial_surface: CompositionDrawingSurface = initial_brush
            .Surface()
            .expect("initial surface")
            .cast()
            .expect("initial drawing surface");
        let initial_size = initial_surface.SizeInt32().expect("initial surface size");

        root.__run_layout_as_window_root_at_dpi_for_test(300.0, 100.0, 120)
            .expect("120-DPI geometry pass");
        let text_visual: Visual = root.children[0].visual.cast().expect("Text Visual");
        let scaled_visual_size = text_visual.Size().expect("scaled Text Visual size");
        let stale_brush: CompositionSurfaceBrush = root.children[0]
            .visual
            .Brush()
            .expect("stale Text brush")
            .cast()
            .expect("stale surface brush");
        let stale_surface: CompositionDrawingSurface = stale_brush
            .Surface()
            .expect("stale surface")
            .cast()
            .expect("stale drawing surface");
        assert_eq!(
            stale_surface.SizeInt32().expect("stale surface size"),
            initial_size,
            "geometry-only scaling must not claim that the text surface was refreshed"
        );
        assert!(
            initial_size.Width < scaled_visual_size.X.ceil() as i32
                || initial_size.Height < scaled_visual_size.Y.ceil() as i32,
            "the control requires the 96-DPI surface to be observably stale at 120 DPI"
        );

        // The public standalone layout entry derives its target from the now
        // committed geometry cache. A shared marker would skip here; the
        // independent raster marker must still trigger the replacement.
        root.run_layout_as_window_root(300.0, 100.0)
            .expect("retry stale text refresh");
        let refreshed_brush: CompositionSurfaceBrush = root.children[0]
            .visual
            .Brush()
            .expect("refreshed Text brush")
            .cast()
            .expect("refreshed surface brush");
        let refreshed_surface: CompositionDrawingSurface = refreshed_brush
            .Surface()
            .expect("refreshed surface")
            .cast()
            .expect("refreshed drawing surface");
        let refreshed_size = refreshed_surface
            .SizeInt32()
            .expect("refreshed surface size");
        assert_eq!(refreshed_size.Width, scaled_visual_size.X.ceil() as i32);
        assert_eq!(refreshed_size.Height, scaled_visual_size.Y.ceil() as i32);
        assert_ne!(
            refreshed_size, initial_size,
            "the retry must install a new device-resolution surface"
        );
    });
}
