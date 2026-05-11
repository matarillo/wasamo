use std::sync::OnceLock;
use std::thread::ThreadId;
use windows::{
    System::DispatcherQueueController,
    Win32::System::WinRT::{
        CreateDispatcherQueueController, DispatcherQueueOptions, DQTAT_COM_STA,
        DQTYPE_THREAD_CURRENT,
    },
    UI::Composition::Compositor,
};

use crate::text::TextRenderer;

pub struct Runtime {
    pub compositor: Compositor,
    pub text_renderer: TextRenderer,
    // Kept alive for the process lifetime; dropping it shuts down the DQ.
    _dq_controller: DispatcherQueueController,
}

// Safety: all calls are required to originate from the main thread (§3 of
// architecture.md). No concurrent access occurs.
unsafe impl Send for Runtime {}
unsafe impl Sync for Runtime {}

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

// Owning UI thread — captured at `wasamo_init` time per DD-M2-P6-005 and
// abi_spec §6. Stored separately from `RUNTIME` so the thread-affinity
// guard can be exercised as pure logic without standing up a live
// Compositor / DispatcherQueueController (CLAUDE.md testing rules: pure
// logic gets unit tests, OS APIs are exercised in CI).
static OWNING_THREAD: OnceLock<ThreadId> = OnceLock::new();

fn capture_owning_thread() {
    let _ = OWNING_THREAD.set(std::thread::current().id());
}

pub fn init() -> windows::core::Result<()> {
    capture_owning_thread();
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
    let text_renderer = TextRenderer::new(&compositor)?;
    RUNTIME
        .set(Runtime {
            compositor,
            text_renderer,
            _dq_controller: dq_controller,
        })
        .ok();
    Ok(())
}

pub fn get() -> &'static Runtime {
    RUNTIME.get().expect("wasamo_init() not called")
}

/// Returns `true` if the runtime has been initialized and the calling
/// thread matches the owning (UI) thread. Returns `false` otherwise —
/// either runtime not initialized, or called from a different thread.
pub fn is_owning_thread() -> bool {
    OWNING_THREAD
        .get()
        .map(|t| *t == std::thread::current().id())
        .unwrap_or(false)
}

/// Whether `wasamo_init` (or the test-only seam below) has run. Used to
/// distinguish "not initialized" from "wrong thread" at the ABI boundary.
pub fn is_initialized() -> bool {
    OWNING_THREAD.get().is_some()
}

/// Test-only seam (DD-M2-P6-005 verification): mark the calling thread as
/// the runtime's owning thread *without* the OS-side initialization that
/// `wasamo_init` performs. Once set, subsequent calls (including
/// `wasamo_init`) cannot change the owning thread.
///
/// The `__` prefix and the explicit `_for_test` suffix mark this as a
/// non-production entry point. Production hosts must use `wasamo_init`.
#[doc(hidden)]
pub fn __install_owning_thread_for_test() {
    capture_owning_thread();
}
