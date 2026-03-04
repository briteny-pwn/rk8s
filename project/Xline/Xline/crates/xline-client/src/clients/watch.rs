use std::sync::Arc;

use curp::rpc::{MethodId, QuicChannel};
use xlineapi::{RequestUnion, WatchResponse};

use crate::{
    build_meta,
    error::{Result, XlineClientError},
    types::watch::{WATCH_CHANNEL_SIZE, WatchOptions, WatchStreaming, Watcher},
};

/// Default timeout for watch stream-setup handshake.
const WATCH_SETUP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// Client for Watch operations.
#[derive(Clone, Debug)]
pub struct WatchClient {
    /// QUIC channel shared with all sub-clients
    channel: Arc<QuicChannel>,
    /// Auth token
    token: Option<String>,
}

impl WatchClient {
    /// Creates a new `WatchClient`
    #[inline]
    #[must_use]
    pub fn new(channel: Arc<QuicChannel>, token: Option<String>) -> Self {
        Self { channel, token }
    }

    /// Watches for events happening or that have happened. Both input and output
    /// are streams; the input has the set of keys to watch and output has events.
    ///
    /// # Errors
    ///
    /// Returns an error if the initial QUIC stream setup fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xline_client::{Client, ClientOptions};
    /// use xline_client::types::watch::WatchOptions;
    /// use anyhow::Result;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     // the name and address of all curp members
    ///     let curp_members = ["10.0.0.1:2379", "10.0.0.2:2379", "10.0.0.3:2379"];
    ///
    ///     let mut client = Client::connect(curp_members, todo!("provide ClientOptions"))
    ///         .await?
    ///         .watch_client();
    ///     let (mut watcher, mut stream) = client.watch("key", None).await?;
    ///
    ///     while let Some(resp) = stream.message().await? {
    ///         println!("{:?}", resp);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub async fn watch<K: Into<Vec<u8>>>(
        &mut self,
        key: K,
        options: Option<WatchOptions>,
    ) -> Result<(Watcher, WatchStreaming)> {
        let watch_options = options.unwrap_or_default().with_key(key);
        let watch_id = watch_options.inner.watch_id;

        // Open the bidirectional QUIC stream.
        let (sender, response_stream) = self
            .channel
            .bidi_streaming_call::<xlineapi::WatchRequest, WatchResponse>(
                MethodId::XlineWatch,
                build_meta(&self.token),
                WATCH_CHANNEL_SIZE,
            )
            .await
            .map_err(XlineClientError::from)?;

        // Send the initial WatchCreateRequest.
        let create_req = xlineapi::WatchRequest {
            request_union: Some(RequestUnion::CreateRequest(watch_options.into())),
        };
        sender
            .send(create_req)
            .await
            .map_err(|e| XlineClientError::WatchError(e.to_string()))?;

        // WatchStreaming keeps a clone of the sender alive so the send task stays running.
        let watcher = Watcher::new(watch_id, sender.clone());
        let streaming = WatchStreaming::new(response_stream, sender);

        Ok((watcher, streaming))
    }
}
