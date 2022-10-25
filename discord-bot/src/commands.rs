use serenity::{
    framework::standard::{
        macros::{command, group},
        CommandResult,
    },
    model::prelude::Message,
    prelude::Context,
};

use crate::{biconomy::BiconomyClient, ConfigDependency};

#[group]
#[commands(ping, check_gas_tank)]
pub struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
async fn check_gas_tank(ctx: &Context, msg: &Message) -> CommandResult {
    let data_read = ctx.data.read().await;
    let config = data_read
        .get::<ConfigDependency>()
        .expect("Expected ConfigDependency in TypeMap.")
        .clone();
    let client = BiconomyClient::new(config.biconomy_auth_token.clone());
    let resp = client
        .gas_tank_balance(config.biconomy_auth_token.clone())
        .await;
    msg.reply(ctx, resp).await?;

    Ok(())
}
