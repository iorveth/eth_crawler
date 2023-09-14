use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use core::num::ParseIntError;
use derive_more::{Display, Error};
use migration::DbErr;

#[derive(Debug, Display, Error)]
pub enum ServerError {
    #[display(fmt = "Invalid address: {}", address)]
    InvalidAddress { address: String },
    #[display(fmt = "Reqwest error: {}", reqwest_error)]
    ReqwestError { reqwest_error: reqwest::Error },
    #[display(fmt = "Reqwest parsing error happened")]
    ReqwestParsingError,
    #[display(
        fmt = "Starting block number {} is greater than current block number {}",
        starting_block_number,
        current_block_number
    )]
    InvalidStartingBlockNumber {
        starting_block_number: u64,
        current_block_number: u64,
    },
    #[display(fmt = "Database error: {}", db_err)]
    DbErr { db_err: DbErr },
    #[display(fmt = "Parsing error: {}", parse_int_error)]
    ParseIntError { parse_int_error: ParseIntError },
}

impl From<reqwest::Error> for ServerError {
    fn from(reqwest_error: reqwest::Error) -> Self {
        Self::ReqwestError { reqwest_error }
    }
}

impl From<DbErr> for ServerError {
    fn from(db_err: DbErr) -> Self {
        Self::DbErr { db_err }
    }
}

impl From<ParseIntError> for ServerError {
    fn from(parse_int_error: ParseIntError) -> Self {
        Self::ParseIntError { parse_int_error }
    }
}

impl error::ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::InvalidAddress { .. } => StatusCode::BAD_REQUEST,
            Self::InvalidStartingBlockNumber { .. } => StatusCode::BAD_REQUEST,
            Self::ReqwestError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ReqwestParsingError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DbErr { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ParseIntError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
