<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# PHASE 2C: WINDOWING FOUNDATION ARCHITECTURE
**Status:** DRAFT v0.2 (Pending Final Gate Review)

## 1. Architectural Goals & Philosophy
The Windowing Foundation translates abstract `InputEvent`s into targeted actions and manages visual state without continuously burning CPU cycles. Guided by the Catalyst Philosophy ("Sleep light. Wake fast. Work hard."), this architecture guarantees **Event-Driven Rendering** with strict performance boundaries.

## 2. Core Primitives

### 2.1 WindowId & Stable References
- `WindowId(u32)`: A stable, lightweight token acting as an index into a generational arena.
- Enables safe, predictable lookups without pointer aliasing, `Rc<RefCell>`, or memory fragmentation.

### 2.2 Geometry: Rect & Region
- `Rect`: Basic bounding box `(x, y, width, height)`.
- `Region`: A set of non-overlapping `Rect`s representing complex shapes.

### 2.3 Dirty-Region Calculation & Bounded Heuristic
Region calculation must not consume more CPU time than the rendering it prevents.
- **Cost-Based Region Coalescing:** Instead of a rigid rect limit, merging decisions evaluate: `fragmentation_cost VS full_redraw_cost`.
  - Merging distant small rects into a massive bounding box is prohibited if it drastically increases the total drawn area.
  - Adjacent or highly overlapping regions are coalesced.
- **P2C-PERF-02: Bounded Optimization Budget**
  - Region management shall operate within a fixed computational budget.
  - If the calculation exceeds this budget, the system shall fall back to a simpler redraw strategy (e.g., bounding box or full redraw) rather than continuing unbounded optimization.

## 3. Window Model & Tree Management

### 3.1 Compact Window Metadata
A `Window` contains structural hierarchy and bounds, with no raw pointer ownership.
```rust
struct Window {
    id: WindowId,
    parent_id: Option<WindowId>, // P2C-WM-01
    first_child: Option<WindowId>,
    next_sibling: Option<WindowId>,
    bounds: Rect,
    flags: WindowFlags,
}
```
**P2C-WM-01: Explicit Parent/Child Relationship**
Parent-child relationships must be represented deterministically without raw pointer ownership.

### 3.2 Hierarchical Z-Ordering
- Z-order is fundamentally hierarchical, based on sibling ordering.
- A child window (e.g., a button) cannot violate the Z-order boundaries of its parent.
- While flat traversal arrays may be used internally as an optimization cache, the semantic model remains strictly hierarchical.

## 4. Input Routing & Focus Model

### 4.1 Hit Testing
Hit testing recursively descends the Window Tree (top-most siblings first, then children) to find the deepest visible window containing `(x, y)`.

### 4.2 Pointer Capture
- `MouseButtonDown` grants the target window **Pointer Capture**.
- Subsequent `MouseMove` and `MouseButtonUp` events are routed directly to the captured window, regardless of the cursor's physical location.
- Capture releases explicitly on `MouseButtonUp`.

### 4.3 Keyboard Focus
- A distinct `WindowId` holds Keyboard Focus.
- `KeyDown` / `KeyUp` route strictly to this ID.

## 5. Invalidation & Rendering Contract

### 5.1 Decoupling Invalidation and Rendering
The lifecycle is strictly separated:
`Input -> State Change -> Invalidate(Rect) -> Dirty Region Manager -> [Scheduler Boundary] -> Render()`
- State changes only issue invalidation requests. They do not trigger immediate drawing.
- The `Render()` pass executes subsequently when the compositor/scheduler determines it is optimal.

### 5.2 Cursor Invalidation
**P2C-CURSOR-01: Cursor Ghosting Prevention**
Pointer movement shall explicitly invalidate both the *previous* cursor region and the *current* cursor region when rendered into the framebuffer, merging them into the dirty set: `Dirty = old_cursor ∪ new_cursor`.

### 5.3 The P2C-PERF-01 & P2C-PERF-03 Invariants
**P2C-PERF-01: No Unbounded Redraw.**
- The main loop will NEVER execute a full-screen redraw implicitly. Full redraws are explicit.
- Routine renders only process the `Dirty Region Set`.

**P2C-PERF-03: Idle Quiescence.**
- When there is no pending input, no state changes, no dirty regions, no animation deadline, and no display event, the windowing subsystem enters a **QUIESCENT** state.
- It performs zero unnecessary windowing or rendering work.

## 6. Dependency Boundaries & Future Compatibility
- The rendering boundary is abstracted to support Future Compositors (Software, GPU, Remote Display) without breaking Phase 2C contracts.

## 7. Verification Strategy
Before proceeding to Phase 2D, the implementation must pass:
- **Test 1 - No-op Idle:** Logging proves 0 redraw calls when the system is quiescent.
- **Test 2 - Bounded Redraw:** Modifying a small window only redraws that window's `Rect`.
- **Test 3 - Capture Persistence:** Dragging the mouse outside a clicked window continues routing to that window.
- **Test 4 - Coalescing Limits:** Overlapping regions correctly coalesce based on cost, avoiding massive bounding boxes for distant regions.
- **Test 5 - Old/New Cursor Invalidation:** Moving cursor A -> B generates `dirty = A ∪ B`.
- **Test 6 - No-op Invalidation:** Redundant `invalidate(rect)` calls on the exact same area do not cause pathological region explosion.
- **Test 7 - Window Move:** Moving Window A invalidates both its old bounds (to restore background) and new bounds (to draw window).
- **Test 8 - Occlusion:** A window updating underneath a completely opaque topmost window does not trigger unnecessary pixel rendering (foundational for the future compositor).
