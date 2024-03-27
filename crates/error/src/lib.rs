use std::fmt::Display;

pub type EngineResult<T> = std::result::Result<T, EngineError>;

pub struct EngineError {
    error_id: &'static str,
    error_msg: String,
}

impl EngineError {
    pub fn new(error_id: &'static str, error_msg: String) -> Self {
        Self {
            error_id,
            error_msg,
        }
    }

    pub fn as_result<T>(self) -> EngineResult<T> {
        Err(self)
    }

    pub fn get_id(&self) -> &'static str {
        self.error_id
    }

    pub fn get_msg(&self) -> &str {
        &self.error_msg
    }
}

impl Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error ID: {} \t {}", self.error_id, self.error_msg)
    }
}

#[macro_export]
macro_rules! engine_error {
    ($id:expr, $($args:tt)*) => {
        $crate::EngineError::new($id, format!($($args)*))
    };
}
