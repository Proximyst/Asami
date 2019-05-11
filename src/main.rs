#![feature(try_blocks)]

#[macro_use]
extern crate diesel;

use self::{
    commands::{DEVELOPER_GROUP, MISCELLANEOUS_GROUP},
    config::Configuration,
    data::{
        OwnerContainer, ServerSettings, ServerSettingsContainer,
        ShardManagerContainer, UserSettings, UserSettingsContainer, PostgreSqlContainer,
    },
    prelude::*,
};
use lru_time_cache::LruCache;
use serenity::{framework::StandardFramework, prelude::*};
use std::{collections::HashSet, sync::Arc, time::Duration};

mod commands;
mod config;
mod data;
mod error;
mod ketoswritewrapper;
mod serenityhandler;

pub mod consts;
pub mod scheme;

pub mod prelude {
    pub use super::data::{ShardManagerContainer, PostgreSqlContainer};
    pub use super::error::*;
    pub use log::{debug, error, info, trace, warn};
    pub use r2d2::{Pool, PooledConnection};
    pub use diesel::{r2d2::ConnectionManager as DieselConnectionManager, pg::PgConnection};

    pub type PgPool = Pool<DieselConnectionManager<PgConnection>>;
}

fn main() -> Result<()> {
    // Loads configuration from "config.toml" in PWD.
    // TODO? (Unsure still): Read directory from env
    let config: Configuration = {
        use std::{fs, io::prelude::*};
        match fs::File::open("config.toml") {
            Err(_) => {
                let config = Configuration::default();
                fs::write("config.toml", toml::to_string(&config)?)?;
                return Err(SettingErrorKind::ConfigFileGenerated.into());
            }
            Ok(mut f) => {
                let mut content = String::new();
                f.read_to_string(&mut content)?;
                toml::from_str(&content)?
            }
        }
    };

    // Initialise and use the SimpleLog logger along with the log crate's macros.
    println!("Making logger...");
    {
        use simplelog::{CombinedLogger, Config as LogConfig, LevelFilter, TermLogger};
        CombinedLogger::init(vec![TermLogger::new(
            if *config.debug_logging() {
                LevelFilter::Debug
            } else {
                LevelFilter::Info
            },
            LogConfig::default(),
        )
        .failure()?])?;
    }
    info!("The logger has been initalised.");

    // Connect to PgSql with R2D2 pooling.
    info!("Creating connection pool with PostgreSql...");
    let pgsql = DieselConnectionManager::new(config.pgsql_url().to_owned());
    let pgsql = PgPool::new(pgsql)?;
    info!("PgSql connection pool created!");

    // Create the Discord client.
    let mut discord_client: Client = Client::new(&config.token(), self::serenityhandler::SerenityHandler)?;

    // Load the owners from the application on Discord's developers page
    let owners = {
        let mut set = HashSet::new();
        set.insert(
            discord_client
                .cache_and_http
                .http
                .get_current_application_info()?
                .owner
                .id,
        );
        set
    };

    // Write data to the sharemap
    {
        let mut data = discord_client.data.write();
        data.insert::<ShardManagerContainer>(Arc::clone(&discord_client.shard_manager));
        data.insert::<PostgreSqlContainer>(pgsql.clone());
        #[rustfmt::skip]
        data.insert::<ServerSettingsContainer>(
            LruCache::with_expiry_duration(Duration::from_secs(60 * 5))
        );
        #[rustfmt::skip]
        data.insert::<UserSettingsContainer>(
            LruCache::with_expiry_duration(Duration::from_secs(60 * 5))
        );
        data.insert::<OwnerContainer>(owners.clone());
    }

    // Configure the bot
    discord_client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.with_whitespace(true) // allow `a! command`
                    .case_insensitivity(true) // allow `a!cOmMaNd`
                    .no_dm_prefix(true) // allow `command` in dm
                    .owners(owners) // set owners of the bot
                    .prefix("a!") // set the prefix to `a!`
                    .delimiters(vec![" "]) // split arguments at space; `a!test a b c` => `[a!test, a, b, c]`
            })
            .before(|ctx, msg, _| {
                {
                    // Always allow owners of the bot to use it
                    let data = ctx.data.read();
                    if data
                            .get::<OwnerContainer>()
                            .map(|s| s.contains(&msg.author.id))
                            .unwrap_or(false)
                    {
                        return true;
                    }
                }

                // TODO: Support server-specific permission levels
                // TODO: Support user blacklisting set by server admins
                let settings = match UserSettings::new(msg.author.id.0, &ctx.data) {
                    Err(e) => {
                        error!("Couldn't get user data: {:?}", e);
                        let _ = msg.reply(&ctx, "An error occurred while fetching your user data.");
                        return false;
                    }
                    Ok(s) => s,
                };
                let read = settings.read();
                if *read.blacklisted() {
                    return false;
                }

                // TODO: Support channel white-/blacklisting set by server admins
                // TODO: Support ignoring channels with certain permissions missing
                let guild_id = match msg.guild_id {
                    Some(s) => s,
                    None => return true,
                };
                let settings = match ServerSettings::new(guild_id.0, &ctx.data) {
                    Err(e) => {
                        error!("Couldn't get server data: {:?}", e);
                        return false;
                    }
                    Ok(s) => s,
                };
                let read = settings.read();
                !*read.blacklisted()
            })
            .after(|ctx, msg, _, err| {
                if let Err(e) = err {
                    // TODO: Log for bot owners
                    let _ = msg.reply(
                        &ctx,
                        &format!("The command erred with the following error: {}", e.0),
                    );
                }
            })
            .help(&self::commands::help::HELP_MENU_HELP_COMMAND)
            .group(&MISCELLANEOUS_GROUP)
            .group(&DEVELOPER_GROUP)
    );

    // Start and shard the bot as needed to work with discord
    discord_client.start_autosharded()?;

    Ok(())
}
