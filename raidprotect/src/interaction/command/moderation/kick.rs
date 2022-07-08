//! Kick command.
//!
//! The command allows to kick a member from the server. User can specify a
//! reason directly in the command (as an optional parameter), or in the modal
//! that is shown if it hasn't been set in the command.
//!
//! When a user is kicked, the action is logged in the database and a message is
//! sent in the guild's logs channel. The kicked user receives a pm with the
//! reason of the kick.

use anyhow::anyhow;
use nanoid::nanoid;
use raidprotect_model::{
    cache::model::interaction::{PendingModal, PendingSanction},
    mongodb::modlog::ModlogType,
};
use tracing::instrument;
use twilight_interactions::command::{CommandModel, CreateCommand, ResolvedUser};
use twilight_model::{
    application::{
        component::{text_input::TextInputStyle, ActionRow, Component, TextInput},
        interaction::application_command::CommandData,
    },
    guild::Permissions,
    user::User,
};

use crate::{
    cluster::ClusterState,
    interaction::{context::InteractionContext, embed, response::InteractionResponse},
    translations::Lang,
    util::TextProcessExt,
};

/// Kick command model.
///
/// See the [`module`][self] documentation for more information.
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "kick",
    desc = "Kicks a user from the server",
    default_permissions = "KickCommand::default_permissions",
    dm_permission = false
)]
pub struct KickCommand {
    /// Member to kick.
    #[command(rename = "member")]
    pub user: ResolvedUser,
    /// Reason for kick.
    pub reason: Option<String>,
}

impl KickCommand {
    fn default_permissions() -> Permissions {
        Permissions::KICK_MEMBERS
    }

    #[instrument]
    pub async fn handle(
        context: InteractionContext<CommandData>,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let parsed = KickCommand::from_interaction(context.data.into())?;
        let guild = &context
            .guild
            .ok_or_else(|| anyhow!("command not executed in a guild"))?;

        let user = parsed.user.resolved;
        let member = match parsed.user.member {
            Some(member) => member,
            None => return Ok(embed::kick::not_member(user.name)),
        };

        // Fetch the author and the bot permissions.
        let permissions = state.redis().permissions(guild.guild.id).await?;
        let author_permissions = permissions
            .member(context.user.id, &guild.member.roles)
            .await?;
        let member_permissions = permissions.member(user.id, &member.roles).await?;
        let bot_permissions = permissions.current_member().await?;

        // Check if the author and the bot have required permissions.
        if member_permissions.is_owner() {
            return Ok(embed::kick::member_owner());
        }

        if !bot_permissions.guild().contains(Permissions::KICK_MEMBERS) {
            return Ok(embed::kick::bot_missing_permission());
        }

        // Check if the role hierarchy allow the author and the bot to perform
        // the kick.
        let member_highest_role = member_permissions.highest_role();

        if member_highest_role >= author_permissions.highest_role() {
            return Ok(embed::kick::user_hierarchy());
        }

        if member_highest_role >= bot_permissions.highest_role() {
            return Ok(embed::kick::bot_hierarchy());
        }

        // Send reason modal.
        match parsed.reason {
            Some(_reason) => Ok(InteractionResponse::EphemeralDeferredMessage),
            None => KickCommand::reason_modal(user, guild.config().enforce_reason, state).await,
        }
    }

    /// Modal that asks the user to enter a reason for the kick.
    ///
    /// This modal is only shown if the user has not specified a reason in the
    /// initial command.
    async fn reason_modal(
        user: User,
        enforce_reason: bool,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let username = user.name.truncate(15);
        let components = vec![
            Component::ActionRow(ActionRow {
                components: vec![Component::TextInput(TextInput {
                    custom_id: "reason".to_string(),
                    label: Lang::Fr.modal_kick_reason_label().to_string(),
                    max_length: Some(100),
                    min_length: None,
                    placeholder: Some(Lang::Fr.modal_reason_placeholder().to_string()),
                    required: Some(enforce_reason),
                    style: TextInputStyle::Short,
                    value: None,
                })],
            }),
            Component::ActionRow(ActionRow {
                components: vec![Component::TextInput(TextInput {
                    custom_id: "notes".to_string(),
                    label: Lang::Fr.modal_notes_label().to_string(),
                    max_length: Some(1000),
                    min_length: None,
                    placeholder: Some(Lang::Fr.modal_notes_placeholder().to_string()),
                    required: Some(false),
                    style: TextInputStyle::Paragraph,
                    value: None,
                })],
            }),
        ];

        // Add pending component in Redis
        let custom_id = nanoid!();
        let pending = PendingModal::Sanction(PendingSanction {
            id: custom_id.clone(),
            kind: ModlogType::Kick,
            user,
        });

        state.redis().set(&pending).await?;

        Ok(InteractionResponse::Modal {
            custom_id,
            title: Lang::Fr.modal_kick_title(username),
            components,
        })
    }
}