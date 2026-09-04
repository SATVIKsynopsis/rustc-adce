# Regex Evaluation

## Setup

The ADCE pass was evaluated on the Rust `regex` workspace using the stage1 compiler from the Rust compiler development tree.

The crate was compiled with the ADCE pass enabled and disabled under the same MIR optimization configuration. MIR before/after measurements were then compared.

## Results

The evaluation covered **410 functions**.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Functions | 410 | 410 | — |
| Basic blocks | 1,748 | 1,728 | 20 (1.14%) |
| Statements | 22,754 | 22,730 | 24 (0.11%) |

**10 functions** were changed during the ADCE pass.

The Phase 1 instrumentation independently reported **7 control-dependence collapses** during compilation.

## Example Transformation

One observed transformation occurred in branch-heavy code where a `SwitchInt` had two different values targeting the same block.

Conceptually, a switch of the form:

```text
switchInt(value) -> [1: bb51, 0: bb33, otherwise: bb6]
