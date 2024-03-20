use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use aws_sdk_sns::error::SdkError;
use aws_sdk_sns::operation::publish::PublishError;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Error while publishing message: {0}")]
    PublishMessage(#[from] SdkError<PublishError>),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
