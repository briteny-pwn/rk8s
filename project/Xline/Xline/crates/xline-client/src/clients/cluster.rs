use std::{sync::Arc, time::Duration};

use curp::rpc::{MethodId, QuicChannel};
use xlineapi::{
    MemberAddRequest, MemberAddResponse, MemberListRequest, MemberListResponse,
    MemberPromoteRequest, MemberPromoteResponse, MemberRemoveRequest, MemberRemoveResponse,
    MemberUpdateRequest, MemberUpdateResponse,
};

use crate::{build_meta, error::{Result, XlineClientError}};

/// Timeout for unary cluster management calls.
const CALL_TIMEOUT: Duration = Duration::from_secs(30);

/// Client for Cluster operations.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ClusterClient {
    /// QUIC channel for all cluster management calls.
    channel: Arc<QuicChannel>,
    /// Auth token.
    token: Option<String>,
}

impl ClusterClient {
    /// Create a new cluster client
    #[inline]
    #[must_use]
    pub fn new(channel: Arc<QuicChannel>, token: Option<String>) -> Self {
        Self { channel, token }
    }

    /// Add a new member to the cluster.
    ///
    /// # Errors
    ///
    /// Returns an error if the request could not be sent or if the response is invalid.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xline_client::{Client, ClientOptions};
    /// use anyhow::Result;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let curp_members = ["10.0.0.1:2379", "10.0.0.2:2379", "10.0.0.3:2379"];
    ///
    ///     let mut client = Client::connect(curp_members, todo!("provide ClientOptions"))
    ///         .await?
    ///         .cluster_client();
    ///
    ///     let resp = client.member_add(["127.0.0.1:2380"], true).await?;
    ///
    ///     println!(
    ///         "members: {:?}, added: {:?}",
    ///         resp.members, resp.member
    ///     );
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub async fn member_add<I: Into<String>, P: Into<Vec<I>>>(
        &mut self,
        peer_urls: P,
        is_learner: bool,
    ) -> Result<MemberAddResponse> {
        self.channel
            .unary_call::<MemberAddRequest, MemberAddResponse>(
                MethodId::XlineMemberAdd,
                MemberAddRequest {
                    peer_ur_ls: peer_urls.into().into_iter().map(Into::into).collect(),
                    is_learner,
                },
                build_meta(&self.token),
                CALL_TIMEOUT,
            )
            .await
            .map_err(XlineClientError::from)
    }

    /// Remove an existing member from the cluster.
    ///
    /// # Errors
    ///
    /// Returns an error if the request could not be sent or if the response is invalid.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xline_client::{Client, ClientOptions};
    /// use anyhow::Result;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let curp_members = ["10.0.0.1:2379", "10.0.0.2:2379", "10.0.0.3:2379"];
    ///
    ///     let mut client = Client::connect(curp_members, todo!("provide ClientOptions"))
    ///         .await?
    ///         .cluster_client();
    ///     let resp = client.member_remove(1).await?;
    ///
    ///     println!("members: {:?}", resp.members);
    ///
    ///     Ok(())
    ///  }
    ///
    #[inline]
    pub async fn member_remove(&mut self, id: u64) -> Result<MemberRemoveResponse> {
        self.channel
            .unary_call::<MemberRemoveRequest, MemberRemoveResponse>(
                MethodId::XlineMemberRemove,
                MemberRemoveRequest { id },
                build_meta(&self.token),
                CALL_TIMEOUT,
            )
            .await
            .map_err(XlineClientError::from)
    }

    /// Promote an existing member to be the leader of the cluster.
    ///
    /// # Errors
    ///
    /// Returns an error if the request could not be sent or if the response is invalid.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xline_client::{Client, ClientOptions};
    /// use anyhow::Result;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let curp_members = ["10.0.0.1:2379", "10.0.0.2:2379", "10.0.0.3:2379"];
    ///
    ///     let mut client = Client::connect(curp_members, todo!("provide ClientOptions"))
    ///         .await?
    ///         .cluster_client();
    ///     let resp = client.member_promote(1).await?;
    ///
    ///     println!("members: {:?}", resp.members);
    ///
    ///     Ok(())
    /// }
    ///
    #[inline]
    pub async fn member_promote(&mut self, id: u64) -> Result<MemberPromoteResponse> {
        self.channel
            .unary_call::<MemberPromoteRequest, MemberPromoteResponse>(
                MethodId::XlineMemberPromote,
                MemberPromoteRequest { id },
                build_meta(&self.token),
                CALL_TIMEOUT,
            )
            .await
            .map_err(XlineClientError::from)
    }

    /// Update an existing member in the cluster.
    ///
    /// # Errors
    ///
    /// Returns an error if the request could not be sent or if the response is invalid.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xline_client::{Client, ClientOptions};
    /// use anyhow::Result;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let curp_members = ["10.0.0.1:2379", "10.0.0.2:2379", "10.0.0.3:2379"];
    ///
    ///     let mut client = Client::connect(curp_members, todo!("provide ClientOptions"))
    ///         .await?
    ///         .cluster_client();
    ///     let resp = client.member_update(1, ["127.0.0.1:2379"]).await?;
    ///
    ///     println!("members: {:?}", resp.members);
    ///
    ///     Ok(())
    ///  }
    ///
    #[inline]
    pub async fn member_update<I: Into<String>, P: Into<Vec<I>>>(
        &mut self,
        id: u64,
        peer_urls: P,
    ) -> Result<MemberUpdateResponse> {
        self.channel
            .unary_call::<MemberUpdateRequest, MemberUpdateResponse>(
                MethodId::XlineMemberUpdate,
                MemberUpdateRequest {
                    id,
                    peer_ur_ls: peer_urls.into().into_iter().map(Into::into).collect(),
                },
                build_meta(&self.token),
                CALL_TIMEOUT,
            )
            .await
            .map_err(XlineClientError::from)
    }

    /// List all members in the cluster.
    ///
    /// # Errors
    ///
    /// Returns an error if the request could not be sent or if the response is invalid.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use xline_client::{Client, ClientOptions};
    /// use anyhow::Result;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let curp_members = ["10.0.0.1:2379", "10.0.0.2:2379", "10.0.0.3:2379"];
    ///
    ///     let mut client = Client::connect(curp_members, todo!("provide ClientOptions"))
    ///         .await?
    ///         .cluster_client();
    ///     let resp = client.member_list(false).await?;
    ///
    ///     println!("members: {:?}", resp.members);
    ///
    ///     Ok(())
    /// }
    #[inline]
    pub async fn member_list(&mut self, linearizable: bool) -> Result<MemberListResponse> {
        self.channel
            .unary_call::<MemberListRequest, MemberListResponse>(
                MethodId::XlineMemberList,
                MemberListRequest { linearizable },
                build_meta(&self.token),
                CALL_TIMEOUT,
            )
            .await
            .map_err(XlineClientError::from)
    }
}
