#![feature(custom_mir, core_intrinsics)]

use std::intrinsics::mir::*;

#[custom_mir(dialect = "runtime")]
pub fn dead_projection(x: (i32, i32)) -> i32 {
    mir! {
        let a: (i32, i32);
        {
            a = x;
            RET = a.0;
            Return()
        }
    }
}

pub fn main() {
    let _ = dead_projection((10, 20));
}
