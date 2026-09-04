// CHECK: fn choose
// CHECK: bb0
// CHECK: bb1
// CHECK: bb2

fn choose(flag: bool) -> i32 {
    let x;

    if flag {
        x = 10;
    } else {
        x = 20;
    }

    x
}

fn main() {
    choose(true);
}