### DD-P3-003 — Size constraint model

**Status:** Accepted

**Context:**
Each widget must declare how it occupies space on each axis. The model
determines how VStack/HStack distribute space among children, and how
Rectangle declares its size.

**Options:**

Option A — Three-value enum: `Fixed(f32)`, `Fill`, `Shrink`
- `Fixed(f32)`: explicit pixel size on the given axis.
- `Fill`: expand to consume remaining space after fixed-size children are
  placed. Multiple `Fill` siblings divide remaining space equally.
- `Shrink`: wrap to content (sum of children + spacing + padding for stacks;
  explicit dimensions for Rectangle).
- What you gain: Simple to implement. Covers all M1 DSL examples. Maps
  naturally onto the SwiftUI/Flutter size-model mental model.
- What you give up: No min/max constraints. A `Fill` child inside a `Shrink`
  parent resolves to zero size — a degenerate case that must be documented.

Option B — Min/max constraint system (CSS-style)
- What you gain: Expressive; handles edge cases gracefully.
- What you give up: Significantly more complex. All M1 use cases are
  expressible with Option A.

**Decision:** Option A — `Fixed / Fill / Shrink` three-value model for M1.
Min/max constraints deferred to M2 or later.

**Implementation note (post-implementation):**

In the `measure()` pass, `Fill` returns `0.0` on that axis — it signals "I will take whatever
the parent allocates" rather than declaring a demand. The parent resolves the final size during
`arrange()` by dividing remaining space equally among `Fill` siblings. This means a `Fill`
child inside a `Shrink` parent receives `0.0` (remaining = 0, clamped; per DD-P3-005). This
is documented behaviour, not an error.

Default size values per widget type:

| Widget | Width default | Height default |
|---|---|---|
| `VStack` | `Fill` | `Shrink` |
| `HStack` | `Shrink` | `Fill` |
| `Rectangle` | `Fixed` (caller must specify) | `Fixed` (caller must specify) |

A `Rectangle` with no explicit dimension is treated as an API error
(see DD-P3-005).

---
