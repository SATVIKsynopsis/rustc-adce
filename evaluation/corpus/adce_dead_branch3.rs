#![feature(core_intrinsics, custom_mir)]
use std::intrinsics::mir::*;

#[inline(never)]
fn cond() -> bool {
    false
}

//@ test-mir-pass: AdcePass

// EMIT_MIR adce_dead_branch3.dead_branch3.AdcePass.before.mir
// EMIT_MIR adce_dead_branch3.dead_branch3.AdcePass.after.mir

#[custom_mir(dialect = "runtime", phase = "post-cleanup")]
fn dead_branch3() {
    // CHECK-LABEL: fn dead_branch3(
    // CHECK: bb0:
    // CHECK: _1 = cond()
    // CHECK: bb1:
    // CHECK: return

    mir! {
        let condition: bool;

        {
            Call(condition = cond(), ReturnTo(bb1), UnwindContinue())
        }

        bb1 = {
            match condition {
                true => bb2,
                _ => bb3,
            }
        }

        bb2 = {
            Goto(bb3)
        }

        bb3 = {
            Return()
        }
    }
}

fn main() {
    dead_branch3();
}