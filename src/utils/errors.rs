use std::error;

#[derive(Debug)]
pub enum CustomError {
    MongoError(mongodb::error::Error),
    CustomError(String),
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomError::MongoError(err) => write!(f, "{:?}", err),
            CustomError::CustomError(err) => write!(f, "{err}"),
        }
    }
}

impl error::Error for CustomError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            CustomError::MongoError(err) => Some(err),
            CustomError::CustomError(_) => None,
        }
    }
}

impl From<mongodb::error::Error> for CustomError {
    fn from(value: mongodb::error::Error) -> Self {
        Self::MongoError(value)
    }
}

impl From<String> for CustomError {
    fn from(value: String) -> Self {
        Self::CustomError(value)
    }
}

impl From<&str> for CustomError {
    fn from(value: &str) -> Self {
        Self::CustomError(value.to_string())
    }
}
