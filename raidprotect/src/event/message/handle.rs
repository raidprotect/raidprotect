use std::sync::Arc;

use rosetta_i18n::Language;
use tracing::{error, info};
use twilight_model::channel::Message;
use twilight_util::builder::embed::EmbedBuilder;

use super::parser::parse_message;
use crate::{cluster::ClusterState, interaction::embed::COLOR_TRANSPARENT, translations::Lang};

/// Handle incoming [`Message`].
///
/// This method will forward message to the cache and various auto-moderation
/// modules.
pub async fn handle_message(message: Message, state: Arc<ClusterState>) {
    let parsed = parse_message(&message);
    state.redis().set(&parsed).await.ok();

    if is_old_command(&message.content) {
        let message = message.clone();
        let state = state.clone();

        tokio::spawn(warn_old_command(message, state));
    }

    info!("received message: {}", message.content) // Debug util real implementation
}

async fn warn_old_command(message: Message, state: Arc<ClusterState>) {
    let lang = message
        .author
        .locale
        .map(|locale| Lang::from(locale.as_str()))
        .unwrap_or_else(Lang::fallback);
    let embed = EmbedBuilder::new()
        .title(lang.warning_deprecated_command_title())
        .description(lang.warning_deprecated_command_description())
        .color(COLOR_TRANSPARENT)
        .build();

    match state
        .http()
        .create_message(message.channel_id)
        .embeds(&[embed])
    {
        Ok(msg) => {
            let msg = msg.reply(message.id);

            if let Err(error) = msg.exec().await {
                error!(error = ?error, "failed to warn user about question mark commands deprecation");
            }
        }
        _ => error!("failed to create embed"),
    }
}

fn is_old_command(content: &str) -> bool {
    const OLD_COMMANDS: [&str; 11] = [
        "?kick",
        "?userinfo",
        "?lock",
        "?unlock",
        "?raidmode",
        "?settings",
        "?captcha",
        "?ban",
        "?invite",
        "?clear",
        "?stats",
    ];

    OLD_COMMANDS.iter().any(|cmd| content.starts_with(cmd))
}
