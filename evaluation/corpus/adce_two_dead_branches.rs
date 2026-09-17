#![feature(custom_mir, core_intrinsics)]

use std::intrinsics::mir::*;

#[custom_mir(dialect = "runtime")]
pub fn two_dead_branches(x: u8, y: u8) {
    mir! {
        {
            match x {
                1 => bb1,
                _ => bb2,
            }
        }
        bb1 = {
            Goto(bb3)
        }
        bb2 = {
            Goto(bb3)
        }
        bb3 = {
            match y {
                1 => bb4,
                _ => bb5,
            }
        }
        bb4 = {
            Goto(bb6)
        }
        bb5 = {
            Goto(bb6)
        }
        bb6 = {
            Return()
        }
    }
}

pub fn main() {
    two_dead_branches(1, 1);
}
