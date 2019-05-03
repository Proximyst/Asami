use super::prelude::*;

#[command]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    let _ = msg.reply(&ctx, "Pong!");

    Ok(())
}
