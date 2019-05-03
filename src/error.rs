use failure::{Error as FError, Fail};
use serenity::{framework::standard::CommandResult, Error as SerenityError};
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, FError>;

#[derive(Debug, Fail)]
pub enum CommandUsageKind {
    #[fail(display = "There should only be a code block specified.")]
    DeveloperEvaluateUsage,
    #[fail(display = "No IDs were specified.")]
    DeveloperBlacklistNoIds,
}

#[derive(Debug, Fail)]
pub enum SettingErrorKind {
    #[fail(display = "A config.toml file has been generated.
Please fill it with proper information.")]
    ConfigFileGenerated,
}

#[derive(Debug, Fail)]
pub enum ForeignErrorKind {
    #[fail(display = "{}", _0)]
    Serenity(SerenityError),
}

#[derive(Debug, Fail)]
pub enum StdErrorKind {
    #[fail(display = "Option value not present.")]
    None,
    #[fail(display = "{}", _0)]
    StringValue(String),
}

pub trait OptionExt<T> {
    fn failure(self) -> Result<T>;
}

pub trait ToCommandResult {
    fn command_result(self) -> CommandResult;
}

impl<T> OptionExt<T> for Option<T> {
    fn failure(self) -> Result<T> {
        self.ok_or(StdErrorKind::None.into())
    }
}
