#![feature(custom_mir, core_intrinsics)]

use std::intrinsics::mir::*;

#[custom_mir(dialect = "runtime")]
pub fn dead_first(x: &i32, y: &i32) -> i32 {
    mir! {
        let a: &i32;
        {
            a = x;
            a = y;
            RET = *a;
            Return()
        }
    }
}

pub fn main() {
    let x = 10;
    let y = 20;
    let _ = dead_first(&x, &y);
}
