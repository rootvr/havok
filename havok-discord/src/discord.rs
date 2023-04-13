mod command;
mod handler;

use self::command::alias::AliasContainer;
use self::command::alias::All;
use handler::*;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::StandardFramework;
use serenity::http::Http;
use serenity::model::prelude::Message;
use serenity::prelude::Context;
use serenity::prelude::GatewayIntents;
use serenity::prelude::Mutex;
use serenity::prelude::TypeMapKey;
use serenity::Client;
use serenity::FutureExt;
use std::collections::HashSet;
use std::env;
use std::sync::Arc;
use tokio::signal::unix::signal;
use tokio::signal::unix::SignalKind;
use tracing::warn;
use tracing_unwrap::OptionExt;
use tracing_unwrap::ResultExt;

// TODO(resu): Make this dynamic
const PREFIX_SIGIL: &str = "!";

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[inline]
pub async fn send_msg(ctx: &Context, msg: &Message, txt: &str) -> Result<Message, serenity::Error> {
    msg.reply_ping(ctx, txt).await
}

pub async fn run() {
    dotenv::dotenv().expect_or_log("No `.env` file");

    let token = env::var("DISCORD_TOKEN").expect_or_log("Env var `DISCORD_TOKEN`");

    let http = Http::new(&token);

    let (owners, _bot_id) = http
        .get_current_application_info()
        .await
        .map(|info| {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            (owners, info.id)
        })
        .expect_or_log("Could not access app info");

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix(PREFIX_SIGIL))
        .group(&command::meta::META_GROUP)
        .group(&command::owner::OWNER_GROUP)
        .group(&command::havok::HAVOK_GROUP)
        .group(&command::alias::ALIAS_GROUP);

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect_or_log("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<AliasContainer>(All::new());
    }

    let data = client.data.clone();

    let shard_manager = client.shard_manager.clone();

    let handle_client = async {
        client.start().await.expect_or_log("Client error");
    };

    let handle_ctrlc = async {
        tokio::signal::ctrl_c()
            .await
            .expect_or_log("Could not register `Ctrl+C` handler");
        warn!("Received `Ctrl-C`, shutting down...");
        let data = data.read().await;
        let _ = data.get::<AliasContainer>().unwrap_or_log();
        // TODO(resu): persist aliases
        shard_manager.lock().await.shutdown_all().await;
    };

    #[cfg(unix)]
    let handle_sigterm = async {
        signal(SignalKind::terminate())
            .expect_or_log("Could not register `SIGTERM` handler")
            .recv()
            .await;
        warn!("Received `SIGTERM`, shutting down...");
        let data = data.read().await;
        let _ = data.get::<AliasContainer>().unwrap_or_log();
        // TODO(resu): persist aliases
        shard_manager.lock().await.shutdown_all().await;
    };

    let all_futures = vec![
        #[cfg(unix)]
        handle_sigterm.boxed(),
        handle_client.boxed(),
        handle_ctrlc.boxed(),
    ];

    futures::future::select_all(all_futures).await;
}
