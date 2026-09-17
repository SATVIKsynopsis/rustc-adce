#![feature(custom_mir, core_intrinsics)]

use std::intrinsics::mir::*;

#[custom_mir(dialect = "runtime")]
pub fn cfg_dead_stores(x: u8) -> i32 {
    mir! {
        let a: i32;
        let b: i32;
        {
            match x {
                1 => bb1,
                _ => bb2,
            }
        }
        bb1 = {
            a = 10;
            b = a;
            Goto(bb3)
        }
        bb2 = {
            a = 20;
            b = a;
            Goto(bb3)
        }
        bb3 = {
            RET = 0;
            Return()
        }
    }
}

pub fn main() {
    let _ = cfg_dead_stores(1);
}
