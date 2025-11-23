use error_rail::traits::ErrorOps;

#[test]
fn test_error_ops_recover_ok() {
    let result: Result<i32, &str> = Ok(42);
    let recovered = result.recover(|_| Ok(0));
    assert_eq!(recovered, Ok(42));
}
