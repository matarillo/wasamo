//! counter-rust/src/main.rs — Hello Counter example in Rust (M1 host-imperative shape)
//!
//! This program constructs the same widget tree as examples/counter/counter.ui
//! imperatively through the wasamo safe Rust wrapper over the experimental C ABI:
//!
//!   VStack {
//!     Text  { "Count: 0"  font: title }
//!     Button { "Increment" style: accent }
//!   }
//!
//! See examples/counter/counter.ui for the future M2 declarative form.
//! The .ui → runtime lowering (wasamoc codegen) is M2 scope; M1 verifies
//! that the C ABI and Visual Layer work correctly.

use std::cell::Cell;
use std::rc::Rc;

use wasamo::{Runtime, Value, Widget, Window};
use wasamo::experimental::{
    button, text, vstack, WASAMO_BUTTON_STYLE, WASAMO_TEXT_CONTENT, WASAMO_TEXT_STYLE,
};

fn main() -> Result<(), wasamo::Error> {
    // 1. Initialize the runtime (calls wasamo_init; shutdown on drop).
    let rt = Runtime::init()?;

    // 2. Create a window (800 × 600).
    let window = Window::create("Counter", 800, 600)?;

    // 3. Build the widget tree (bottom-up, matching counter.ui).

    // Text: "Count: 0" with title typography (TypographyStyle::Title = 3).
    let label: Widget = text("Count: 0")?;
    label.set_property(WASAMO_TEXT_STYLE, &Value::I32(3))?;

    // Button: "Increment" with accent style (ButtonStyle::Accent = 1).
    let btn: Widget = button("Increment")?;
    btn.set_property(WASAMO_BUTTON_STYLE, &Value::I32(1))?;

    // Shared counter state. Widget is Copy so `label` can be captured
    // in the closure and also passed to vstack below.
    let count = Rc::new(Cell::new(0i32));
    let _conn = btn.on_clicked({
        let count = Rc::clone(&count);
        move || {
            let n = count.get() + 1;
            count.set(n);
            let s = format!("Count: {}", n);
            let _ = label.set_property(WASAMO_TEXT_CONTENT, &Value::String(&s));
        }
    });

    // VStack: label + button.
    // Widget is Copy so both handles remain valid after this call.
    let root: Widget = vstack(&[label, btn])?;

    // 4. Install the root widget and show the window.
    window.set_root(root)?;
    window.show()?;

    // 5. Run the message loop (blocks until the window is closed).
    rt.run();

    Ok(())
}
