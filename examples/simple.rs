use sqs_listener::{SQSListener, SQSListenerClientBuilder};
use std::env;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;

    let queue_url = env::var("QUEUE_URL").unwrap_or("".to_string());

    let region = env::var("REGION").ok();

    let listener = SQSListener::new(queue_url, |message| {
        println!("Message received {:#?}", message)
    });
    let client = SQSListenerClientBuilder::new(region)
        .listener(listener)
        .build()?;
    let _ = client.start().await;
    Ok(())
}
