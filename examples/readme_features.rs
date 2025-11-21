use error_rail::validation::Validation;
use error_rail::{context, location, tag, ComposableError};

fn structured_context_example() {
    println!("--- Structured Context ---");
    let err = ComposableError::<&str, u32>::new("db error")
        .with_context(tag!("database"))
        .with_context(location!())
        .with_context(context!("failed to connect"));

    println!("Error: {:?}", err);
}

fn validation_accumulation_example() {
    println!("\n--- Validation Accumulation ---");
    let v1 = Validation::<&str, i32>::valid(10);
    let v2 = Validation::<&str, i32>::invalid("too small");
    let combined: Validation<&str, Vec<i32>> = vec![v1, v2].into_iter().collect();

    assert!(combined.is_invalid());
    println!("Combined validation is invalid: {}", combined.is_invalid());
    if let Validation::Invalid(errors) = combined {
        println!("Errors: {:?}", errors);
    }
}

fn main() {
    structured_context_example();
    validation_accumulation_example();
}
