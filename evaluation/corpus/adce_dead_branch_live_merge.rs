#![feature(custom_mir, core_intrinsics)]

use std::intrinsics::mir::*;

#[custom_mir(dialect = "runtime")]
pub fn dead_branch_live_merge(x: u8) -> i32 {
    mir! {
        let value: i32;
        {
            match x {
                1 => bb1,
                _ => bb2,
            }
        }
        bb1 = {
            value = 10;
            Goto(bb3)
        }
        bb2 = {
            value = 20;
            Goto(bb3)
        }
        bb3 = {
            RET = value;
            Return()
        }
    }
}

pub fn main() {
    let _ = dead_branch_live_merge(1);
}