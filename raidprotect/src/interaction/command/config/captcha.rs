//! Captcha configuration commands.

use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::{
        component::{button::ButtonStyle, ActionRow, Button, Component},
        interaction::{application_command::InteractionChannel, Interaction},
    },
    channel::message::MessageFlags,
    guild::Role,
    http::interaction::InteractionResponseType,
};
use twilight_util::builder::{embed::EmbedBuilder, InteractionResponseDataBuilder};

use crate::{
    cluster::ClusterState,
    interaction::{
        embed::{self, COLOR_RED},
        response::InteractionResponse,
        util::InteractionExt,
    },
};

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "captcha", desc = "Configure the RaidProtect captcha")]
pub enum CaptchaConfigCommand {
    #[command(name = "enable")]
    Enable(CaptchaEnableCommand),
    #[command(name = "disable")]
    Disable(CaptchaDisableCommand),
    #[command(name = "logs")]
    Logs(CaptchaLogsCommand),
    #[command(name = "autorole-add")]
    AutoroleAdd(CaptchaAutoroleAddCommand),
    #[command(name = "autorole-remove")]
    AutoroleRemove(CaptchaAutoroleRemoveCommand),
    #[command(name = "autorole-list")]
    AutoroleList(CaptchaAutoroleListCommand),
}

impl CaptchaConfigCommand {
    pub(super) async fn exec(
        self,
        interaction: Interaction,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        match self {
            CaptchaConfigCommand::Enable(command) => command.exec(interaction, state).await,
            CaptchaConfigCommand::Disable(_) => todo!(),
            CaptchaConfigCommand::Logs(_) => todo!(),
            CaptchaConfigCommand::AutoroleAdd(_) => todo!(),
            CaptchaConfigCommand::AutoroleRemove(_) => todo!(),
            CaptchaConfigCommand::AutoroleList(_) => todo!(),
        }
    }
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "enable", desc = "Enable the RaidProtect captcha")]
pub struct CaptchaEnableCommand;

impl CaptchaEnableCommand {
    async fn exec(
        self,
        interaction: Interaction,
        _state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let lang = interaction.locale()?;

        let embed = EmbedBuilder::new()
            .color(COLOR_RED)
            .title(lang.captcha_enable_title())
            .description(lang.captcha_enable_description())
            .build();

        let components = Component::ActionRow(ActionRow {
            components: vec![
                Component::Button(Button {
                    custom_id: Some("captcha_enable".to_string()),
                    disabled: false,
                    emoji: None,
                    label: Some(lang.captcha_enable_button().to_string()),
                    style: ButtonStyle::Success,
                    url: None,
                }),
                Component::Button(Button {
                    custom_id: None,
                    disabled: false,
                    emoji: None,
                    label: Some(lang.learn_more().to_string()),
                    style: ButtonStyle::Link,
                    url: Some("https://docs.raidprotect.org/".to_string()),
                }),
            ],
        });

        let response = InteractionResponseDataBuilder::new()
            .embeds([embed])
            .components([components])
            .flags(MessageFlags::EPHEMERAL)
            .build();

        Ok(InteractionResponse::Raw {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(response),
        })
    }
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "disable", desc = "Disable the RaidProtect captcha")]
pub struct CaptchaDisableCommand;

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "logs", desc = "Set the RaidProtect captcha logs channel")]
pub struct CaptchaLogsCommand {
    /// Channel to send the logs to.
    channel: InteractionChannel,
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "autorole-add",
    desc = "Add a role to the RaidProtect captcha autorole"
)]
pub struct CaptchaAutoroleAddCommand {
    /// Role to add to the autorole.
    role: Role,
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "autorole-remove",
    desc = "Remove a role from the RaidProtect captcha autorole"
)]
pub struct CaptchaAutoroleRemoveCommand {
    /// Role to remove from the autorole.
    role: Role,
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "autorole-list",
    desc = "List the roles of the RaidProtect captcha autorole"
)]
pub struct CaptchaAutoroleListCommand;
