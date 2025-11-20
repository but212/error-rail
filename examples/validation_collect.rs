use error_rail::Validation;

fn validate_positive(n: i32) -> Validation<&'static str, i32> {
    if n > 0 {
        Validation::Valid(n)
    } else {
        Validation::invalid("must be positive")
    }
}

fn validate_even(n: i32) -> Validation<&'static str, i32> {
    if n % 2 == 0 {
        Validation::Valid(n)
    } else {
        Validation::invalid("must be even")
    }
}

fn validate_number(n: i32) -> Validation<&'static str, i32> {
    let positive = validate_positive(n);
    let even = validate_even(n);

    match (positive, even) {
        (Validation::Valid(_), Validation::Valid(_)) => Validation::Valid(n),
        (Validation::Invalid(mut a), Validation::Invalid(b)) => {
            a.extend(b);
            Validation::Invalid(a)
        }
        (Validation::Invalid(errors), _) | (_, Validation::Invalid(errors)) => {
            Validation::Invalid(errors)
        }
    }
}

fn main() {
    let input = [1, 2, -3, 4];

    let combined: Validation<&'static str, Vec<i32>> =
        input.into_iter().map(validate_number).collect();

    match combined {
        Validation::Valid(values) => println!("valid numbers: {:?}", values),
        Validation::Invalid(errors) => {
            for err in errors {
                println!("validation error: {err}");
            }
        }
    }
}
