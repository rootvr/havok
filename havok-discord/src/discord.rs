mod handler;
mod havok;
mod meta;
mod owner;

use handler::*;
use havok::*;
use meta::*;
use owner::*;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::StandardFramework;
use serenity::http::Http;
use serenity::model::prelude::Message;
use serenity::prelude::Context;
use serenity::prelude::GatewayIntents;
use serenity::prelude::Mutex;
use serenity::prelude::TypeMapKey;
use serenity::Client;
use std::collections::HashSet;
use std::env;
use std::sync::Arc;
use tracing_unwrap::ResultExt;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[inline]
pub async fn send(ctx: &Context, msg: &Message, txt: &str) -> Result<Message, serenity::Error> {
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
        .configure(|c| c.owners(owners).prefix("!"))
        .group(&META_GROUP)
        .group(&OWNER_GROUP)
        .group(&HAVOK_GROUP);

    let intents = GatewayIntents::GUILD_MESSAGES
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
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect_or_log("Could not register Ctrl+C handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    client.start().await.expect_or_log("Client error");
}
