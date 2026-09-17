#![feature(custom_mir, core_intrinsics)]
use std::intrinsics::mir::*;

#[custom_mir(dialect = "runtime")]
fn dont_opt(x: Vec<bool>) {
    mir! {
        { Drop(x, ReturnTo(bb1), UnwindUnreachable()) }
        bb1 = { Return() }
    }
}

#[custom_mir(dialect = "runtime")]
fn cannot_opt_generic<T>(x: T) {
    mir! {
        { Drop(x, ReturnTo(bb1), UnwindUnreachable()) }
        bb1 = { Return() }
    }
}

fn main() {
    dont_opt(vec![true]);
    cannot_opt_generic(42);
}
