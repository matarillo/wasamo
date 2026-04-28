use std::sync::OnceLock;
use windows::{
    System::DispatcherQueueController,
    Win32::System::WinRT::{
        CreateDispatcherQueueController, DispatcherQueueOptions,
        DQTAT_COM_STA, DQTYPE_THREAD_CURRENT,
    },
    UI::Composition::Compositor,
};

pub struct Runtime {
    pub compositor: Compositor,
    // Kept alive for the process lifetime; dropping it shuts down the DQ.
    _dq_controller: DispatcherQueueController,
}

// Safety: all calls are required to originate from the main thread (§3 of
// architecture.md). No concurrent access occurs.
unsafe impl Send for Runtime {}
unsafe impl Sync for Runtime {}

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn init() -> windows::core::Result<()> {
    if RUNTIME.get().is_some() {
        return Ok(());
    }
    let options = DispatcherQueueOptions {
        dwSize: std::mem::size_of::<DispatcherQueueOptions>() as u32,
        threadType: DQTYPE_THREAD_CURRENT,
        apartmentType: DQTAT_COM_STA,
    };
    let dq_controller = unsafe { CreateDispatcherQueueController(options)? };
    let compositor = Compositor::new()?;
    RUNTIME
        .set(Runtime { compositor, _dq_controller: dq_controller })
        .ok();
    Ok(())
}

pub fn get() -> &'static Runtime {
    RUNTIME.get().expect("wasamo_init() not called")
}
