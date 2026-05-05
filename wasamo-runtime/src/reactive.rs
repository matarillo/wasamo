//! Reactive engine primitives — M2-Phase 5 implementation target.
//!
//! Phase 4 establishes the API boundary (DD-M2-P4-004 = Option A):
//! no host-visible batching ABI is added; the existing queue-and-drain
//! semantics serve as the M2 batching contract. This module provides the
//! internal-only `with_batched_writes` helper that Phase 5 will use to
//! suppress per-write invalidation cascades during a reactive update cycle.
//!
//! The helper is `pub(crate)` and lives here so Phase 5 can implement the
//! dependency tracker and binding evaluator in the same module without
//! exposing anything to the C ABI.

/// Execute `f` with writes batched: invalidation cascades triggered inside
/// `f` are deferred until `f` returns, then flushed once.
///
/// Phase 4 provides the skeleton only; Phase 5 will implement the dependency
/// tracker and dirty-set that this function manages.
pub(crate) fn with_batched_writes<R, F: FnOnce() -> R>(f: F) -> R {
    // Phase 5 TODO: enter batch mode (increment a thread-local depth counter;
    // suppress per-write dirty-set flushes while depth > 0; flush once on exit).
    f()
}
