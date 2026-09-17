#![feature(custom_mir, core_intrinsics)]

use std::intrinsics::mir::*;

struct Droppable;

impl Drop for Droppable {
    fn drop(&mut self) {}
}

#[custom_mir(dialect = "runtime")]
pub fn dead_branch_drop(x: u8, value: Droppable) {
    mir! {
        {
            match x {
                1 => bb1,
                _ => bb2,
            }
        }
        bb1 = {
            Drop(value, ReturnTo(bb3), UnwindUnreachable())
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
    dead_branch_drop(1, Droppable);
}
