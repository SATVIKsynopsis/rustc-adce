Rust MIR Aggressive Dead Code Elimination (ADCE)

rustc's MIR pipeline already has passes that clean up dead code — SimplifyLocals, SimplifyCfg, DeadStoreElimination, but none of them reason about control dependence. LLVM has had this for a long time as ADCE: if neither branch of an if actually does anything observable, the whole branch can go, not just the dead values inside it. rustc had no equivalent, so this project builds one.

What it does

The pass has two parts:

Control flow deadness (Phase 1). Builds a post-dominator graph over the MIR CFG, uses it to find which blocks are control dependent on a given branch, and collapses the branch to a plain Goto if none of those blocks do anything observable.
Value deadness (Phase 2). Reuses rustc's own MaybeTransitiveLiveLocals analysis (the same one DeadStoreElimination uses) to find dead assignments and turn them into no-ops.

It's deliberately conservative. It never touches a branch that contains a Call, Drop, Assert or anything else with a real effect and it respects the same safety rules as rustc's existing passes around borrowed locals and debuginfo.

Where the code lives
implementation/adce.rs
implementation/post_dom.rs

These are copies for reference.

The actual working version lives in my [rustc fork](https://github.com/SATVIKsynopsis/rust/tree/rustc_adce), at commit [`6e57dd2643a`](https://github.com/SATVIKsynopsis/rust/commit/6e57dd2643a), under `compiler/rustc_mir_transform/src/adce.rs` and `post_dom.rs`.

Building and testing it

You'll need a local rustc checkout with these two files added and AdcePass registered in compiler/rustc_mir_transform/src/lib.rs's pass list.

# build a stage1 compiler
./x.py build --stage 1

# run the ADCE-specific tests
./x.py test tests/mir-opt/adce_*.rs

# run the full mir-opt suite
./x.py test tests/mir-opt

The test corpus in evaluation/corpus/ can be copied into tests/mir-opt/ in your own checkout to run the same way.

Tests

evaluation/corpus/ has 22 test cases written during development, split roughly evenly between cases where the pass should change the MIR and cases where it should correctly leave it alone. They cover dead assignments, borrowed locals, debuginfo sensitive locals, dead branches, unwind paths, drops, unreachable/diverging control flow, transitive dead stores, and a couple of pathological CFG shapes (a branch that targets itself, and so on). post_dom_diamond.rs specifically exercises the post-dominator construction on its own, separate from the rest of the pass.

Evaluation

Full methodology and results are in docs/evaluation.md. Short version:

opentelemetry-rust (619 functions, mostly straight-line library code):

Basic blocks: 3,744 -> 3,203 (-14.5%)
Statements: 41,381 -> 40,388 (-2.4%)
Phase 1 control dependence collapses: 0

regex (410 functions, branch-heavy/state-machine code):

Basic blocks: 1,748 -> 1,728 (-1.1%)
Statements: 22,754 -> 22,730 (-0.1%)
Phase 1 control dependence collapses: 7

Those block/statement numbers are the whole pass Phase 1, SimplifyCfg cleanup, and Phase 2 combined and not Phase 1 alone. I instrumented the pass to check why it doesn't fire on opentelemetry-rust specifically: every candidate branch had a Call, Drop, or similar real effect in it, which the pass correctly refuses to touch. Branch dense, state machine style code (regex) is where this kind of redundant control flow actually shows up.

I also compared the final linked binary size for regex-cli with the pass on and off and the difference was 64 bytes on a ~4.9 MB binary, i.e. no real change. That's not surprising: LLVM does its own dead code elimination after MIR lowering, so a lot of what this pass catches earlier, LLVM would likely have caught anyway. The point of doing it at the MIR level isn't a smaller final binary — it's less IR for every later pass to chew through.

What went wrong along the way

Building this surfaced four real bugs, two of which were genuine correctness problems, not just rough edges:

An early version could route a branch collapse into a cleanup (unwind-landing-pad) block using a plain Goto, which rustc doesn't allow — this crashed the compiler on a real coroutine test.
Phase 2 originally queried liveness at the wrong point while walking statements backwards, which could make a value look dead right after its own definition even though the next line still used it which is a real, silent miscompile, caught by an existing rustc regression test named after a historical NRVO bug.
An early version of the post-dominator graph only counted a few terminator kinds as real exits, which crashed on any function with a non-terminating loop.
When I made Phase 1 liveness-aware, the new version should only ever have found more collapses than the old one, instead it found fewer on regex (2 instead of 7) which turned out to be a genuine logic bug in how storage markers were being checked.

Full write-ups:
- [Design](docs/design.md)
- [Bugs found and fixes](docs/bugs-found.md)
- [Evaluation methodology](docs/evaluation.md)
- [Evaluation corpus](evaluation/corpus/)

What this doesn't do
It doesn't try to reason about loops that never terminate — post-dominance isn't well defined there, so the pass just leaves them alone. LLVM has a separate, opt-in flag for this (-adce-remove-loops); i didn't attempt it.
Post-dominators are computed once per pass run, before any branch gets collapsed. I tested functions with multiple independent dead branches and that works fine, but i didn't test nested or overlapping cases where collapsing one branch could change the post dominance picture for another.
I looked at a second LLVM technique, BDCE (bit-tracking dead code elimination - only removing the parts of a computation whose bits are never actually observed) and tried a small widen then narrow cast case. It didn't fire without more copy chain analysis than i had time to get right, so it's not in the final pass and just written up as a possible next step.
Status

This is an experimental pass, not something ready to upstream as is. The conservative choices (only plain Rvalue::Use assignments in Phase 2, only Goto terminated blocks in Phase 1) were deliberate, the goal was a correct, well tested implementation of the core idea, not maximum coverage on day one.