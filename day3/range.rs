fn main() {
    // (1..4) yields 1, 2, 3
    for number in (1..4).rev() { // .rev() reverses the range
        println!("{number}!");
    }
    println!("LIFTOFF!!!");
}