//! Discord HTTP client using information from the cache.
//!
//! This module exports wrapper around twilight's HTTP client that use the cache
//! data to check permissions before making requests.

use thiserror::Error;
use twilight_http::{
    request::{channel::message::CreateMessage, guild::CreateGuildChannel},
    Client as HttpClient,
};
use twilight_model::{
    guild::Permissions,
    id::{
        marker::{ChannelMarker, GuildMarker},
        Id,
    },
};
use twilight_validate::channel::ChannelValidationError;

use crate::{permission::PermissionError, redis::RedisClient};

#[derive(Debug)]
pub struct CacheHttp<'a> {
    redis: &'a RedisClient,
    http: &'a HttpClient,
    guild_id: Id<GuildMarker>,
}

impl<'a> CacheHttp<'a> {
    /// Initialize a new [`CacheHttp`].
    pub(crate) fn new(
        redis: &'a RedisClient,
        http: &'a HttpClient,
        guild_id: Id<GuildMarker>,
    ) -> Self {
        Self {
            redis,
            http,
            guild_id,
        }
    }

    /// Send a message to a channel.
    ///
    /// This method ensures that the bot has the [`SEND_MESSAGES`],
    /// [`SEND_MESSAGES_IN_THREADS`], [`USE_EXTERNAL_EMOJIS`] and [`EMBED_LINKS`]
    /// permissions before executing the request.
    ///
    /// [`SEND_MESSAGES`]: Permissions::SEND_MESSAGES
    /// [`SEND_MESSAGES_IN_THREADS`]: Permissions::SEND_MESSAGES_IN_THREADS
    /// [`USE_EXTERNAL_EMOJIS`]: Permissions::USE_EXTERNAL_EMOJIS
    /// [`EMBED_LINKS`]: Permissions::EMBED_LINKS
    pub async fn create_message(
        &self,
        channel: Id<ChannelMarker>,
    ) -> Result<CreateMessage<'a>, CacheHttpError> {
        let permissions = self.redis.permissions(self.guild_id).await?;
        let (permissions, kind) = permissions.current_member().await?.channel(channel).await?;

        let send_messages = if kind.is_thread() {
            Permissions::SEND_MESSAGES
        } else {
            Permissions::SEND_MESSAGES_IN_THREADS
        };

        if !permissions
            .contains(send_messages | Permissions::USE_EXTERNAL_EMOJIS | Permissions::EMBED_LINKS)
        {
            return Err(CacheHttpError::CreateMessage);
        }

        Ok(self.http.create_message(channel))
    }

    /// Create a new guild channel.
    ///
    /// This method ensure that the bot has the [`MANAGE_CHANNELS`] permission.
    ///
    /// [`MANAGE_CHANNELS`]: Permissions::MANAGE_CHANNELS
    pub async fn create_guild_channel(
        &'a self,
        name: &'a str,
    ) -> Result<CreateGuildChannel<'a>, CacheHttpError> {
        let permissions = self
            .redis
            .permissions(self.guild_id)
            .await?
            .current_member()
            .await?
            .guild();

        if !permissions.contains(Permissions::MANAGE_CHANNELS) {
            return Err(CacheHttpError::CreateGuildChannel);
        }

        Ok(self.http.create_guild_channel(self.guild_id, name)?)
    }
}

#[derive(Debug, Error)]
pub enum CacheHttpError {
    #[error("permission computing failed: {0}")]
    Permission(#[from] PermissionError),
    #[error("missing permissions to send message")]
    CreateMessage,
    #[error("missing permissions to create channel")]
    CreateGuildChannel,
    #[error(transparent)]
    ChannelValidationError(#[from] ChannelValidationError),
}
