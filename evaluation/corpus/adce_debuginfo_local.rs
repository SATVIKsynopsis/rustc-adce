// EMIT_MIR adce_debuginfo_local.main.AdcePass.after.mir

// CHECK-LABEL: fn main
// CHECK: debug x => const 42_i32;
// CHECK: debug y => _1;

fn main() {
    let x = 42;
    let y = 20;

    println!("{}", y);
}