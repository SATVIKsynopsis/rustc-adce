# Evaluation

ADCE was evaluated using three complementary sources:

1. a synthetic regression corpus,
2. OpenTelemetry Rust, and
3. the Rust `regex` crate.

The goal is not only to measure how much MIR is removed, but also to demonstrate
that the pass remains conservative around side effects, drops, borrows, unwind
paths, and control flow.

## 1. Synthetic regression corpus

The evaluation corpus contains 21 ADCE-focused test cases under
`evaluation/corpus/`.

The tests cover both positive and negative cases, including:

- dead local assignments,
- multiple dead stores,
- transitive dead stores,
- dead reference chains,
- dead projections,
- dead branches,
- nested and sequential branches,
- multi-arm switches,
- self-targeting control flow,
- calls with potentially observable side effects,
- borrowed locals,
- drop and unwind behavior.

The positive cases verify that genuinely dead MIR can be eliminated, while the
negative cases verify that ADCE does not remove operations whose effects must
remain observable.

The corpus also contains `post_dom_diamond.rs`, which specifically exercises
the post-dominator infrastructure used by the control-dependence phase.

## 2. OpenTelemetry Rust

ADCE was evaluated against the OpenTelemetry Rust codebase using the stage1
compiler with MIR optimization level 4.

Across 619 functions:

- 75 functions were changed during the ADCE pass.
- Basic blocks decreased from 3,744 to 3,203.
- 541 basic blocks were removed, a 14.45% reduction.
- Statements decreased from 41,381 to 40,388.
- 993 statements were removed, a 2.40% reduction.

These aggregate `before`/`after` counts include the `SimplifyCfg::Final` cleanup
that follows ADCE Phase 1. Therefore, the 541-block reduction should not be
interpreted as 541 direct Phase 1 branch collapses.

The detailed OpenTelemetry results are documented in
`results/opentelemetry.md`.

## 3. Rust regex

ADCE was also evaluated against the Rust `regex` workspace.

Across 410 functions:

- 10 functions were changed.
- Basic blocks decreased from 1,748 to 1,728.
- 20 basic blocks were removed, a 1.14% reduction.
- Statements decreased from 22,754 to 22,730.
- 24 statements were removed, a 0.11% reduction.

Direct instrumentation of ADCE Phase 1 reported 7 control-dependence
collapses during compilation of the `regex` crate.

The aggregate MIR differences again include the `SimplifyCfg::Final` cleanup
following Phase 1, so the aggregate block reduction is not reported as a
direct Phase 1 collapse count.

The detailed regex results are documented in `results/regex.md`.

## 4. Interpreting the results

The synthetic corpus provides targeted correctness and regression coverage,
while the two real-world evaluations show how the pass behaves on substantially
larger MIR bodies.

The results also illustrate the intentionally conservative design of the pass.
ADCE is not currently attempting to remove every theoretically dead statement.
Instead, it focuses on transformations for which the implementation can
establish sufficient safety:

- control-dependent branches are collapsed only under a restrictive
  post-dominator-based condition;
- borrowed and debug-info-sensitive locals are protected;
- Phase 2 currently targets simple local `Rvalue::Use` assignments;
- calls, drops, and unwind-sensitive control flow are preserved;
- CFG cleanup is delegated to the existing `SimplifyCfg` pass.

This makes the evaluation useful both as a measure of optimization and as a
record of the safety boundary currently implemented by the pass.