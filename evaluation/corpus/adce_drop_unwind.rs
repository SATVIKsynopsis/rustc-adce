// EMIT_MIR adce_drop_unwind.main.AdcePass.diff

// CHECK: debug x =>
// CHECK: drop(_13)

struct Foo;

impl Drop for Foo {
    fn drop(&mut self) {}
}

fn might_panic() {
    panic!("boom");
}

fn main() {
    let x = Foo;

    might_panic();

    println!("done");
}