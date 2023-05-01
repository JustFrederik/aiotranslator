#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Error {
    pub message: String,
    error: Option<String>,
}

impl Error {
    pub fn new(message: impl ToString, error: impl ToString) -> Self {
        Error {
            message: message.to_string(),
            error: Some(error.to_string()),
        }
    }

    pub fn new_option(message: impl ToString) -> Self {
        Error {
            message: message.to_string(),
            error: None,
        }
    }
}
