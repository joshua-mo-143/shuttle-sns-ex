use axum::{routing::{get, post}, Router, response::{IntoResponse, Response}, http::StatusCode, extract::{Request, FromRequest}, Json, RequestExt};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SnsMessage {
    #[serde(rename = "Type")]
    kind: String,
    message_id: String,
    topic_arn: String,
    subject: String,
    message: String,
    timestamp: String,
    signature_version: String,
    signature: String,
    signing_cert_url: String,
    unsubscribe_url: String
}

#[axum::async_trait]
impl<S> FromRequest<S> for SnsMessage {
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let headers = req.headers();
        
        if !headers.contains_key("x-amz-sns-message-type") 
            | !headers.contains_key("x-amz-message-id") 
            | !headers.contains_key("x-amz-topic-arn")
            | !headers.contains_key("x-amz-subscription-arn") {
            return Err((StatusCode::BAD_REQUEST).into_response())
        } 

        let Json(payload): axum::Json<SnsMessage> = req.extract().await.map_err(|_| (StatusCode::BAD_REQUEST).into_response())?;

        Ok(payload)
    }
}

async fn receive_sns(
        sns: SnsMessage
    ) -> StatusCode {
    println!("{}", sns.message);
    StatusCode::OK
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new().route("/", get(hello_world))
        .route("/notifications", post(receive_sns));

    Ok(router.into())
}
