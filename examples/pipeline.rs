use error_rail::{context, group, ErrorPipeline};

fn do_http_call() -> Result<&'static str, &'static str> {
    Err("timeout")
}

fn parse_payload(_payload: &str) -> Result<&'static str, &'static str> {
    Err("invalid json")
}

fn main() {
    let result = ErrorPipeline::new(do_http_call())
        .with_context(context!("calling upstream service"))
        .with_context(group!(location(file!(), line!()), tag("http")))
        .and_then(parse_payload)
        .finish_boxed();

    match result {
        Ok(body) => println!("success: {body}"),
        Err(err) => {
            eprintln!("error chain => {}", err.error_chain());
        },
    }
}
