fn main() {
    let mut counter = 0;

    let result = loop {
        counter += 1;

        if counter == 10 {
            break counter * 2; // Stops the loop and returns a value
        }
    };

    println!("The result is {result}"); // Prints 20
}
