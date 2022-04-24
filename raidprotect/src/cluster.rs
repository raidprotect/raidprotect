//! Shards cluster implementation.

use std::sync::Arc;

use futures::StreamExt;
use raidprotect_cache::{InMemoryCache, MessageCache, MessageExpireTask};
use raidprotect_interaction::register_commands;
use raidprotect_model::{
    interaction::component::{PendingComponentExpireTask, PendingComponentQueue},
    mongodb::{MongoDbClient, MongoDbError},
};
use raidprotect_state::ClusterState;
use raidprotect_util::shutdown::ShutdownSubscriber;
use thiserror::Error;
use tracing::{info, info_span, instrument, trace};
use twilight_gateway::{
    cluster::{ClusterStartError, Events, ShardScheme},
    Cluster, Intents,
};
use twilight_http::{response::DeserializeBodyError, Client as HttpClient, Error as HttpError};
use twilight_model::gateway::{
    payload::outgoing::update_presence::UpdatePresencePayload,
    presence::{ActivityType, MinimalActivity, Status},
};

use crate::event::ProcessEvent;

/// Discord shards cluster.
///
/// This type is a wrapper around twilight [`Cluster`] and manages incoming
/// events from Discord.
#[derive(Debug)]
pub struct ShardCluster {
    /// Inner shard cluster managed by twilight
    cluster: Arc<Cluster>,
    /// Events stream
    events: Events,
    /// Shared cluster state
    state: Arc<ClusterState>,
    /// Message cache expiration task
    messages_expire: MessageExpireTask,
    /// Pending components expiration task
    pending_components_expire: PendingComponentExpireTask,
}

impl ShardCluster {
    /// Initialize a new [`ShardCluster`].
    ///
    /// This method also initialize an [`HttpClient`] and an [`InMemoryCache`],
    /// that can be later retrieved using corresponding methods.
    pub async fn new(
        token: String,
        command_guild: Option<u64>,
        mongodb_uri: &str,
        mongodb_database: String,
    ) -> Result<Self, ClusterError> {
        // Initialize HTTP client and get current user.
        let http = Arc::new(HttpClient::new(token.clone()));
        let application = http
            .current_user_application()
            .exec()
            .await?
            .model()
            .await?;

        info!("logged as {} with ID {}", application.name, application.id);

        let cache = InMemoryCache::new(application.id.cast());
        let (messages, messages_expire) = MessageCache::new();
        let (pending_components, pending_components_expire) = PendingComponentQueue::new();

        let mongodb = MongoDbClient::connect(mongodb_uri, mongodb_database).await?;
        mongodb.ping().await?; // Ensure database is reachable

        let intents = Intents::GUILDS
            | Intents::GUILD_MEMBERS
            | Intents::GUILD_MESSAGES
            | Intents::MESSAGE_CONTENT;

        let (cluster, events) = Cluster::builder(token.to_string(), intents)
            .http_client(http.clone())
            .shard_scheme(ShardScheme::Auto)
            .presence(presence())
            .build()
            .await?;

        info!("started cluster with {} shards", cluster.shards().len());

        let state = ClusterState::new(cache, mongodb, http, messages, pending_components);

        register_commands(&state, application.id, command_guild).await;

        Ok(Self {
            cluster: Arc::new(cluster),
            events,
            state: Arc::new(state),
            messages_expire,
            pending_components_expire,
        })
    }

    /// Start the cluster and handle incoming events.
    ///
    /// A [`ShutdownSubscriber`] must be provided to gracefully stop the cluster.
    #[instrument(name = "start_cluster", skip_all)]
    pub async fn start(mut self, mut shutdown: ShutdownSubscriber) {
        // Start the cluster
        let cluster = self.cluster.clone();
        tokio::spawn(async move {
            cluster.up().await;
        });

        // Start message cache expiration task
        let messages_expire = self.messages_expire.clone();
        tokio::spawn(async move {
            messages_expire.run().await;
        });

        // Start pending components expiration task
        let pending_components_expire = self.pending_components_expire.clone();
        tokio::spawn(async move {
            pending_components_expire.run().await;
        });

        // Handle incoming events
        tokio::select! {
            _ = self.handle_events() => {},
            _ = shutdown.wait_shutdown() => {},
        };

        self.cluster.down();
    }

    /// Handle incoming events
    async fn handle_events(&mut self) {
        while let Some((_shard_id, event)) = self.events.next().await {
            let span = info_span!("handle_event");

            span.in_scope(|| {
                trace!(event = ?event, "received event");
                event.process(self.state.clone());
            });
        }
    }
}

/// Get the bot presence.
fn presence() -> UpdatePresencePayload {
    let activity = MinimalActivity {
        kind: ActivityType::Watching,
        name: String::from("raidprotect.org"),
        url: None,
    };

    UpdatePresencePayload {
        activities: vec![activity.into()],
        afk: false,
        since: None,
        status: Status::Online,
    }
}

/// Error when initializing a [`ShardCluster`].
#[derive(Debug, Error)]
pub enum ClusterError {
    /// HTTP request failed
    #[error("http error: {0}")]
    Http(#[from] HttpError),
    /// Response body deserialization error
    #[error("deserialize error: {0}")]
    Deserialize(#[from] DeserializeBodyError),
    /// Failed to start cluster
    #[error("failed to start cluster: {0}")]
    Start(#[from] ClusterStartError),
    /// Error while connecting to MongoDB database
    #[error("failed to connect to MongoDB")]
    MongoDb(#[from] MongoDbError),
}