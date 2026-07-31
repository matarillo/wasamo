use std::sync::OnceLock;
use std::thread::ThreadId;
use windows::{
    System::DispatcherQueueController,
    Win32::System::WinRT::{
        CreateDispatcherQueueController, DispatcherQueueOptions, DQTAT_COM_STA,
        DQTYPE_THREAD_CURRENT,
    },
    Win32::UI::HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
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

/// The diagnostic to record for a Per-Monitor-Aware V2 declaration attempt,
/// or `None` when the declaration took effect and there is nothing to
/// disclose (DD-M4-P1-001 §Failure handling, option F2).
///
/// Split out of [`declare_per_monitor_aware_v2`] because the *selection* is
/// pure logic while the call is not: process DPI awareness is one-shot per
/// process, so a test that has watched the runtime declare successfully can
/// never watch it fail, and vice versa. As a free function the branch is
/// exercisable in both directions in one binary.
///
/// **One branch, deliberately.** A third arm separating `ERROR_ACCESS_DENIED`
/// from any other `HRESULT` was considered and rejected: the awareness context
/// is a compile-time constant, so `SetProcessDpiAwarenessContext` has no
/// reachable failure other than "already set" — the arm would be unreachable
/// code written to make a string read better. The message below therefore
/// names the `HRESULT` and names the known cause *as* the known cause rather
/// than asserting it of an outcome it has not seen.
fn declaration_diagnostic(outcome: &windows::core::Result<()>) -> Option<String> {
    let err = outcome.as_ref().err()?;
    Some(format!(
        "wasamo_init: the runtime's Per-Monitor-Aware V2 declaration did not \
         take effect ({err}). The known cause is a process whose DPI awareness \
         was already set — by the host's application manifest or by an earlier \
         call — which is legitimate and is not an error: wasamo_init returned \
         WASAMO_OK and every scale factor is derived from the effective \
         per-window DPI the OS reports. What is not guaranteed under an \
         awareness below Per-Monitor-Aware V2 is crispness, not correctness."
    ))
}

/// Declare the process Per-Monitor-Aware V2, and record the outcome as a
/// diagnostic rather than as a status (DD-M4-P1-001, abi_spec §4.1).
///
/// **The result is deliberately not propagated.** A failure here means the
/// host already declared its own awareness, which is a legitimate thing for a
/// host to have done; failing `wasamo_init` would break it for doing the right
/// thing. There is also no "assume scale 1" fallback, and its absence is
/// load-bearing rather than an omission: DD-M4-P1-001's tolerance of a failed
/// declaration rests on the conversion machinery having exactly one code path,
/// which asks the OS for each window's effective DPI instead of asking whether
/// this call succeeded.
fn declare_per_monitor_aware_v2() {
    let outcome =
        unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
    if let Some(diagnostic) = declaration_diagnostic(&outcome) {
        crate::abi::set_last_error(diagnostic);
    }
}

pub fn init() -> windows::core::Result<()> {
    capture_owning_thread();
    if RUNTIME.get().is_some() {
        return Ok(());
    }
    // The first OS-touching act, and below the one-shot above rather than
    // over it: process awareness can only be set while unset, so a second
    // `wasamo_init` placed above this guard would re-declare and take
    // ERROR_ACCESS_DENIED against the runtime's own earlier, correct
    // declaration. `capture_owning_thread` precedes it and is not OS work
    // that can lock the awareness in.
    declare_per_monitor_aware_v2();
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

#[cfg(test)]
mod tests {
    use super::declaration_diagnostic;
    use windows::core::{Error, HRESULT};

    /// `ERROR_ACCESS_DENIED` as an `HRESULT`, which is what
    /// `SetProcessDpiAwarenessContext` reports through the `windows` crate
    /// when the process's awareness was already set.
    const E_ACCESS_DENIED: HRESULT = HRESULT(0x8007_0005_u32 as i32);

    #[test]
    fn a_declaration_that_took_effect_discloses_nothing() {
        assert_eq!(declaration_diagnostic(&Ok(())), None);
    }

    #[test]
    fn a_declaration_that_did_not_take_effect_discloses_the_consequence() {
        let diagnostic = declaration_diagnostic(&Err(Error::from_hresult(E_ACCESS_DENIED)))
            .expect("a failed declaration must be disclosed");
        // The three things DD-M4-P1-001 requires the disclosure to carry: that
        // the declaration did not take effect, that this is not an error, and
        // what is actually given up. Asserted as content rather than as a
        // string equality, so rewording the message does not redden the test
        // while dropping a clause does.
        assert!(diagnostic.contains("did not take effect"), "{diagnostic}");
        assert!(diagnostic.contains("is not an error"), "{diagnostic}");
        assert!(diagnostic.contains("crispness"), "{diagnostic}");
    }

    /// The disclosure must name the failure it is disclosing. A message that
    /// says only "something went wrong" is what the phase's own ADR calls
    /// claiming more than you deliver, one level down.
    #[test]
    fn the_disclosure_names_the_hresult_it_is_about() {
        let diagnostic = declaration_diagnostic(&Err(Error::from_hresult(E_ACCESS_DENIED)))
            .expect("a failed declaration must be disclosed");
        assert!(
            diagnostic.contains("0x80070005"),
            "the HRESULT is what a developer greps for: {diagnostic}"
        );
    }
}
