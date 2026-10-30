fn main() {
    let a = [10, 20, 30, 40, 50];

    // Iterating with an immutable reference
    for element in &a {
        println!("the value is: {element}");
    }
}