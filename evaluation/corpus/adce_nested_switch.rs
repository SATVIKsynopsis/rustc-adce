#![feature(custom_mir, core_intrinsics)]

use std::intrinsics::mir::*;

#[custom_mir(dialect = "runtime")]
fn nested_switch(x: u8) -> u8 {
    mir! {
        {
            match x {
                0 => bb1,
                _ => bb2,
            }
        }
        bb1 = {
            match x {
                0 => bb3,
                _ => bb4,
            }
        }
        bb2 = {
            RET = 2;
            Return()
        }
        bb3 = {
            RET = 3;
            Return()
        }
        bb4 = {
            RET = 4;
            Return()
        }
    }
}

fn main() {
    nested_switch(0);
}
