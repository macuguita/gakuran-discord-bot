use std::env;

#[derive(Debug)]
pub struct Config {
    pub token: String,
}

impl Config {
    pub fn from_env() -> Self {
        let token = env::var("DISCORD_TOKEN").expect("Did not provide a discord bot token");

        Self { token }
    }
}
