// EMIT_MIR adce_dead_store.main.AdcePass.diff

// CHECK-LABEL: fn main
// CHECK-NOT: _1 = const 10_i32
// CHECK: _1 = const 20_i32

fn main() {
    let x = 10;
    let y = 20;

    println!("{}", y);
}