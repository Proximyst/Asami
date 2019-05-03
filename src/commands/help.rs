use super::prelude::*;
use serenity::framework::standard::{help_commands, CommandGroup, HelpOptions};
use std::{collections::HashSet, hash::BuildHasher};

#[help]
#[individual_command_tip = "Hello! Hallo! こんにちは！Hola! Bonjour! 您好!\n\
If you want more information about a specific command, just enter it as an argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[lacking_permissions = "Hide"]
#[wrong_channel = "Strike"]
#[lacking_role = "Nothing"]
fn help_menu(
    ctx: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId, impl BuildHasher>,
) -> CommandResult {
    help_commands::with_embeds(ctx, msg, args, help_options, groups, owners)
}
