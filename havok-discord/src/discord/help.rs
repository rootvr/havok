use serenity::framework::standard::help_commands;
use serenity::framework::standard::macros::help;
use serenity::framework::standard::macros::hook;
use serenity::framework::standard::Args;
use serenity::framework::standard::CommandGroup;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::DispatchError;
use serenity::framework::standard::HelpOptions;
use serenity::model::channel::Message;
use serenity::model::prelude::UserId;
use serenity::prelude::Context;
use std::collections::HashSet;
use tracing::error;
use tracing::info;
use tracing::warn;

#[help]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Nothing"]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
pub(crate) async fn before(_ctx: &Context, msg: &Message, command: &str) -> bool {
    info!("Got command `{}` by user `{}`", command, msg.author.name);
    true
}

#[hook]
pub(crate) async fn after(_ctx: &Context, _msg: &Message, command: &str, result: CommandResult) {
    match result {
        Ok(()) => info!("Processed command `{}`", command),
        Err(why) => error!("Command `{}` returned error `{:?}`", command, why),
    }
}

#[hook]
pub(crate) async fn unknown_command(_ctx: &Context, _msg: &Message, unknown: &str) {
    warn!("Could not find command named `{}`", unknown);
}

#[hook]
pub(crate) async fn dispatch_error(
    ctx: &Context,
    msg: &Message,
    error: DispatchError,
    _command: &str,
) {
    if let DispatchError::Ratelimited(info) = error {
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!("**info** *try again after {}s*", info.as_secs()),
                )
                .await;
        }
    }
}
