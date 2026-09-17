#![feature(custom_mir, core_intrinsics)]

use std::intrinsics::mir::*;

#[inline(never)]
fn side_effect() {}

#[custom_mir(dialect = "runtime")]
pub fn dead_branch_call(x: u8) {
    mir! {
        {
            match x {
                1 => bb1,
                _ => bb2,
            }
        }
        bb1 = {
            Call(RET = side_effect(), ReturnTo(bb3), UnwindContinue())
        }
        bb2 = {
            Goto(bb3)
        }
        bb3 = {
            Return()
        }
    }
}

pub fn main() {
    dead_branch_call(1);
}
