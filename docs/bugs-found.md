# Bugs and Correctness Lessons

During development, several issues exposed important interactions between ADCE, MIR dataflow, and Rust-specific control flow.

## 1. Dataflow Cursor and Post-Dominator Reachability

An early implementation attempted to reuse a `MaybeTransitiveLiveLocals` results cursor while traversing the custom post-dominator graph.

This caused a runtime compiler ICE in the dataflow cursor because the post-dominator traversal could visit blocks that the MIR dataflow analysis considered unreachable.

### Lesson

The post-dominator graph has a different traversal structure from the forward MIR dataflow graph. The two analyses should therefore remain independent.

The final implementation performs:

1. control-flow analysis using post-dominators;
2. CFG simplification;
3. local liveness analysis using `MaybeTransitiveLiveLocals`.

The liveness cursor is used only during Phase 2.

## 2. Unwind and Cleanup Control Flow

Branches involving calls, drops, or unwind paths cannot be treated as ordinary empty CFG branches.

A transformation that appears structurally dead can still change observable cleanup behavior.

### Lesson

The control-flow phase is conservative around cleanup blocks and requires compatible cleanup status before collapsing a branch.

Regression tests include `evaluation/corpus/adce_dead_branch_unwind.rs`
and `evaluation/corpus/adce_drop_unwind.rs`.

## 3. Self-Targeting and Cyclic CFGs

MIR can contain unusual CFG structures such as:

- a branch targeting itself;
- loops;
- cyclic chains of dead assignments.

These structures are useful for testing both the post-dominator implementation and the liveness phase.

The implementation was tested against self-targeting and cyclic CFGs without
producing an ICE. Regression tests include
`evaluation/corpus/adce_switch_to_self.rs`,
`evaluation/corpus/adce_target_self.rs`, and
`evaluation/corpus/adce_transitive_cycle.rs`.

### Lesson

CFG algorithms must not assume that every successor moves toward a distinct block or that the graph is acyclic.

## 4. Borrowed Locals

A local can appear dead from a simple value-liveness perspective while still being involved in borrowing semantics.

The value-deadness phase therefore excludes locals identified by `borrowed_locals`.
This is regression-tested by `evaluation/corpus/adce_borrowed_local.rs`.

### Lesson

MIR dead-code elimination cannot rely only on whether a value appears unused. Rust's borrowing and cleanup semantics impose additional constraints.

## 5. Debug Information

Removing an assignment to a local can affect the information available to debuginfo.

The implementation therefore excludes locals identified as required for debuginfo from Phase 2 elimination.

This behavior is regression-tested by
`evaluation/corpus/adce_debuginfo_local.rs`.

## 6. Conservative Side-Effect Handling

Calls and drops are deliberately not treated as dead merely because their return values appear unused.

For example, a call may have externally observable side effects, and a `Drop` terminator may execute user-defined destruction code.

### Lesson

ADCE must distinguish between an unused value and an unobservable operation.

The current implementation therefore limits Phase 2 to simple `Rvalue::Use` assignments and leaves calls, drops, and other potentially effectful operations untouched.

The corresponding safety cases include
`evaluation/corpus/adce_call_side_effect.rs` and
`evaluation/corpus/adce_drop_safety.rs`.

## 7. Testing Strategy

The regression corpus was expanded as these cases were discovered.

The tests include both positive transformations and negative safety cases. In particular, the negative cases verify that the pass does not incorrectly remove:

- calls;
- drops;
- unwind behavior;
- borrowed locals;
- debuginfo-sensitive assignments;
- pathological control-flow structures.
