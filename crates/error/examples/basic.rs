use error::*;

fn return_error() -> EngineResult<()> {
    EngineError::new("ErrorLess", "No error".to_string()).as_result()
}

fn return_ok() -> EngineResult<u32> {
    engine_error!("ErrorID", "An error occured here {}", 7).as_result()
}

fn main() {
    match return_error() {
        Ok(_) => println!("No error"),
        Err(e) => println!("Error: {}", e),
    };

    match return_ok() {
        Ok(val) => println!("Ok: {}", val),
        Err(e) => println!("Error: {}", e),
    }
}
