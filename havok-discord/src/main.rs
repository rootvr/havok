use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::StandardFramework;
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::Context;
use serenity::prelude::EventHandler;
use serenity::prelude::GatewayIntents;
use serenity::prelude::Mutex;
use serenity::prelude::TypeMapKey;
use serenity::Client;
use std::collections::HashSet;
use std::env;
use std::sync::Arc;
use tracing::info;
use tracing_unwrap::ResultExt;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as `{}`", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[group]
#[commands(ping)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    info!("Received: `{:?}`", msg.content);
    msg.channel_id.say(&ctx.http, "pong!").await?;
    Ok(())
}

#[tokio::main]
#[tracing::instrument]
async fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_ansi(true)
        .init();

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
        .expect_or_log("Could not access app info: ");

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("/"))
        .group(&GENERAL_GROUP);

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

    client.start().await.expect_or_log("Client error: ");
}
