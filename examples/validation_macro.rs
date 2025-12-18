//! Validation with validate! macro (New in 0.7.0)
//!
//! This example demonstrates the new validate! macro for cleaner validation syntax.
//! Run with: cargo run --example validation_macro

use error_rail::{validate, validation::Validation};

fn validate_username(name: &str) -> Validation<&'static str, String> {
    if name.len() >= 3 && name.len() <= 20 {
        Validation::Valid(name.to_string())
    } else {
        Validation::invalid("username must be 3-20 characters")
    }
}

fn validate_email(email: &str) -> Validation<&'static str, String> {
    if email.contains('@') && email.contains('.') {
        Validation::Valid(email.to_string())
    } else {
        Validation::invalid("email must contain @ and .")
    }
}

fn validate_age(age: i32) -> Validation<&'static str, i32> {
    if age >= 0 && age <= 150 {
        Validation::Valid(age)
    } else {
        Validation::invalid("age must be between 0 and 150")
    }
}

fn main() {
    println!("=== Validation with validate! macro ===\n");

    // Example 1: All valid
    println!("Example 1: All valid inputs");
    let username = validate_username("john_doe");
    let email = validate_email("john@example.com");
    let age = validate_age(25);

    let result = validate!(username = username, email = email, age = age);

    match result {
        Validation::Valid((u, e, a)) => {
            println!("✓ All valid!");
            println!("  Username: {}", u);
            println!("  Email: {}", e);
            println!("  Age: {}", a);
        },
        Validation::Invalid(errors) => {
            println!("✗ Validation errors:");
            for err in errors.into_inner() {
                println!("  - {}", err);
            }
        },
    }

    // Example 2: Some invalid
    println!("\nExample 2: Some invalid inputs");
    let username = validate_username("ab"); // Too short
    let email = validate_email("invalid"); // No @ or .
    let age = validate_age(25); // Valid

    let result = validate!(username = username, email = email, age = age);

    match result {
        Validation::Valid((u, e, a)) => {
            println!("✓ All valid!");
            println!("  Username: {}", u);
            println!("  Email: {}", e);
            println!("  Age: {}", a);
        },
        Validation::Invalid(errors) => {
            println!("✗ Found {} validation errors:", errors.len());
            for err in errors.into_inner() {
                println!("  - {}", err);
            }
        },
    }

    // Example 3: All invalid
    println!("\nExample 3: All invalid inputs");
    let username = validate_username("x"); // Too short
    let email = validate_email("bad-email"); // No @
    let age = validate_age(200); // Too old

    let result = validate!(username = username, email = email, age = age);

    match result {
        Validation::Valid(_) => println!("✓ All valid!"),
        Validation::Invalid(errors) => {
            println!("✗ Found {} validation errors:", errors.len());
            for err in errors.into_inner() {
                println!("  - {}", err);
            }
        },
    }

    println!("\n✓ Example completed!");
}
