#![feature(custom_mir, core_intrinsics)]
#![crate_type = "lib"]

use std::intrinsics::mir::*;

#[inline(never)]
fn side_effect() {}

#[custom_mir(dialect = "runtime")]
fn call_side_effect() {
    mir! {
        {
            Call(RET = side_effect(), ReturnTo(bb1), UnwindContinue())
        }
        bb1 = {
            Return()
        }
    }
}

fn main() {}
