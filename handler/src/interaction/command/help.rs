//! Help command.

use thiserror::Error;
use tracing::{error, instrument};
use twilight_interactions::{
    command::{CommandModel, CommandOption, CreateCommand, CreateOption},
    error::ParseError,
};
use twilight_util::builder::embed::EmbedBuilder;
use twilight_validate::embed::EmbedValidationError;

use crate::interaction::{
    context::CommandContext,
    response::{CommandResponse, InteractionError, InteractionErrorKind},
};

/// Help command model.
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "help", desc = "Show the list of available commands")]
pub struct HelpCommand {
    /// Displays the help for a specific command.
    pub command: Option<Command>,
}

/// Command list model.
#[derive(Debug, Clone, CommandOption, CreateOption)]
pub enum Command {
    #[option(name = "test", value = "test")]
    Test,
}

impl HelpCommand {
    /// Handle interaction for this command.
    #[instrument]
    pub async fn handle(context: CommandContext) -> Result<CommandResponse, HelpCommandError> {
        let _parsed = HelpCommand::from_interaction(context.data.into())?;

        let embed = EmbedBuilder::new().description("Hello world!").build();

        Ok(CommandResponse::Embed(embed))
    }
}

/// Error when executing [`HelpCommand`]
#[derive(Debug, Error)]
pub enum HelpCommandError {
    #[error("failed to parse command: {0}")]
    Parse(#[from] ParseError),
    #[error("failed to build embed: {0}")]
    Embed(#[from] EmbedValidationError),
}

impl InteractionError for HelpCommandError {
    const INTERACTION_NAME: &'static str = "help";

    fn into_error(self) -> InteractionErrorKind {
        match self {
            HelpCommandError::Parse(error) => InteractionErrorKind::internal(error),
            HelpCommandError::Embed(error) => InteractionErrorKind::internal(error),
        }
    }
}