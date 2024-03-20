use anyhow::Result as AnyhowResult;
use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_sns::Client;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Deserialize;
use shuttle_secrets::SecretStore;

mod errors;
use errors::ApiError;

#[derive(Clone)]
pub struct AppState {
    sns: aws_sdk_sns::Client,
    topic_arn: String,
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn main(#[shuttle_secrets::Secrets] secrets: SecretStore) -> shuttle_axum::ShuttleAxum {
    let access_key_id = secrets
        .get("AWS_ACCESS_KEY_ID")
        .expect("AWS_ACCESS_KEY_ID not set in Secrets.toml");
    let secret_access_key = secrets
        .get("AWS_SECRET_ACCESS_KEY")
        .expect("AWS_ACCESS_KEY_ID not set in Secrets.toml");

    let creds = Credentials::from_keys(access_key_id, secret_access_key, None);

    let cfg = aws_config::from_env()
        .region(Region::new("eu-west-02"))
        .credentials_provider(creds)
        .load()
        .await;

    let sns = aws_sdk_sns::Client::new(&cfg);
    let topic_arn = create_topic(&sns, "my_topic").await?;

    // NOTE: Change this to your deployment URL for your receiver service!
    // The SNS receiver route should be on `/sns` as instructed previously
    let url = "https://sns-receiver.shuttleapp.rs/sns";

    if !subscription_exists(&sns, url, &topic_arn).await? {
        subscribe_to_topic(&sns, url, &topic_arn).await?;
    }

    let state = AppState { sns, topic_arn };

    let router = Router::new().route("/", get(hello_world)).with_state(state);

    Ok(router.into())
}

async fn subscription_exists(sns: &Client, url: &str, arn: &str) -> AnyhowResult<bool> {
    let subscribers = sns
        .list_subscriptions()
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("error: {e}"))?;

    if let Some(subs) = subscribers.subscriptions {
        if subs.iter().any(|sub| {
            sub.clone().endpoint.unwrap() == *url && sub.clone().topic_arn.unwrap() == *arn
        }) {
            return Ok(true);
        }

        return Ok(false);
    }

    Ok(false)
}

async fn create_topic(sns: &Client, name: &str) -> AnyhowResult<String> {
    let topic = sns.create_topic().name(name);

    let output = topic.send().await.unwrap();

    println!("Topic created: {output:?}");
    Ok(output.topic_arn.unwrap())
}

async fn subscribe_to_topic(sns: &Client, url: &str, arn: &str) -> AnyhowResult<()> {
    let sub = sns
        .subscribe()
        .protocol("https".to_string())
        .endpoint(url)
        .topic_arn(arn);

    let output = sub
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("error: {e}"))?;
    println!("New subscriber created: {output:?}");

    Ok(())
}

#[derive(Deserialize)]
struct PublishMessageParams {
    message: String,
    subject: String,
}

async fn publish_message(
    State(state): State<AppState>,
    Json(json): Json<PublishMessageParams>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .sns
        .publish()
        .topic_arn(state.topic_arn)
        .message(json.message)
        .subject(json.subject)
        .send()
        .await?;

    Ok(StatusCode::OK)
}
