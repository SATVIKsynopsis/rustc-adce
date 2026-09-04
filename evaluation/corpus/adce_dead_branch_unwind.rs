#![feature(core_intrinsics, custom_mir)]
use std::intrinsics::mir::*;

#[inline(never)]
fn cond() -> bool {
    false
}

struct Droppable;

impl Drop for Droppable {
    fn drop(&mut self) {}
}

#[inline(never)]
fn make_droppable() -> Droppable {
    Droppable
}

//@test-mir-pass: AdcePass

// EMIT_MIR adce_dead_branch_unwind.dead_branch_unwind.AdcePass.before.mir
// EMIT_MIR adce_dead_branch_unwind.dead_branch_unwind.AdcePass.after.mir

#[custom_mir(dialect = "runtime", phase = "post-cleanup")]
fn dead_branch_unwind() {
    // CHECK-LABEL: fn dead_branch_unwind(

    mir! {
        let condition: bool;
        let value: Droppable;

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
            Call(value = make_droppable(), ReturnTo(bb3), UnwindCleanup(bb4))
        }

        bb3 = {
            Return()
        }

        bb4 (cleanup) = {
            Drop(value, ReturnTo(bb5), UnwindTerminate(ReasonInCleanup))
        }

        bb5 (cleanup) = {
            UnwindResume()
        }
    }
}

fn main() {
    dead_branch_unwind();
}