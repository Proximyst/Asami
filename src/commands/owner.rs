use super::prelude::*;

#[command]
#[owners_only]
fn quit(ctx: &mut Context, msg: &Message) -> CommandResult {
    let _ = msg.reply(&ctx, "Shutting down using the Context given.");
    ctx.shard.shutdown_clean();

    Ok(())
}

#[command]
#[owners_only]
fn blacklist(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let server = args.single::<bool>()?;

    let mut ids = vec![];
    while args.remaining() >= 1 {
        ids.push(args.single::<u64>()?);
    }
    if ids.len() == 0 {
        return Err(CommandUsageKind::DeveloperBlacklistNoIds.into());
    }

    if server {
        for id in &ids {
            let setting = ServerSettings::new(*id, &ctx.data)?;
            let mut write = setting.write();
            let blacklisted = !*(write.blacklisted());
            write.set_blacklisted(blacklisted);
        }
    } else {
        for id in &ids {
            let setting = UserSettings::new(*id, &ctx.data)?;
            let mut write = setting.write();
            let blacklisted = !*(write.blacklisted());
            write.set_blacklisted(blacklisted);
        }
    }

    msg.reply(
        &ctx,
        &format!(
            "Toggled blacklist on {} {} ID{}.",
            ids.len(),
            if server { "server" } else { "user" },
            if ids.len() == 1 { "" } else { "s" },
        ),
    )?;

    Ok(())
}

#[command]
#[owners_only]
fn evaluate(ctx: &mut Context, msg: &Message) -> CommandResult {
    use ketos::io::SharedWrite as _;
    use std::rc::Rc;

    let code_block: Vec<&str> = msg.content.splitn(3, "```").collect();
    if code_block.len() != 3 {
        return Err(CommandUsageKind::DeveloperEvaluateUsage.into());
    }
    let mut code_block = code_block[1];
    if code_block.starts_with("lisp") {
        code_block = code_block.trim_start_matches("lisp\n");
    }
    debug!("Entire code to run with ketos:\n{}", code_block);

    let output_writer = Rc::new(crate::ketoswritewrapper::KetosWriteWrapper::bytearray());
    let interpreter = ketos::Builder::new()
        .io(Rc::new(ketos::GlobalIo::new(
            Rc::clone(&output_writer) as Rc<dyn ketos::SharedWrite>,
            Rc::clone(&output_writer) as Rc<dyn ketos::SharedWrite>,
        )))
        .finish();

    let compiled = interpreter.compile_exprs(code_block)?;
    debug!("Compiled: {:?}", compiled);

    let output = interpreter
        .execute_program(compiled)
        .map(|ref v| interpreter.format_value(v))
        .map_err(|ref e| format!("{:?}", e));
    let output = match output {
        Ok(s) => s,
        Err(s) => s,
    };

    drop(interpreter); // drop the other Rcs
    output_writer.flush()?;

    let writer = Rc::try_unwrap(output_writer).ok().failure()?;
    let prints = writer.as_string()?;

    msg.reply(
        &ctx,
        &format!(
            "Interpreter returned: {}\nInterpreter printed:\n{}",
            output, prints
        ),
    )?;

    Ok(())
}
