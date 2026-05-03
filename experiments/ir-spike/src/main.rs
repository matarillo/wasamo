use wasamo_runtime as wasamo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    wasamo::init()?;

    let mut window = wasamo::window_create("Counter (IR spike)", 400, 300)?;

    let uic_path = concat!(env!("CARGO_MANIFEST_DIR"), "/counter.uic");
    let root = wasamo::experimental_ir_loader::load(uic_path)?;

    wasamo::window_set_root(&mut window, root)?;
    wasamo::window_show(&window);
    wasamo::run();
    Ok(())
}
