use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub discord_bot_token: String,
    pub biconomy_auth_token: String,
}
