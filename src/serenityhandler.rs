use super::prelude::*;
use serenity::{
    model::{channel::Message, gateway::Ready, guild::Member, id::GuildId, user::User},
    prelude::*,
};
use std::sync::Arc;

pub struct SerenityHandler;

impl EventHandler for SerenityHandler {
    fn guild_member_addition(&self, ctx: Context, guild: GuildId, member: Member) {
        let current = &ctx.cache.read().user;
        if current.id != member.user_id() {
            return;
        }
        let server_settings = match crate::data::ServerSettings::new(guild.0, &ctx.data) {
            Ok(s) => s,
            Err(e) => {
                error!("Couldn't retrieve settings for {}: {:?}", guild.0, e);
                return;
            }
        };
        if *server_settings.read().blacklisted() {
            if let Err(e) = ctx.http.leave_guild(guild.0) {
                error!("Couldn't leave guild {}: {:?}", guild.0, e);
            }
        }
    }

    fn guild_member_removal(&self, ctx: Context, guild: GuildId, user: User, _: Option<Member>) {
        let current = &ctx.cache.read().user;
        if user.id != current.id {
            return;
        }
        let mut write = ctx.data.write();
        let cache = match write.get_mut::<crate::data::ServerSettingsContainer>() {
            Some(s) => s,
            None => {
                error!("Server settings container is not in the share map.");
                return;
            }
        };
        if let Some(s) = cache.get(&guild.0) {
            if *s.read().blacklisted() {
                return;
            }
        }
        if let Some(s) = cache.remove(&guild.0) {
            let s = match Arc::try_unwrap(s) {
                Ok(s) => s,
                Err(_) => {
                    error!("Couldn't unwrap server {}'s arc", guild.0);
                    return;
                },
            };
            match s.into_inner().delete().err() {
                None => {},
                Some(e) => error!("Couldn't delete {}: {}", guild.0, e),
            }
        }
        info!("Left guild {}", guild.0);
    }

    fn message(&self, _: Context, msg: Message) {
        debug!("{}: {}", msg.author.name, msg.content);
    }

    fn ready(&self, _: Context, ready: Ready) {
        info!(
            "{} is connected on shard {}/{}.",
            ready.user.name,
            ready.shard.map(|o| o[0]).unwrap_or(0) + 1,
            ready.shard.map(|o| o[1]).unwrap_or(0)
        );
    }
}
