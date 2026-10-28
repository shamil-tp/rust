use std::io::{self, Write};

fn main() {
    print!("Enter age: ");
    // Flush stdout so the prompt prints before waiting for input
    io::stdout().flush().unwrap();

    let mut input = String::new();
    
    // Step 1: Read a line of text into `input`
    io::stdin().read_line(&mut input).expect("Failed to read line");

    // Step 2: Trim whitespace/newline and parse to integer (`i32`)
    let age: i32 = input.trim().parse().expect("Please enter a valid number!");

    println!("You are {} years old", age);
}