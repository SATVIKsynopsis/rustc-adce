#![feature(custom_mir, core_intrinsics)]
#![crate_type = "lib"]

use std::intrinsics::mir::*;

#[custom_mir(dialect = "runtime")]
fn target_self(val: i32) {
    mir! {
        {
            Goto(bb1)
        }
        bb1 = {
            match val {
                0 => bb2,
                _ => bb1,
            }
        }
        bb2 = {
            match val {
                0 => bb3,
                _ => bb1,
            }
        }
        bb3 = {
            Return()
        }
    }
}

fn main() {}
