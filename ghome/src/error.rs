use actix_web::http::StatusCode;
use actix_web::ResponseError;
use thiserror::Error;

pub type WebResult<T> = Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("{0}")]
    Mysql(#[from] mysql::Error),
    #[error(r#"{{"error":"invalid_grant"}}"#)]
    InvalidGrant,
    #[error("{0}")]
    Refinery(#[from] refinery::error::Error),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Bad Request")]
    BadRequest,
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Mysql(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidGrant => StatusCode::BAD_REQUEST,
            Self::Refinery(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::SerdeJson(_) => StatusCode::BAD_REQUEST,
            Self::BadRequest => StatusCode::BAD_REQUEST,
        }
    }
}
