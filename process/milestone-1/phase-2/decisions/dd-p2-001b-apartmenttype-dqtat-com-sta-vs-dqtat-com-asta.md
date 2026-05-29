### DD-P2-001b — `apartmentType`: `DQTAT_COM_STA` vs `DQTAT_COM_ASTA`

**Status:** Accepted

**Context:**
`CreateDispatcherQueueController` takes a `DispatcherQueueOptions.apartmentType`
that controls how the calling thread's COM apartment is initialized.
Two meaningful values exist for desktop apps:

| Constant | Value | COM apartment |
|---|---|---|
| `DQTAT_COM_ASTA` | 1 | Application STA — WinRT / UWP apartment model |
| `DQTAT_COM_STA` | 2 | Standard STA — classic Win32 COM apartment model |

Microsoft's official `HelloComposition` C++ sample uses `DQTAT_COM_ASTA`
because it is written with C++/WinRT, whose default execution model is ASTA.
However, C++/WinRT is not the execution model of Wasamo.

**Options:**

Option A — `DQTAT_COM_ASTA` (Application STA)
- What you gain: Reentrancy protection — the ASTA model blocks nested
  message-pump reentrancy that can cause subtle bugs and deadlocks. Matches
  the threading model WinRT objects were originally designed for (UWP).
- What you give up: ASTA semantics are UWP-specific. Windows App SDK (WinUI 3)
  explicitly migrated *away* from ASTA to standard STA for desktop apps,
  accepting the reentrancy trade-off in exchange for conventional Win32
  behavior. Wasamo would diverge from the direction of the broader ecosystem.

Option B — `DQTAT_COM_STA` (standard STA)
- What you gain: Follows the Win32 desktop app convention. Aligns with Windows
  App SDK / WinUI 3 desktop, which chose standard STA when moving off UWP.
  No UWP-specific apartment constraints on the message loop.
- What you give up: No automatic reentrancy protection. Nested message-pump
  reentrancy (e.g., from a modal dialog or a blocking call that pumps messages)
  must be handled by the application, not the apartment model.

**Decision:** Option B (`DQTAT_COM_STA`) — Wasamo is a Win32 desktop app,
not UWP. Standard STA is what Windows App SDK uses when targeting desktop,
and it is the natural fit for a framework built on Win32 HWND hosting.

**Note — windows 0.58 feature name:**
`"System_DispatcherQueue"` does not exist in windows 0.58.
`DispatcherQueueController` is defined directly in `Windows::System` (no
sub-feature). The correct addition to `wasamo/Cargo.toml` is `"System"`.
`CreateDispatcherQueueController` and its supporting types
(`DispatcherQueueOptions`, `DQTYPE_THREAD_CURRENT`, `DQTAT_COM_STA`) are
already available via the existing `Win32_System_WinRT` feature.

---
