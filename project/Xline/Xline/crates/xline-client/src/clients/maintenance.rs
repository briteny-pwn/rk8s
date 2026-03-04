use std::{pin::Pin, sync::Arc, time::Duration};

use curp::rpc::{MethodId, QuicChannel};
use futures::{Stream, StreamExt};
use xlineapi::{
    AlarmAction, AlarmRequest, AlarmResponse, AlarmType, SnapshotRequest, SnapshotResponse,
    StatusRequest, StatusResponse,
};

use crate::{build_meta, error::{Result, XlineClientError}};

/// Timeout for unary maintenance calls.
const CALL_TIMEOUT: Duration = Duration::from_secs(30);

/// Timeout for the snapshot stream setup (the stream itself is long-lived).
const SNAPSHOT_TIMEOUT: Duration = Duration::from_secs(3600);

/// Server-streaming response for snapshot operations.
///
/// Wraps the QUIC server-streaming response and provides a `message()` method
/// compatible with the previous `tonic::Streaming<SnapshotResponse>` interface.
pub struct SnapshotStream {
    /// Underlying QUIC stream
    inner: Pin<
        Box<
            dyn Stream<
                    Item = std::result::Result<SnapshotResponse, curp::rpc::CurpError>,
                > + Send,
        >,
    >,
}

impl std::fmt::Debug for SnapshotStream {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnapshotStream").finish_non_exhaustive()
    }
}

impl SnapshotStream {
    /// Returns the next snapshot chunk, or `Ok(None)` when the stream ends.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying QUIC transport signals a failure.
    #[inline]
    pub async fn message(&mut self) -> Result<Option<SnapshotResponse>> {
        match self.inner.next().await {
            Some(Ok(resp)) => Ok(Some(resp)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }
}

/// Client for Maintenance operations.
#[derive(Clone, Debug)]
pub struct MaintenanceClient {
    /// QUIC channel for all maintenance calls.
    channel: Arc<QuicChannel>,
    /// Auth token.
    token: Option<String>,
}

impl MaintenanceClient {
    /// Creates a new maintenance client
    #[inline]
    #[must_use]
    pub fn new(channel: Arc<QuicChannel>, token: Option<String>) -> Self {
        Self { channel, token }
    }

    /// Gets a snapshot over a stream
    ///
    /// # Errors
    ///
    /// This function will return an error if the inner RPC client encountered a propose failure
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xline_client::{Client, ClientOptions};
    /// use anyhow::Result;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     // the name and address of all curp members
    ///     let curp_members = ["10.0.0.1:2379", "10.0.0.2:2379", "10.0.0.3:2379"];
    ///
    ///     let mut client = Client::connect(curp_members, todo!("provide ClientOptions"))
    ///         .await?
    ///         .maintenance_client();
    ///
    ///     // snapshot
    ///     let mut msg = client.snapshot().await?;
    ///     let mut snapshot = vec![];
    ///     loop {
    ///         if let Some(resp) = msg.message().await? {
    ///             snapshot.extend_from_slice(&resp.blob);
    ///             if resp.remaining_bytes == 0 {
    ///                 break;
    ///             }
    ///         }
    ///     }
    ///     println!("snapshot size: {}", snapshot.len());
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub async fn snapshot(&mut self) -> Result<SnapshotStream> {
        let inner = self
            .channel
            .server_streaming_call::<SnapshotRequest, SnapshotResponse>(
                MethodId::XlineSnapshot,
                SnapshotRequest {},
                build_meta(&self.token),
                SNAPSHOT_TIMEOUT,
            )
            .await
            .map_err(XlineClientError::from)?;

        Ok(SnapshotStream { inner })
    }

    /// Sends a alarm request
    ///
    /// # Errors
    ///
    /// This function will return an error if the inner RPC client encountered a propose failure
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xline_client::{Client, ClientOptions};
    /// use xlineapi::{AlarmAction, AlarmRequest, AlarmType};
    /// use anyhow::Result;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     // the name and address of all curp members
    ///     let curp_members = ["10.0.0.1:2379", "10.0.0.2:2379", "10.0.0.3:2379"];
    ///
    ///     let mut client = Client::connect(curp_members, todo!("provide ClientOptions"))
    ///         .await?
    ///         .maintenance_client();
    ///
    ///     client.alarm(AlarmAction::Get, 0, AlarmType::None).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub async fn alarm(
        &mut self,
        action: AlarmAction,
        member_id: u64,
        alarm_type: AlarmType,
    ) -> Result<AlarmResponse> {
        self.channel
            .unary_call::<AlarmRequest, AlarmResponse>(
                MethodId::XlineAlarm,
                AlarmRequest {
                    action: action.into(),
                    member_id,
                    alarm: alarm_type.into(),
                },
                build_meta(&self.token),
                CALL_TIMEOUT,
            )
            .await
            .map_err(XlineClientError::from)
    }

    /// Sends a status request
    ///
    /// # Errors
    ///
    /// This function will return an error if the inner RPC client encountered a propose failure
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xline_client::{Client, ClientOptions};
    /// use anyhow::Result;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     // the name and address of all curp members
    ///     let curp_members = ["10.0.0.1:2379", "10.0.0.2:2379", "10.0.0.3:2379"];
    ///
    ///     let mut client = Client::connect(curp_members, todo!("provide ClientOptions"))
    ///         .await?
    ///         .maintenance_client();
    ///
    ///     client.status().await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub async fn status(&mut self) -> Result<StatusResponse> {
        self.channel
            .unary_call::<StatusRequest, StatusResponse>(
                MethodId::XlineStatus,
                StatusRequest::default(),
                build_meta(&self.token),
                CALL_TIMEOUT,
            )
            .await
            .map_err(XlineClientError::from)
    }
}
