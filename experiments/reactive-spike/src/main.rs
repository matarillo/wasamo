//! reactive-spike — Phase 5 close GUI checkpoint.
//!
//! Constructs a counter using `register_binding` directly, without any
//! host-side `set_property` call on click. Demonstrates that the reactive
//! engine drives Text label updates end-to-end through the binding path.
//!
//! Widget tree:
//!   VStack {
//!     Text("Count: 0")   ← bound to Signal `count` via reactive binding
//!     Button("Increment")  ← click handler calls count.set(count.get()+1) only
//!   }

use std::rc::Rc;

use wasamo_runtime::{
    experimental_reactive_spike::{
        register_counter_binding, SpikeBindingHandle, SpikeSignal,
    },
    get_compositor, get_text_renderer, init,
    Alignment,
    run,
    TypographyStyle,
    ButtonStyle, WidgetNode,
    window_set_root, window_create, window_show,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init()?;

    let compositor = get_compositor();
    let renderer   = get_text_renderer();

    let mut window = window_create("Reactive Spike — Counter", 800, 600)?;

    // Build widgets bottom-up.
    let mut label = WidgetNode::text(compositor, renderer, "Count: 0", TypographyStyle::Title)?;
    let mut btn   = WidgetNode::button(compositor, renderer, "Increment", ButtonStyle::Accent)?;

    // Set up the reactive binding BEFORE moving label into the tree.
    // The binding holds a raw pointer to the label node; Box<WidgetNode> is
    // heap-allocated and its address is stable after append_child moves it.
    let count = Rc::new(SpikeSignal::new(0i32));

    // SAFETY: `label` lives inside the window's root_widget for the duration
    // of the event loop. `_binding` is stored on the stack below and outlives
    // `run()`. The label pointer is stable because Box does not move the
    // allocation on ownership transfer.
    let _binding: SpikeBindingHandle = unsafe {
        register_counter_binding(
            label.as_mut() as *mut WidgetNode,
            "root.count",
            &count,
        )
    };

    // Click handler: update the Signal only — no set_property call from the host.
    let count_c = Rc::clone(&count);
    btn.set_clicked(move || {
        count_c.set(count_c.get() + 1);
    });

    // Assemble the widget tree.
    let mut root = WidgetNode::vstack(compositor, 12.0, 24.0, Alignment::Center)?;
    root.append_child(label)?;
    root.append_child(btn)?;

    // Install root and show the window.
    window_set_root(&mut window, root)?;
    window_show(&window);

    // Enter the message loop (blocks until the window is closed).
    run();

    Ok(())
}
