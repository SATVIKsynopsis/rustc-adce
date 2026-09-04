// EMIT_MIR adce_dead_branch.dead_branch.AdcePass.after.mir

// CHECK-LABEL: fn dead_branch
// CHECK: switchInt
// CHECK: _0 = const 5_i32;
// CHECK: return;

fn dead_branch(flag: bool) -> i32 {
    let x;

    if flag {
        x = 10;
    } else {
        x = 20;
    }

    5
}

fn main() {
    dead_branch(true);
}