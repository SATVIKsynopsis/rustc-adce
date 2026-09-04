# Rust MIR Aggressive Dead Code Elimination (ADCE)

An experimental Aggressive Dead Code Elimination (ADCE) optimization pass for Rust's MIR.

## Overview

This project implements an ADCE pass inside `rustc`'s MIR optimization pipeline.

The pass combines two complementary analyses:

1. **Control-flow deadness**
   - Computes post-dominators over the MIR control-flow graph.
   - Uses control dependence to identify branches whose dependent regions have no observable work.
   - Conservatively collapses such branches to their post-dominating merge point.

2. **Value deadness**
   - Uses Rust's MIR dataflow analysis `MaybeTransitiveLiveLocals`.
   - Identifies dead local assignments.
   - Currently removes a deliberately conservative subset: simple `Rvalue::Use` assignments to non-borrowed, non-debug-info locals.

The implementation is intentionally conservative around calls, drops, unwinding, borrowed locals, and debug information.

## Implementation

The implementation lives in the Rust compiler fork:

- `compiler/rustc_mir_transform/src/adce.rs`
- `compiler/rustc_mir_transform/src/post_dom.rs`

The pass is registered in `rustc_mir_transform/src/lib.rs` and runs as part of the optimized MIR pipeline.

### Pass structure

```text
MIR
 │
 ├── Phase 1: Control-flow deadness
 │     ├── Construct post-dominator graph
 │     ├── Compute post-dominators
 │     ├── Find control-dependent regions
 │     ├── Verify regions contain no observable work
 │     └── Collapse safe dead branches
 │
 ├── SimplifyCfg
 │
 └── Phase 2: Value deadness
       ├── Compute live locals
       ├── Find dead simple assignments
       └── Replace them with Nop

## Safety and Conservatism

The pass avoids transformations when observable behavior may be affected.

In particular, the current implementation preserves:

- calls and their side effects
- `Drop` operations
- unwind/control-flow paths
- borrowed locals
- locals required for debuginfo
- branches whose dependent regions contain observable statements

The control-flow phase also checks cleanup status before collapsing a branch.

## Tests

The `evaluation/corpus/` directory contains the MIR optimization test cases used during development.

The corpus covers:

- dead assignments
- borrowed locals
- debuginfo-sensitive locals
- dead branches
- branches involving unwind paths
- drops
- unreachable/diverging control flow
- cyclic/transitive dead stores
- pathological self-targeting branches
- post-dominator construction

## Evaluation

The pass was evaluated against both synthetic MIR tests and real-world Rust crates.

### OpenTelemetry

Across 619 functions:

- 75 functions changed during the ADCE pass
- Basic blocks: **3,744 → 3,203** (-14.45%)
- Statements: **41,381 → 40,388** (-2.40%)

### Regex

Across 410 functions:

- 10 functions changed during the ADCE pass
- Basic blocks: **1,748 → 1,728** (-1.14%)
- Statements: **22,754 → 22,730** (-0.11%)
- Phase 1 reported **7 control-dependence collapses**

The aggregate before/after counts include the `SimplifyCfg::Final` cleanup that follows Phase 1, so they should be interpreted as the total MIR change observed across the ADCE pass rather than as direct counts of Phase 1 rewrites.

## Development Notes

Several safety and correctness issues were encountered during development, including interactions between MIR dataflow reachability and the custom post-dominator graph. These are documented in:

- [`docs/design.md`](docs/design.md)
- [`docs/bugs-found.md`](docs/bugs-found.md)

## Project Status

The current implementation is a conservative experimental ADCE pass.

Future work could extend value-level dead-code elimination beyond simple `Rvalue::Use` assignments and improve control-dependence analysis while preserving Rust-specific safety requirements.
