use std::collections::HashSet;

use serenity::{
    framework::standard::{
        help_commands,
        macros::{command, group, help, hook},
        Args, CommandGroup, CommandResult, DispatchError, HelpOptions,
    },
    model::prelude::{Message, UserId},
    prelude::Context,
};

use crate::{biconomy::BiconomyClient, CommandCounter, ConfigDependency};

#[group]
#[commands(ping)]
pub struct General;

#[group]
#[commands(check_gas_tank)]
// Sets multiple prefixes for a group.
// This requires us to call commands in this group
// via `!biconomy` (or `!bc`) instead of just `!`.
#[prefixes("biconomy", "bc")]
// Set a description to appear if a user wants to display a single group
// e.g. via help using the group-name or one of its prefixes.
#[description = "To help with interacting with biconomy."]
// Summary only appears when listing multiple groups.
#[summary = "Interact with Biconomy API!"]
#[commands(check_gas_tank)]
pub struct Biconomy;

// The framework provides two built-in help commands for you to use.
// But you can also make your own customized help command that forwards
// to the behaviour of either of them.
#[help]
// This replaces the information that a user can pass
// a command-name as argument to gain specific information about it.
#[individual_command_tip = "Hello! こんにちは！Hola! Bonjour! 您好! 안녕하세요~\n\n\
If you want more information about a specific command, just pass the command as argument."]
// Some arguments require a `{}` in order to replace it with contextual information.
// In this case our `{}` refers to a command's name.
#[command_not_found_text = "Could not find: `{}`."]
// Define the maximum Levenshtein-distance between a searched command-name
// and commands. If the distance is lower than or equal the set distance,
// it will be displayed as a suggestion.
// Setting the distance to 0 will disable suggestions.
#[max_levenshtein_distance(3)]
// When you use sub-groups, Serenity will use the `indention_prefix` to indicate
// how deeply an item is indented.
// The default value is "-", it will be changed to "+".
#[indention_prefix = "+"]
// On another note, you can set up the help-menu-filter-behaviour.
// Here are all possible settings shown on all possible options.
// First case is if a user lacks permissions for a command, we can hide the command.
#[lacking_permissions = "Hide"]
// If the user is nothing but lacking a certain role, we just display it hence our variant is `Nothing`.
#[lacking_role = "Nothing"]
// The last `enum`-variant is `Strike`, which ~~strikes~~ a command.
#[wrong_channel = "Strike"]
// Serenity will automatically analyse and generate a hint/tip explaining the possible
// cases of ~~strikethrough-commands~~, but only if
// `strikethrough_commands_tip_in_{dm, guild}` aren't specified.
// If you pass in a value, it will be displayed instead.
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
pub async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    // Increment the number of times this command has been run once. If
    // the command's name does not exist in the counter, add a default
    // value of 0.
    let mut data = ctx.data.write().await;
    let counter = data
        .get_mut::<CommandCounter>()
        .expect("Expected CommandCounter in TypeMap.");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true // if `before` returns false, command processing doesn't happen.
}

#[hook]
pub async fn after(
    _ctx: &Context,
    _msg: &Message,
    command_name: &str,
    command_result: CommandResult,
) {
    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
pub async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
pub async fn normal_message(_ctx: &Context, msg: &Message) {
    println!("Message is not a command '{}'", msg.content);
}

#[hook]
pub async fn delay_action(ctx: &Context, msg: &Message) {
    // You may want to handle a Discord rate limit if this fails.
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
pub async fn dispatch_error(
    ctx: &Context,
    msg: &Message,
    error: DispatchError,
    _command_name: &str,
) {
    if let DispatchError::Ratelimited(info) = error {
        // We notify them only once.
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!("Try this again in {} seconds.", info.as_secs()),
                )
                .await;
        }
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
#[description = "Checks the gas tank balance of a Dapp.\nUse with `!biconomy check_gas_tank <DAPP_API_KEY>`"]
#[allowed_roles("The Crew")]
async fn check_gas_tank(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let api_key = match args.single::<String>() {
        Ok(data) => data,
        Err(_) => {
            msg.reply(ctx, "API_KEY is not provided").await?;
            return Ok(());
        }
    };
    let data_read = ctx.data.read().await;
    let config = data_read
        .get::<ConfigDependency>()
        .expect("Expected ConfigDependency in TypeMap.")
        .clone();
    let client = BiconomyClient::new(&config.biconomy_auth_token);
    let resp = client.gas_tank_balance(&api_key).await;
    match resp {
        Ok(resp) => {
            msg.reply(
                ctx,
                format!(
                    "**{}** is left in Dapp *{}*. (Grace Period: **{}**)",
                    resp.dapp_gas_tank_data.effective_balance_in_standard_form,
                    api_key,
                    resp.dapp_gas_tank_data.is_in_grace_period
                ),
            )
            .await?
        }
        Err(resp) => {
            msg.reply(ctx, format!("**{}: {}**", resp.code, resp.message))
                .await?
        }
    };

    Ok(())
}
