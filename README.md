# sqs_listener

![Build Status](https://github.com/avencera/sqs_listener/workflows/Rust/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/sqs_listener.svg)](https://crates.io/crates/sqs_worker)
[![Documentation](https://docs.rs/sqs_listener/badge.svg)](https://docs.rs/sqs_worker/0.1.0/sqs_worker/)
[![Rust 1.52+](https://img.shields.io/badge/rust-1.52+-orange.svg)](https://www.rust-lang.org)

## Getting Started

Available on crates: [crates.io/sqs_worker](https://crates.io/crates/sqs_worker)

Documentation available at: [docs.rs/sqs_worker](https://docs.rs/sqs_worker/0.1.0/sqs_worker/)

```toml
sqs_worker = "0.1.2"
```

### Simple Example

Simple example: [/examples/simple.rs](examples/simple.rs)

```rust
use sqs_worker::{EnvironmentVariableCredentialsProvider, SQSListener, SQSListenerClientBuilder};
use std::env;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;

    let queue_url = env::var("QUEUE_URL").expect("QUEUE_URL env variable needs to be present");

    let region = env::var("REGION").ok();

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
```

### Start a listener using AWS creds

Example with creds: [/examples/with_creds.rs](examples/with_creds.rs)

```rust
use sqs_worker::{EnvironmentVariableCredentialsProvider, SQSListener, SQSListenerClientBuilder};
use std::env;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;

    let queue_url = env::var("QUEUE_URL").expect("QUEUE_URL env variable needs to be present");

    let region = env::var("REGION").ok();

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
```
