use axum::{routing::get, Router, Json, extract::State, http::StatusCode, response::IntoResponse};
use aws_config::{Region};
use aws_sdk_sns::types::Tag;
use aws_credential_types::Credentials;
use shuttle_secrets::SecretStore;
use sqlx::PgPool;
use serde::Deserialize;
use std::collections::HashMap;

mod errors;
use errors::ApiError;

#[derive(Clone)]
pub struct AppState {
    sns: aws_sdk_sns::Client,
    db: PgPool,
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_secrets::Secrets] secrets: SecretStore,
    #[shuttle_shared_db::Postgres] db: PgPool,
) -> shuttle_axum::ShuttleAxum {

    let access_key_id = secrets.get("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID not set in Secrets.toml");
    let secret_access_key = secrets.get("AWS_SECRET_ACCESS_KEY").expect("AWS_ACCESS_KEY_ID not set in Secrets.toml");

        let creds = Credentials::from_keys(
               access_key_id,
               secret_access_key,
               None
            );

        let cfg = aws_config::from_env()
            .region(Region::new("eu-west-02"))
            .credentials_provider(creds)
            .load()
            .await;

       let sns = aws_sdk_sns::Client::new(&cfg);

    let state = AppState { sns, db };

    let router = Router::new().route("/", get(hello_world)).with_state(state);

    Ok(router.into())
}

#[derive(Deserialize)]
pub struct CreateTopicParams {
    name: String,
    tags: Option<Vec<(String, String)>>,
    attributes: Option<HashMap<String, String>>
}

async fn create_topic(
    State(state): State<AppState>,
    Json(json): Json<CreateTopicParams>
)-> Result<impl IntoResponse, ApiError> {
    let topic = state.sns.create_topic();

    let topic = topic.name(json.name);

    let tags = match json.tags {
        Some(tags) => {
            let res: Vec<Tag> = tags.into_iter().map(|(key, value)| {

            Tag::builder()
                .set_key(Some(key))
                .set_value(Some(value))
                .build().unwrap()

            }).collect();

            Some(res)

        },
        None => None
    };

    let topic = if tags.is_some() { topic.set_tags(tags) } else { topic };
    let topic = if json.attributes.is_some() { topic.set_attributes(json.attributes) } else { topic };
    let output = topic.send().await?;

    println!("Topic created: {output:?}");

    Ok(StatusCode::OK)
}

async fn delete_topic(
    State(state): State<AppState>,
    Json(json): Json<DeleteItemParams>
) -> Result<impl IntoResponse, ApiError> {
    let _ = state.sns.delete_topic()
        .topic_arn(json.arn)
        .send().await?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct EmailSubscriptionParams {
    email: String,
    arn: String,
    attributes: Option<HashMap<String, String>>
}

#[derive(Deserialize)]
pub struct DeleteItemParams {
    arn: String
}

async fn subscribe_to_topic(
    State(state): State<AppState>,
    Json(json): Json<EmailSubscriptionParams>
) -> Result<impl IntoResponse, ApiError> {
    let sub = state.sns.subscribe()
        .protocol("email".to_string())
        .endpoint(json.email)
        .topic_arn(json.arn);

    let sub = if json.attributes.is_some() { sub.set_attributes(json.attributes) } else { sub };

    let output = sub.send().await?;
    println!("New subscriber created: {output:?}");

    Ok(StatusCode::OK)
}

async fn unsubscribe_from_topic(
    State(state): State<AppState>,
    Json(json): Json<DeleteItemParams>
) -> Result<impl IntoResponse, ApiError> {
    let _ = state.sns.unsubscribe()
        .subscription_arn(json.arn)
        .send().await?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
struct PublishMessageParams {
    arn: String,
    message: String,
    subject: String
}

async fn publish_message(
    State(state): State<AppState>,
    Json(json): Json<PublishMessageParams>
) -> Result<impl IntoResponse, ApiError> {
    let res = state.sns.publish()
        .topic_arn(json.arn)
        .message(json.message)
        .subject(json.subject)
        .send().await?;

    Ok(StatusCode::OK)
}
