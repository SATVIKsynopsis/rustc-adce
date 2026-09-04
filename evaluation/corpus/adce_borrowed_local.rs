// EMIT_MIR adce_borrowed_local.main.AdcePass.diff
// CHECK: _1 = const 42_i32;
// CHECK: {{_.*}} = &_1;

fn main() {
    let x = 42;
    let r = &x;

    println!("{}", r);
}