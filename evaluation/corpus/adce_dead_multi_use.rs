#![feature(custom_mir, core_intrinsics)]

use std::intrinsics::mir::*;

#[custom_mir(dialect = "runtime")]
pub fn dead_multi_use(x: i32) -> i32 {
    mir! {
        let a: i32;
        let b: i32;
        let c: i32;
        {
            a = x;
            b = a;
            c = a;
            RET = x;
            Return()
        }
    }
}

pub fn main() {
    let _ = dead_multi_use(42);
}
