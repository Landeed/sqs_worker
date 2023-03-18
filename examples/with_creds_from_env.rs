use env_logger;
use eyre;
use sqs_listener::{SQSListener, SQSListenerClientBuilder};
use std::env;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();

    let queue_url = env::var("QUEUE_URL").expect("SQS_QUEUE_URL environment variable not set");

    let client = SQSListenerClientBuilder::new()
        .listener(SQSListener::new(queue_url, |message| {
            println!("Message received {:#?}", message)
        }))
        .build()?;

    let _ = client.start().await;

    Ok(())
}
