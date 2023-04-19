mod handler;
use handler::Handler;

pub(crate) mod map;
use map::ShardManagerMap;

pub(crate) mod utils;
use utils::PREFIX_SIGIL;

pub(crate) mod help;
use help::after;
use help::before;
use help::dispatch_error;
use help::unknown_command;
use help::MY_HELP;

use crate::command::alias::map::AliasMap;
use crate::command::alias::model::AliasContainer;
use crate::command::alias::ALIAS_GROUP;
use crate::command::meta::META_GROUP;
use crate::command::roll::map::RollMap;
use crate::command::roll::ROLL_GROUP;
use serenity::framework::standard::StandardFramework;
use serenity::http::Http;
use serenity::prelude::GatewayIntents;
use serenity::Client;
use serenity::FutureExt;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use tokio::signal::unix::signal;
use tokio::signal::unix::SignalKind;
use tracing::warn;
use tracing_unwrap::OptionExt;
use tracing_unwrap::ResultExt;

pub(crate) async fn run() {
    dotenv::dotenv().expect_or_log("No `.env` file");

    let token = env::var("DISCORD_TOKEN").expect_or_log("Env var `DISCORD_TOKEN`");

    let http = Http::new(&token);

    let (owners, bot_id) = {
        let info = http
            .get_current_application_info()
            .await
            .expect_or_log("Could not access app info");
        let mut owners = HashSet::new();
        owners.insert(if let Some(team) = info.team {
            team.owner_user_id
        } else {
            info.owner.id
        });
        let bot = http
            .get_current_user()
            .await
            .expect_or_log("Could not access bot id");
        (owners, bot.id)
    };

    let framework = StandardFramework::new()
        .configure(|config| {
            config
                .with_whitespace(true)
                .on_mention(Some(bot_id))
                .prefix(PREFIX_SIGIL)
                .delimiters(vec![" "])
                .owners(owners)
        })
        .before(before)
        .after(after)
        .unrecognised_command(unknown_command)
        .on_dispatch_error(dispatch_error)
        .help(&MY_HELP)
        .group(&META_GROUP)
        .group(&ROLL_GROUP)
        .group(&ALIAS_GROUP);

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
        data.insert::<ShardManagerMap>(client.shard_manager.clone());
        data.insert::<AliasMap>(AliasContainer::new());
        data.insert::<RollMap>(HashMap::new());
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
        let all = data.get::<AliasMap>().unwrap_or_log();
        all.save_all();
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
        let all = data.get::<AliasMap>().unwrap_or_log();
        all.save_all();
        shard_manager.lock().await.shutdown_all().await;
    };

    let handlers = vec![
        #[cfg(unix)]
        handle_sigterm.boxed(),
        handle_client.boxed(),
        handle_ctrlc.boxed(),
    ];

    futures::future::select_all(handlers).await;
}
