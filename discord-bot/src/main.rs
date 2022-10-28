use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use commands::{
    after, before, dispatch_error, normal_message, unknown_command, BICONOMY_GROUP, MY_HELP,
};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use serenity::{
    framework::StandardFramework,
    http::Http,
    prelude::{GatewayIntents, TypeMapKey},
    Client,
};

use crate::commands::GENERAL_GROUP;
use crate::config::Config;
use crate::handler::Handler;

mod biconomy;
mod commands;
mod config;
mod handler;

pub struct ConfigDependency;
impl TypeMapKey for ConfigDependency {
    type Value = Arc<Config>;
}

pub struct CommandCounter;
impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

#[tokio::main]
async fn main() {
    let config: Config = Figment::new()
        .merge(Toml::file("App.toml"))
        .extract()
        .expect("Failed to parse configuration");

    let http = Http::new(&config.discord_bot_token);

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .on_mention(Some(bot_id))
                .prefix("!")
                // Sets the bot's owners. These will be used for commands that
                // are owners only.
                .owners(owners)
        })
        .before(before)
        .after(after)
        .unrecognised_command(unknown_command)
        .normal_message(normal_message)
        .on_dispatch_error(dispatch_error)
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&BICONOMY_GROUP);

    let intents = GatewayIntents::all();

    let mut client = Client::builder(&config.discord_bot_token, intents)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<CommandCounter>(HashMap::default())
        .await
        .expect("Failed to create discord client");

    {
        // Open the data lock in write mode, so keys can be inserted to it.
        let mut data = client.data.write().await;

        data.insert::<ConfigDependency>(Arc::new(config));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
