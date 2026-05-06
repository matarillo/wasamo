//! Experimental reactive spike helper — gated on feature `experimental-reactive-spike`.
//!
//! Exposes the minimum internal API needed by `experiments/reactive-spike/`
//! to wire up a counter without host-side `set_property` calls. This module
//! follows the same pattern as the Phase 2 `experimental_ir_loader`; it is
//! NOT compiled into release builds and NEVER merged to main.

use std::collections::HashMap;
use std::rc::Rc;

use crate::handler::{HandlerExpr, InterpolationPart};
use crate::reactive::{self, BindingTarget, EffectHandle, Signal, WidgetId};
use crate::widget::{PropertyValue, WidgetNode, PROP_TEXT_CONTENT};

/// An opaque handle to a `Signal<i32>`. Keeps the signal alive and provides
/// get/set access without exposing the internal `Signal<i32>` type.
pub struct SpikeSignal(Signal<i32>);

impl SpikeSignal {
    pub fn new(value: i32) -> Self {
        SpikeSignal(Signal::new(value))
    }

    pub fn get(&self) -> i32 {
        self.0.get_untracked()
    }

    pub fn set(&self, value: i32) {
        self.0.set(value);
    }

    pub(crate) fn inner(&self) -> &Signal<i32> {
        &self.0
    }
}

/// An opaque binding handle. Must be stored for the duration of the binding.
/// Dropping it removes the binding from the reactive graph.
pub struct SpikeBindingHandle(#[allow(dead_code)] EffectHandle);

/// Write a string value into a widget's `PROP_TEXT_CONTENT` property.
///
/// # Safety
/// `id` must be a live `*mut WidgetNode` cast to `*mut ()`, valid on the
/// UI thread for the duration of the call.
fn write_text_content(id: WidgetId, _prop: u32, value: &str) {
    let node = unsafe { &mut *(id.0 as *mut WidgetNode) };
    let _ = node.set_property(PROP_TEXT_CONTENT, &PropertyValue::String(value.to_owned()));
}

/// Register a reactive binding that evaluates a `"Count: \{<key>}"` interpolation
/// against `signal` and writes the resulting string into `PROP_TEXT_CONTENT` of
/// `text_node` whenever the signal changes.
///
/// `prop_key` is the property-map key, e.g. `"root.count"`.
///
/// Returns a `SpikeBindingHandle` that must be stored for the lifetime of the
/// binding.
///
/// # Safety
/// `text_node` must be a live `*mut WidgetNode` owned by a window, valid
/// for at least as long as the returned handle is alive.
pub unsafe fn register_counter_binding(
    text_node: *mut WidgetNode,
    prop_key: &str,
    signal: &SpikeSignal,
) -> SpikeBindingHandle {
    let mut props = HashMap::new();
    props.insert(prop_key.to_owned(), signal.inner().clone());
    let props = Rc::new(props);

    let expr = HandlerExpr::Interpolation(vec![
        InterpolationPart::Literal("Count: ".into()),
        InterpolationPart::Expr(HandlerExpr::PropRead { path: prop_key.to_owned() }),
    ]);

    let id = WidgetId(text_node as *mut ());
    let handle = reactive::register_binding(
        BindingTarget::WidgetProperty { node: id, prop: PROP_TEXT_CONTENT },
        expr,
        props,
        write_text_content,
    );
    SpikeBindingHandle(handle)
}

/// Expose `WidgetNode` for use in the driver crate.
pub use crate::widget::WidgetNode as SpikeWidgetNode;
