#![feature(custom_mir, core_intrinsics)]

use std::intrinsics::mir::*;

#[custom_mir(dialect = "runtime")]
pub fn multiple_dead_stores(x: i32) -> i32 {
    mir! {
        let a: i32;
        let b: i32;
        let c: i32;
        {
            a = x;
            b = a;
            c = b;
            RET = x;
            Return()
        }
    }
}

fn main() {
    let _ = multiple_dead_stores(42);
}
