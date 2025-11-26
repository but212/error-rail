use error_rail::ComposableError;
use core::error::Error;

fn main() {
    let err = ComposableError::new("core error")
        .with_context("ctx1")
        .with_context("ctx2")
        .set_code(500);

    println!("Display: {}", err);
    println!("Alternate Display:\n{:#}", err);

    // Check if it implements core::error::Error
    // Note: String/&str errors won't implement Error due to coherence rules,
    // but wrapping a real error (like io::Error) should work.
    let io_err = ComposableError::<std::io::Error>::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        "io boom",
    ));
    print_error(&io_err);
}

fn print_error<E: Error>(e: &E) {
    println!("Is core::error::Error: yes");
    println!("Source: {:?}", e.source());
}
