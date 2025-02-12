/*!
### Simple Example
Simple example: [examples/simple.rs](https://github.com/avencera/sqs_listener/blob/master/examples/simple.rs)
```rust,ignore
use sqs_listener::{Region, SQSListener, SQSListenerClientBuilder};
#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;
    let listener = SQSListener::new("".to_string(), |message| {
        println!("Message received {:#?}", message)
    });
    let client = SQSListenerClientBuilder::new(Region::UsEast1)
        .listener(listener)
        .build()?;
    let _ = client.start().await;
    Ok(())
}
```
### Start a listener using AWS creds
Example with creds: [examples/with_creds.rs](https://github.com/avencera/sqs_listener/blob/master/examples/with_creds.rs)
```rust,ignore
use std::env;
use sqs_listener::{
    credential::StaticProvider, request::HttpClient, Region, SQSListener, SQSListenerClientBuilder,
};
#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;
    let aws_access_key_id =
        env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID env variable needs to be present");
    let aws_secret_access_key = env::var("AWS_SECRET_ACCESS_KEY")
        .expect("AWS_SECRET_ACCESS_KEY env variable needs to be present");
    let listener = SQSListener::new("".to_string(), |message| {
        println!("Message received {:#?}", message)
    });
    let client = SQSListenerClientBuilder::new_with(
        HttpClient::new().expect("failed to create request dispatcher"),
        StaticProvider::new_minimal(aws_access_key_id, aws_secret_access_key),
        Region::UsEast1,
    )
    .listener(listener)
    .build()?;
    let _ = client.start().await;
    Ok(())
}
```
*/

pub mod client;

use act_zero::runtimes::tokio::spawn_actor;
use act_zero::*;
pub use aws_config::environment::EnvironmentVariableCredentialsProvider;
use aws_credential_types::provider::ProvideCredentials;
pub use aws_sdk_config::config::Region;
use aws_sdk_sqs::client::Client;
use aws_sdk_sqs::operation::{
    delete_message::DeleteMessageError, receive_message::ReceiveMessageError,
};
pub use aws_sdk_sqs::types::Message;

use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
use aws_smithy_runtime_api::client::result::SdkError;
use derive_builder::Builder;
use std::time::Duration;

/// Used to build a new [SQSListenerClient]
pub type SQSListenerClientBuilder<F> = client::SQSListenerClientBuilder<F>;

/// Error type of building an [SQSListenerClient] from its [Builder](SQSListenerClientBuilder) fails
///
/// ```rust
/// #[non_exhaustive]
/// pub enum SQSListenerClientBuilderError {
///     UninitializedField(&'static str),
///     ValidationError(String),
/// }
/// ```

pub type SQSListenerClientBuilderError = client::SQSListenerClientBuilderError;
/// Error type for sqs_listener
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unable to receive messages: {0}")]
    ReceiveMessages(#[from] SdkError<ReceiveMessageError, HttpResponse>),

    #[error("unable to acknowledge message: {0}")]
    AckMessage(#[from] SdkError<DeleteMessageError, HttpResponse>),

    #[error("Message did not contain a message handle to use for acknowledging")]
    NoMessageHandle,

    #[error("Listener has stopped")]
    ListenerStopped,

    #[error("Unable to receive messages")]
    UnknownReceiveMessages,

    #[error("No messages in queue")]
    NoMessageInQueue,
}

/// Create a new Builder
impl<F: Fn(&Message) + Send + Sync> SQSListenerClientBuilder<F> {
    /// Create a new listener the default AWS client and queue_url
    pub fn new(region: Option<String>) -> Self {
        let region_provider = region
            .map(|r| Region::new(r))
            .unwrap_or_else(|| Region::new("us-west-2".to_string()));

        let credentials_provider = EnvironmentVariableCredentialsProvider::new();

        let conf = aws_sdk_sqs::Config::builder()
            .region(region_provider)
            .credentials_provider(credentials_provider)
            .build();

        let client = aws_sdk_sqs::Client::from_conf(conf);

        Self::new_with_client(client)
    }

    /// Create a new listener with custom credentials, region, and queue_url
    pub fn new_with(
        region: Option<String>,
        credentials_provider: impl ProvideCredentials + 'static,
    ) -> Self {
        let region_provider = region
            .map(|r| Region::new(r))
            .unwrap_or_else(|| Region::new("us-west-2".to_string()));

        let conf = aws_sdk_sqs::Config::builder()
            .region(region_provider)
            .credentials_provider(credentials_provider)
            .build();

        let client = aws_sdk_sqs::Client::from_conf(conf);

        Self::new_with_client(client)
    }

    /// Create new listener with a client and queue_url
    pub fn new_with_client(client: Client) -> Self {
        client::SQSListenerClientBuilder::priv_new_with_client(client)
    }

    pub fn build(
        self: SQSListenerClientBuilder<F>,
    ) -> Result<SQSListenerClient<F>, SQSListenerClientBuilderError> {
        let inner: client::SQSListenerClient<F> = self.priv_build()?;

        Ok(SQSListenerClient {
            inner: Some(inner),
            addr: Addr::detached(),
        })
    }
}

/// Listener for a `queue_url` with a handler function to be run on each received message
///
/// The handler function should take a [Message] and return a unit `()`
#[derive(Debug)]
pub struct SQSListener<F: Fn(&Message)> {
    /// Url for the SQS queue that you want to listen to
    queue_url: String,

    /// Function to call when a new message is received
    handler: F,
}

impl<F: Fn(&Message)> SQSListener<F> {
    pub fn new(queue_url: String, handler: F) -> Self {
        Self { queue_url, handler }
    }
}

/// Listener client, first build using [SQSListenerClientBuilder] and start by
/// calling [`start()`](SQSListenerClient::start())
///
/// Can also be used to manually [`ack()`](SQSListenerClient::ack_message()) messages
pub struct SQSListenerClient<F: Fn(&Message) + Sync + Send + 'static> {
    addr: Addr<client::SQSListenerClient<F>>,
    inner: Option<client::SQSListenerClient<F>>,
}

impl<F: Fn(&Message) + Sync + Send> Clone for SQSListenerClient<F> {
    fn clone(&self) -> Self {
        Self {
            addr: self.addr.clone(),
            inner: None,
        }
    }
}

impl<F: Fn(&Message) + Sync + Send> SQSListenerClient<F> {
    /// Starts the service, this will run forever until your application exits.
    pub async fn start(mut self) {
        self.addr = spawn_actor(self.inner.expect("impossible to not be set"));
        self.addr.termination().await
    }

    /// If you set `auto_ack` [Config](ConfigBuilder) option to false, you will need to manually
    /// acknowledge messages. If you don't you will receive the same message over and over again.
    ///
    /// Use this function to manually acknowledge messages. If `auto_ack` is to true, you will not
    /// need to use this function
    pub async fn ack_message(self, message: Message) -> Result<(), Error> {
        call!(self.addr.ack_message(message))
            .await
            .map_err(|_err| Error::ListenerStopped)??;

        Ok(())
    }
}

#[derive(Clone, Builder, Debug)]
#[doc(hidden)]
#[builder(pattern = "owned")]
#[builder(build_fn(name = "build_private", private))]
pub struct Config {
    #[builder(default = "Duration::from_secs(5_u64)")]
    /// How often to check for new messages, defaults to 5 seconds
    check_interval: Duration,

    #[builder(default = "true")]
    /// Determines if messages should be automatically acknowledges.
    /// Defaults to true, if disabled you must manually ack the message by calling [`sqs_listener_client.ack(message)`](SQSListenerClient::ack_message)
    auto_ack: bool,
}

impl ConfigBuilder {
    pub fn build(self) -> Config {
        self.build_private()
            .expect("will always work because all fields have defaults")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn creates_with_closure() {
        let hashmap: HashMap<String, String> = HashMap::new();

        let region = Some("us-west-2".to_string());
        let credentials_provider = EnvironmentVariableCredentialsProvider::new();
        let listener = SQSListener::new("".to_string(), move |message| {
            println!("HashMap: {:#?}", hashmap);
            println!("{:#?}", message)
        });

        let client = SQSListenerClientBuilder::new_with(region, credentials_provider)
            .listener(listener)
            .build();

        assert!(client.is_ok())
    }

    #[test]
    fn creates_with_config() {
        let hashmap: HashMap<String, String> = HashMap::new();

        let region = Some("us-west-2".to_string());
        let credentials_provider = EnvironmentVariableCredentialsProvider::new();
        let listener = SQSListener::new("".to_string(), move |message| {
            println!("HashMap: {:#?}", hashmap);
            println!("{:#?}", message)
        });

        let config = ConfigBuilder::default()
            .check_interval(Duration::from_millis(1000))
            .auto_ack(false)
            .build();

        let client = SQSListenerClientBuilder::new_with(region, credentials_provider)
            .listener(listener)
            .config(config)
            .build();

        assert!(client.is_ok())
    }
}
