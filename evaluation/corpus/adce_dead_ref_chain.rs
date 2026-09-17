#![feature(custom_mir, core_intrinsics)]

use std::intrinsics::mir::*;

#[custom_mir(dialect = "runtime")]
pub fn dead_ref_chain(x: &i32, y: &i32) -> i32 {
    mir! {
        let a: &i32;
        let b: &i32;
        {
            a = x;
            b = a;
            a = y;
            RET = *b;
            Return()
        }
    }
}

pub fn main() {
    let x = 10;
    let y = 20;
    let _ = dead_ref_chain(&x, &y);
}
