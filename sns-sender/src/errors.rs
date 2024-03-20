use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use aws_sdk_sns::error::SdkError;
use aws_sdk_sns::operation::create_topic::CreateTopicError;
use aws_sdk_sns::operation::delete_topic::DeleteTopicError;
use aws_sdk_sns::operation::publish::PublishError;
use aws_sdk_sns::operation::subscribe::SubscribeError;
use aws_sdk_sns::operation::unsubscribe::UnsubscribeError;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("SQL error: {0}")]
    SQL(#[from] sqlx::Error),
    #[error("Error while creating topic: {0}")]
    CreateTopic(#[from] SdkError<CreateTopicError>),
    #[error("Error while deleting topic: {0}")]
    DeleteTopic(#[from] SdkError<DeleteTopicError>),
    #[error("Error while subscribing to topic: {0}")]
    SubscribeToTopic(#[from] SdkError<SubscribeError>),
    #[error("Error while unsubscribing from topic: {0}")]
    UnsubscribeFromTopic(#[from] SdkError<UnsubscribeError>),
    #[error("Error while publishing message: {0}")]
    PublishMessage(#[from] SdkError<PublishError>),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let res = match self {
            Self::SQL(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::CreateTopic(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::DeleteTopic(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::SubscribeToTopic(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::UnsubscribeFromTopic(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::PublishMessage(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        res.into_response()
    }
}
