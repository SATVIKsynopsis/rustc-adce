# Regex Evaluation

## Setup

The ADCE pass was evaluated on the Rust `regex` workspace using the stage1 compiler from
the Rust compiler development tree.

The crate was compiled with the ADCE pass enabled and disabled under the same MIR
optimization configuration. MIR before/after measurements were then compared.

## Results

The evaluation covered 410 functions.

| Metric | Before | After | Reduction |
|---|---|---|---|
| Functions | 410 | 410 | — |
| Basic blocks | 1,748 | 1,728 | 20 (1.1%) |
| Statements | 22,754 | 22,730 | 24 (0.1%) |

10 functions were changed during the ADCE pass.

The Phase 1 instrumentation independently reported **7 control-dependence collapses**
during compilation.

## Example transformation

One observed transformation occurred in branch-heavy code where a `switchInt` had two
different arms targeting the same block:

```
switchInt(move _120) -> [1: bb51, 0: bb33, otherwise: bb6];
```

became:

```
switchInt(move _120) -> [1: bb33, 0: bb33, otherwise: bb6];
```

Both arms now go to the same target, `bb33`. This is consistent with a Phase 1
collapse because the branch result no longer affects how the function continues.

I describe this as **“consistent with”** rather than **“directly caused by”** a
specific Phase 1 rewrite because `SimplifyCfg::Final` runs immediately after Phase 1
in the same pass and can also merge these kinds of branches.

The 7 collapse count comes from instrumentation inside Phase 1's own decision logic,
not from reading this MIR diff. Therefore, 7 is the authoritative number for Phase 1's
specific contribution.

## Binary size impact

I compared the final linked `regex-cli` binary with ADCE enabled and disabled.

| Build | Size |
|---|---:|
| ADCE enabled | 4,892,856 bytes |
| ADCE disabled | 4,892,792 bytes |
| Difference | 64 bytes larger with ADCE enabled |

The difference is about 0.0013% on a roughly 4.9 MB executable, which is effectively
no change and is within normal build to build variation.

This is not surprising because LLVM performs its own dead code elimination after MIR
lowering. Because of this, LLVM may independently remove much of the code that ADCE
removes earlier.

The benefit of running ADCE at the MIR level is therefore not a smaller final binary.
Instead, it means there is less MIR for later optimization passes, including LLVM, to
process.

## Interpretation

`regex` produced 7 Phase 1 branch collapses across multiple compilation units. This is
consistent with the pass finding redundant control flow in branch heavy,
state machine style and table-driven code.

This differs from `opentelemetry-rust` where the same instrumentation reported 0
Phase 1 collapses on more typical library code. See `results/opentelemetry.md` for the
contrasting case.

The block and statement reduction numbers represent the full pass: Phase 1 branch
collapsing, Phase 2 dead assignment removal, and the `SimplifyCfg::Final` cleanup that
runs afterward.

Therefore, the aggregate 20 block reduction should not be treated as 20 Phase 1
rewrites.

The 7 collapse figure was measured independently through instrumentation inside
`branch_is_dead` so it is the number used to represent Phase 1's specific
contribution.