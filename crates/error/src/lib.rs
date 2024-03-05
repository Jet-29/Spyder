use std::fmt::{Display, Formatter};

pub mod prelude {
    pub use super::Error;
    pub use super::Result;

}

pub type Result<T> = std::result::Result<T, Box<dyn Error + 'static>>;

pub trait Error {
    fn message(&self) -> &str;
}

impl Display for Box<dyn Error + 'static> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}