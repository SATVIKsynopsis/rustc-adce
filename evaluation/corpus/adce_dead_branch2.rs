// EMIT_MIR adce_dead_branch2.dead_branch2.AdcePass.after.mir

// CHECK-LABEL: fn dead_branch2
// CHECK: bb0:
// CHECK-NOT: switchInt
// CHECK: _0 = const 5_i32;

fn dead_branch2(flag: bool) -> i32 {
    if flag {
        10;
    } else {
        20;
    }

    5
}

fn main() {
    dead_branch2(true);
}