use error::prelude::*;

struct MyError;

impl Error for MyError {
    fn message(&self) -> &str {
        "This is my error"
    }
}

fn return_error() -> Result<()> {
    Err(Box::new(MyError))
}

fn return_ok() -> Result<u32> {
    Ok(7)
}

fn main() {
    let error = return_error().err().unwrap();
    match error {
        MyError => todo!(),
    }

    match return_error() {
        Ok(_) => println!("No error"),
        Err(e) => println!("Error: {}", e),
    }

    match return_ok() {
        Ok(val) => println!("Ok: {}", val),
        Err(e) => println!("Error: {}", e),
    }
}