# OpenTelemetry Evaluation

## Setup

The ADCE pass was evaluated on the `opentelemetry-rust` repository using the stage1 compiler from the Rust compiler development tree.

The crate was compiled with the ADCE pass enabled and disabled under the same MIR optimization configuration. MIR before/after measurements were then compared.

## Results

The evaluation covered **619 functions**.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Functions | 619 | 619 | — |
| Basic blocks | 3,744 | 3,203 | 541 (14.45%) |
| Statements | 41,381 | 40,388 | 993 (2.40%) |

**75 functions** were changed during the ADCE pass.

## Interpretation

The results show that ADCE can substantially reduce MIR size on a real-world Rust crate.

The largest reduction is in basic blocks, while the statement reduction is smaller. This is consistent with the pass eliminating dead control-flow structure as well as dead assignments.

The measurements represent the MIR state before and after the ADCE pass, including the `SimplifyCfg::Final` cleanup that follows Phase 1. Therefore, the aggregate block reduction should not be interpreted as the number of branches directly rewritten by Phase 1.

For OpenTelemetry, the direct Phase 1 instrumentation reported no control-dependence collapses. The observed MIR reduction therefore primarily reflects Phase 2 dead-value elimination and subsequent CFG cleanup.

## Safety Validation

The real-world evaluation was complemented by the synthetic regression corpus in `evaluation/corpus/`.

The corpus specifically checks conservative behavior around:

- calls and side effects;
- drops;
- unwind paths;
- borrowed locals;
- debuginfo;
- cyclic control flow;
- self-targeting branches.
