fn main() {
    let mut x = 0b0000_1000;
    for i in 0..10 {
        x |= 1 << i;
        println!("x = {x:#x}");
    }
}
