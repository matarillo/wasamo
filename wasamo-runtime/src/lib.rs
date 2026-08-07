mod abi;
mod box_values;
// M4-Phase 1 DD-M4-P1-002: the DIP <-> physical-pixel conversion carrier.
// Declared here so the module compiles and its unit tests run; it has no
// production caller until T4 puts the authoritative value on `WindowState`.
mod dip_scale;
mod emit;
// M4-Phase 2 T5: the production focus record and its projection of a live
// `WidgetNode` tree onto `focus_core`. Private -- the phase's cross-task
// obligation is no new ABI function (constraints.md §2), and nothing
// outside this crate needs a `FocusId`.
mod focus;
// M4-Phase 2 DD-M4-P2-003: the Win32-independent focus traversal core,
// consumed through `crate::focus`'s projection of a live `WidgetNode`
// tree (`WindowState`'s keyboard and pointer arms, from T5). `focus_spike`
// is the pre-ADR spike's override-map projection, kept only for
// `tests/focus_mechanism_fixture.rs` until T7 retires it -- see
// process/milestone-4/phase-2/decisions/exploration/.
mod focus_core;
mod focus_spike;
pub mod handler;
// M4-Phase 2 T2: pure-logic hit resolution (DD-M4-P2-002). No Compositor,
// no windows types — see hit.rs's module doc.
mod hit;
pub mod ir_loader;
mod layout;
pub(crate) mod reactive;
mod registry;
mod runtime;
mod text;
mod widget;
mod window;

/// Re-export of the C ABI surface for integration-test consumption. The
/// production callers always go through `wasamo.h` against `wasamo.dll`;
/// this module only exists so cross-crate tests inside this workspace can
/// invoke the ABI without an extra FFI hop.
pub mod ffi {
    pub use crate::abi::*;

    /// Test-only seam exposed through `__install_owning_thread_for_test`
    /// in the runtime module. See its documentation. Intended exclusively
    /// for the workspace's integration tests.
    #[doc(hidden)]
    pub fn __install_owning_thread_for_test() {
        crate::runtime::__install_owning_thread_for_test();
    }

    #[doc(hidden)]
    pub unsafe fn __add_signal_registration_for_test(
        widget: *mut WasamoWidget,
        name: &str,
        callback: WasamoSignalHandlerFn,
        user_data: *mut std::ffi::c_void,
        destroy_fn: WasamoDestroyFn,
    ) -> u64 {
        crate::registry::add_signal(widget, name.to_owned(), callback, user_data, destroy_fn)
    }

    #[doc(hidden)]
    pub fn __signal_tokens_for_test(widget: *mut WasamoWidget, name: &str) -> Vec<u64> {
        crate::registry::signal_tokens_for(widget, name)
    }

    #[doc(hidden)]
    pub fn __registry_entry_count_for_test() -> usize {
        crate::registry::__entry_count_for_test()
    }

    #[doc(hidden)]
    pub fn __runtime_health_for_test() -> &'static str {
        match crate::reactive::runtime_health() {
            crate::reactive::RuntimeHealth::Healthy => "Healthy",
            crate::reactive::RuntimeHealth::Diverged => "Diverged",
        }
    }

    #[doc(hidden)]
    pub fn __reactive_divergence_diagnostics_present_for_test() -> bool {
        crate::reactive::divergence_diagnostics().is_some()
    }

    /// The OS-reported DPI this window's authoritative scale was built from
    /// (M4-Phase 1 T8; T4 finding F-29).
    ///
    /// `WindowState::scale` is `pub(crate)` on purpose — DD-M4-P1-004 walks
    /// every M4 phase and concludes no host needs the scale factor, so the
    /// field stays off the `pub use`-exported surface. The integration tests
    /// are a separate crate and can reach only `pub` items, hence this seam
    /// rather than a widened field.
    ///
    /// **A `u32` DPI, not a `DipScale`.** The carrier stays crate-private,
    /// which is the same resolution T6 reached for
    /// `WidgetNode::__run_layout_as_window_root_at_dpi_for_test`. A DPI is
    /// also the value a test can compare against `GetDpiForWindow` and
    /// against the `HIWORD(wParam)` it synthesised, without reconstructing a
    /// factor.
    ///
    /// # Safety
    ///
    /// `window` must be non-null, properly aligned, and valid for a shared
    /// read of a live `WasamoWindow` for the duration of the call. It must
    /// not be used after `wasamo_window_destroy`, which frees the
    /// `WindowState` this dereferences. The pointer is read and not
    /// retained, so no aliasing obligation outlives the call — but the
    /// runtime is single-threaded by contract and this must be called on the
    /// owning thread, like every other entry on this type.
    #[doc(hidden)]
    pub unsafe fn __window_scale_dpi_for_test(window: *mut WasamoWindow) -> u32 {
        (*window).scale.dpi()
    }

    /// The window's retained hover/press target (M4-Phase 2 T4;
    /// `WindowState::hover`), as the path of child indices from the root
    /// widget, or `None` when nothing is currently entered.
    ///
    /// Exists so a fixture can assert the trap-#3 pairing invariant — the
    /// retained record and the painted `ButtonState` agree — rather than
    /// only the painted state that `WidgetNode::__button_state_for_test`
    /// alone would show.
    ///
    /// # Safety
    ///
    /// Same contract as [`__window_scale_dpi_for_test`]: `window` must be
    /// non-null, properly aligned, and valid for a shared read of a live
    /// `WasamoWindow` for the duration of the call, on the runtime's owning
    /// thread, and not used after `wasamo_window_destroy`.
    #[doc(hidden)]
    pub unsafe fn __hover_target_for_test(window: *mut WasamoWindow) -> Option<Vec<usize>> {
        (*window).hover.target().map(|path| path.to_vec())
    }

    /// The window's focused widget, as the path of child indices from the
    /// root widget (M4-Phase 2 T5; `WindowState::focus`), or `None` when
    /// nothing is focused or no root widget is installed.
    ///
    /// **A reader, not a second store.** The window holds exactly one
    /// record — the focused `FocusId` — and this seam re-projects the live
    /// tree on every call and maps the id back to a path through it, the
    /// same reader shape `crate::focus::focused_path` uses in-crate. There
    /// is no path field anywhere for this seam to read out of step with
    /// the id it names.
    ///
    /// # Safety
    ///
    /// Same contract as [`__window_scale_dpi_for_test`]: `window` must be
    /// non-null, properly aligned, and valid for a shared read of a live
    /// `WasamoWindow` for the duration of the call, on the runtime's owning
    /// thread, and not used after `wasamo_window_destroy`.
    #[doc(hidden)]
    pub unsafe fn __focus_path_for_test(window: *mut WasamoWindow) -> Option<Vec<usize>> {
        let state = &*window;
        let root = state.root_widget.as_deref()?;
        crate::focus::focused_path(root, &state.focus)
    }
}

/// M4-Phase 2 spike-projection seam: `focus_core`'s traversal core plus the
/// override-map projection from a live widget tree (`focus_spike`),
/// reachable from the integration-test crate.
///
/// **Not the production path.** `crate::focus` is: it projects
/// `WidgetNode::focus_role()` with no override map, and is what
/// `WindowState`'s keyboard and pointer arms actually call from M4-Phase 2
/// T5. This seam exists only for `tests/focus_mechanism_fixture.rs`, which
/// needs the override map to synthesise the `Group` / `ModalScope` /
/// `ActiveItemList` / `ActiveItem` roles no `.ui` surface can author yet
/// (DD-M4-P2-005 lands at T6). Deleted, along with `focus_spike` and the
/// override map, when T7 lands the authored annotations to project
/// through instead.
#[doc(hidden)]
pub mod __focus_spike {
    pub use crate::focus_core::*;
    pub use crate::focus_spike::*;
}

pub use layout::{Alignment, SizeConstraint, WidgetKind};
pub use text::{TextRenderer, TypographyStyle};
pub use widget::{ButtonStyle, DipRect, WidgetNode};
pub use window::WindowState;

use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, TranslateMessage, MSG,
};

// ── Rust-native API (used by examples and future bindings) ───────────────────

pub fn init() -> windows::core::Result<()> {
    runtime::init()
}

pub fn window_create(
    title: &str,
    width: i32,
    height: i32,
) -> windows::core::Result<Box<WindowState>> {
    window::create(title, width, height)
}

pub fn window_show(state: &WindowState) {
    window::show(state);
}

pub fn get_compositor() -> &'static windows::UI::Composition::Compositor {
    &runtime::get().compositor
}

pub fn get_text_renderer() -> &'static TextRenderer {
    &runtime::get().text_renderer
}

/// Attach a widget's Visual directly to the window's Composition tree,
/// without putting it in the window's `root_widget` slot.
///
/// **No layout pass runs**, which is the point of this entry compared with
/// [`window_set_root`] — the caller positions and sizes the Visual itself.
/// Since M4-Phase 1 that also means a **Button-family widget shows no label
/// through this path**: the label Visual's offset and size are written by
/// `sync_visuals`, so a widget that never reaches a layout pass keeps the
/// Composition default of `Size = (0, 0)`. The label surface and its brush
/// *are* created at construction — the label is rasterized, just never
/// composited, so re-creating the surface does not help. Before that change
/// the label happened to carry constructor-time geometry while its background
/// did not, so this path rendered a half-positioned widget. Use
/// [`window_set_root`] for anything that should lay itself out.
pub fn window_add_widget(window: &WindowState, widget: &WidgetNode) -> windows::core::Result<()> {
    use windows::core::Interface;
    use windows::UI::Composition::Visual;
    let child_visual: Visual = widget.visual.cast()?;
    window.root.Children()?.InsertAtTop(&child_visual)?;
    Ok(())
}

/// Install `root` as the window's content tree, taking ownership of the
/// subtree. Performs an initial layout pass against the window's current
/// client size and registers the root for click hit-testing and
/// resize-driven re-layout.
///
/// Counterpart to `window_add_widget` — the latter merely attaches a Visual
/// to the Composition tree without putting the widget into the window's
/// `root_widget` slot, so it is suitable only for direct visual hosting (no
/// layout, no hit-test). IR-loader-driven trees should use `window_set_root`.
pub fn window_set_root(
    window: &mut WindowState,
    root: Box<WidgetNode>,
) -> windows::core::Result<()> {
    window::set_root(window, root)
}

pub fn run() {
    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
            // Drain queued callback emissions between message dispatches —
            // the message-loop iteration boundary is a "no host code is
            // currently inside a wasamo_* call" point (abi_spec §6).
            emit::drain_if_outermost();
        }
    }
}

// C ABI surface (wasamo.h) lives in `abi.rs`. The `mod abi;` declaration above
// is sufficient to register the `#[no_mangle] pub extern "C"` functions for
// linkage; nothing else needs to be re-exported here.
