use crate::translators::api::baidu::BaiduApiError;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum Error {
    Text(TextError),
    Fetch(String),
    BaiduError(String),
    MissingToken(String),
}

#[derive(Clone, Debug)]
pub struct TextError {
    pub message: String,
    pub error: Option<String>,
}

impl Error {
    pub fn new(message: impl ToString, error: impl ToString) -> Self {
        Error::Text(TextError {
            message: message.to_string(),
            error: Some(error.to_string()),
        })
    }

    pub fn new_option(message: impl ToString) -> Self {
        Error::Text(TextError {
            message: message.to_string(),
            error: None,
        })
    }

    pub fn fetch(error: reqwest::Error) -> Self {
        Error::Fetch(error.to_string())
    }

    pub fn baidu_error(message: BaiduApiError) -> Self {
        Self::BaiduError(format!(
            "Baidu error: Code: {}, Message: {}, Solution: {}, Data: {:?}",
            message.code,
            message.msg,
            message.solution(),
            message.data
        ))
    }

    pub fn missing_token(token: impl ToString) -> Self {
        Error::MissingToken(format!(
            "Missing token: {}. Add token to .env",
            token.to_string()
        ))
    }
}
