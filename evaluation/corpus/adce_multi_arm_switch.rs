#![feature(custom_mir, core_intrinsics)]

use std::intrinsics::mir::*;

#[custom_mir(dialect = "runtime")]
fn multi_arm(x: u8) -> u8 {
    mir! {
        {
            match x {
                0 => bb1,
                1 => bb1,
                2 => bb2,
                _ => bb3,
            }
        }
        bb1 = {
            RET = 10;
            Return()
        }
        bb2 = {
            RET = 20;
            Return()
        }
        bb3 = {
            RET = 30;
            Return()
        }
    }
}

fn main() {
    multi_arm(0);
}
