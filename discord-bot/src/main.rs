use std::sync::Arc;

use figment::{
    providers::{Format, Toml},
    Figment,
};
use serenity::{
    framework::StandardFramework,
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

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP);

    let config: Config = Figment::new()
        .merge(Toml::file("App.toml"))
        .extract()
        .expect("Failed to parse configuration");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&config.discord_bot_token, intents)
        .event_handler(Handler)
        .framework(framework)
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
