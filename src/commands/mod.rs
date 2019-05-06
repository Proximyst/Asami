pub(crate) mod prelude {
    pub use super::super::prelude::*;
    pub use crate::data::{ServerSettings, UserSettings};
    pub use serenity::{
        framework::standard::{
            macros::{check, command, group, help},
            Args, CommandError, CommandResult,
        },
        model::prelude::*,
        prelude::*,
        client::bridge::gateway::ShardManager,
    };
    pub use parking_lot::Mutex;
    pub use std::sync::Arc;
}

pub mod help;
pub mod miscellaneous;
pub mod owner;
use self::prelude::*;
use self::{miscellaneous::*, owner::*};

group!({
    name: "Developer",
    options: {
        owner_only: true,
        description: "Commands only available for the developer of the bot are located here.",
    },
    commands: [quit, evaluate, blacklist],
});

group!({
    name: "Miscellaneous",
    options: {
        description: "All miscellaneous commands which did not fit in any other group are located here.",
    },
    commands: [ping],
});
