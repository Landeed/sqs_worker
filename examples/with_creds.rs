use sqs_listener::{EnvironmentVariableCredentialsProvider, SQSListener, SQSListenerClientBuilder};
use std::env;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;

    let queue_url = env::var("QUEUE_URL").expect("QUEUE_URL env variable needs to be present");

    let region = env::var("REGION").ok();
    // move region provider

    let credentials_provider = EnvironmentVariableCredentialsProvider::new();

    let listener = SQSListener::new(queue_url, |message| {
        println!("Message received {:#?}", message.body())
    });

    let client = SQSListenerClientBuilder::new_with(region, credentials_provider)
        .listener(listener)
        .build()?;
    let _ = client.start().await;
    Ok(())
}
