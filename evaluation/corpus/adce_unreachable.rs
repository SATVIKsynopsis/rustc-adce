// EMIT_MIR adce_unreachable.main.AdcePass.diff

// CHECK: debug x => _1;
// CHECK-NOT: begin_panic
// CHECK-NOT: _14 = const 20_i32

fn main() {
    let x = 10;

    if false {
        let y = 20;
        panic!("{}", y);
    }

    println!("{}", x);
}