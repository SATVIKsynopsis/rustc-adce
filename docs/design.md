# ADCE Design

## 1. Motivation

Dead Code Elimination removes computations whose results are not needed by the rest of the program.

Traditional dead-store elimination is primarily value-oriented. ADCE additionally considers whether control-flow regions are necessary. This implementation therefore combines control-flow analysis with MIR local liveness.

## 2. Two-Phase Design

The pass consists of two phases.

### Phase 1: Control-Flow Deadness

The first phase identifies branches whose control-dependent regions contain no observable work.

A custom post-dominator graph is constructed by reversing MIR edges and introducing a virtual exit node connected to all real exit blocks.

Post-dominators are then computed using the existing `rustc_data_structures` dominator implementation.

For a branch block with multiple successors, the pass examines the control-dependent blocks for each successor.

A branch is collapsed only when:

- the branch has multiple successors;
- the relevant blocks are reachable in the post-dominator graph;
- each dependent block contains only `Nop`, `StorageLive`, or `StorageDead` statements;
- each dependent block has a `Goto` terminator;
- cleanup status is compatible between the branch and its merge block.

When these conditions hold, the branch terminator is replaced with a `Goto` to its immediate post-dominator.

`SimplifyCfg::Final` is then run to clean up the resulting MIR.

This is intentionally conservative. Calls, drops, unwind paths, and other potentially observable control flow are not removed by this phase.

## 3. Phase 2: Value Deadness

The second phase uses the existing MIR dataflow analysis `MaybeTransitiveLiveLocals`.

The analysis determines which locals are live at each MIR location.

The implementation currently restricts transformation to a narrow class of assignments:

```text
_local = Rvalue::Use(...)
