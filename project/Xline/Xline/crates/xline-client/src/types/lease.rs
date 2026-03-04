use std::pin::Pin;

use futures::{Stream, StreamExt};
pub use xlineapi::{
    LeaseGrantResponse, LeaseKeepAliveResponse, LeaseLeasesResponse, LeaseRevokeResponse,
    LeaseStatus, LeaseTimeToLiveResponse,
};

use crate::error::{Result, XlineClientError};

/// The lease keep alive handle.
#[derive(Debug)]
pub struct LeaseKeeper {
    /// lease id
    id: i64,
    /// sender to send keep alive request (tokio mpsc, matching bidi_streaming_call)
    sender: tokio::sync::mpsc::Sender<xlineapi::LeaseKeepAliveRequest>,
}

impl LeaseKeeper {
    /// Creates a new `LeaseKeeper`.
    #[inline]
    #[must_use]
    pub fn new(
        id: i64,
        sender: tokio::sync::mpsc::Sender<xlineapi::LeaseKeepAliveRequest>,
    ) -> Self {
        Self { id, sender }
    }

    /// The lease id which user want to keep alive.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> i64 {
        self.id
    }

    /// Sends a keep alive request and receive response
    ///
    /// # Errors
    ///
    /// This function will return an error if the inner channel is closed
    #[inline]
    pub fn keep_alive(&mut self) -> Result<()> {
        self.sender
            .try_send(xlineapi::LeaseKeepAliveRequest { id: self.id })
            .map_err(|e| XlineClientError::LeaseError(e.to_string()))
    }
}

/// Stream of lease keep-alive responses backed by the QUIC bidirectional channel.
///
/// Replaces the previous `tonic::Streaming<LeaseKeepAliveResponse>`-based wrapper.
pub struct LeaseKeepAliveStream {
    /// Underlying QUIC response stream
    inner: Pin<
        Box<
            dyn Stream<
                    Item = std::result::Result<LeaseKeepAliveResponse, curp::rpc::CurpError>,
                > + Send,
        >,
    >,
}

impl std::fmt::Debug for LeaseKeepAliveStream {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LeaseKeepAliveStream")
            .finish_non_exhaustive()
    }
}

impl LeaseKeepAliveStream {
    /// Creates a new `LeaseKeepAliveStream`.
    #[inline]
    #[must_use]
    pub fn new(
        inner: Pin<
            Box<
                dyn Stream<
                        Item = std::result::Result<LeaseKeepAliveResponse, curp::rpc::CurpError>,
                    > + Send,
            >,
        >,
    ) -> Self {
        Self { inner }
    }

    /// Returns the next keep-alive response, or `Ok(None)` when the stream ends.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying QUIC transport signals a failure.
    #[inline]
    pub async fn message(&mut self) -> Result<Option<LeaseKeepAliveResponse>> {
        match self.inner.next().await {
            Some(Ok(resp)) => Ok(Some(resp)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }
}
