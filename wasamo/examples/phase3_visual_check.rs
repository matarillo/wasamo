// Phase 3 visual check: three coloured rectangles inside a VStack, laid out
// by the layout engine and rendered through the Visual Layer.
//
// Expected result:
//   - Window opens (640 × 480) with Mica backdrop (Win11) or plain background
//   - Three blocks stacked vertically: red, green, blue
//   - Each block is 80px tall, fills the available width
//   - 12px spacing between blocks, 24px padding around the stack
//   - Total stack height: 3 × 80 + 2 × 12 + 2 × 24 = 312px

fn main() -> windows::core::Result<()> {
    wasamo::init()?;

    let window = wasamo::window_create("Phase 3 — Layout Engine Check", 640, 480)?;
    let compositor = wasamo::get_compositor();

    let mut vstack = wasamo::WidgetNode::vstack(compositor, 12.0, 24.0, wasamo::Alignment::Stretch)?;

    let rect1 = wasamo::WidgetNode::rectangle(
        compositor,
        wasamo::SizeConstraint::Fill,
        wasamo::SizeConstraint::Fixed(80.0),
    )?;
    rect1.set_color(compositor, 0xC0, 0x40, 0x40)?; // red

    let rect2 = wasamo::WidgetNode::rectangle(
        compositor,
        wasamo::SizeConstraint::Fill,
        wasamo::SizeConstraint::Fixed(80.0),
    )?;
    rect2.set_color(compositor, 0x40, 0xC0, 0x40)?; // green

    let rect3 = wasamo::WidgetNode::rectangle(
        compositor,
        wasamo::SizeConstraint::Fill,
        wasamo::SizeConstraint::Fixed(80.0),
    )?;
    rect3.set_color(compositor, 0x40, 0x40, 0xC0)?; // blue

    vstack.append_child(rect1)?;
    vstack.append_child(rect2)?;
    vstack.append_child(rect3)?;

    wasamo::window_add_widget(&window, &vstack)?;
    vstack.run_layout(640.0, 480.0)?;

    wasamo::window_show(&window);
    wasamo::run();
    Ok(())
}
