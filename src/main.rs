mod commands;
mod config;
mod embed;
mod mod_log;
mod reaction_roles;

use anyhow::Result;
use config::Config;
use poise::serenity_prelude as serenity;
use std::collections::HashMap;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub type ChannelConfig = HashMap<serenity::GuildId, serenity::ChannelId>;

pub struct Data {
    pub reaction_roles: tokio::sync::Mutex<reaction_roles::ReactionRoles>,
    pub embed_drafts: tokio::sync::Mutex<HashMap<serenity::UserId, embed::EmbedDraft>>,
    pub mod_log: tokio::sync::Mutex<mod_log::ModLogConfig>,
    pub application_responses: tokio::sync::Mutex<ChannelConfig>,
}

/// # Errors
// Returns an error when file cannot be written
pub fn save_channel_config(data: &ChannelConfig, file_name: &str) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    std::fs::write(file_name, json)?;
    Ok(())
}

#[must_use]
pub fn load_channel_config(file_name: &str) -> ChannelConfig {
    std::fs::read_to_string(file_name)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::ReactionAdd { add_reaction } => {
            reaction_roles::handle_reaction_add(ctx, data, add_reaction).await?;
        }
        serenity::FullEvent::ReactionRemove { removed_reaction } => {
            reaction_roles::handle_reaction_remove(ctx, data, removed_reaction).await?;
        }
        serenity::FullEvent::InteractionCreate {
            interaction: serenity::Interaction::Component(component),
        } => {
            embed::interactions::handle_component(ctx, data, component).await?;
        }
        serenity::FullEvent::MessageDelete {
            channel_id,
            deleted_message_id,
            guild_id,
        } => {
            mod_log::handle_message_delete(ctx, data, *channel_id, *deleted_message_id, *guild_id)
                .await?;
        }
        serenity::FullEvent::MessageUpdate {
            old_if_available,
            new,
            event,
        } => {
            mod_log::handle_message_update(
                ctx,
                data,
                old_if_available.as_ref(),
                new.as_ref(),
                event.guild_id,
            )
            .await?;
        }
        _ => {}
    }
    Ok(())
}

#[tokio::main]
#[allow(clippy::unreadable_literal)]
async fn main() -> Result<()> {
    let config = Config::from_env();
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::ping(),
                commands::reactionrole(),
                commands::embed(),
                commands::setmodlog(),
                commands::setapplicationresponses(),
                commands::applyteacher(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                // let guild_id = serenity::GuildId::new(1525578372367777945);
                // poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id)
                //     .await?;
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    reaction_roles: tokio::sync::Mutex::new(reaction_roles::load()),
                    embed_drafts: tokio::sync::Mutex::new(HashMap::new()),
                    mod_log: tokio::sync::Mutex::new(mod_log::load()),
                    application_responses: tokio::sync::Mutex::new(load_channel_config(
                        "application_responses.json",
                    )),
                })
            })
        })
        .build();

    let mut cache_settings = serenity::cache::Settings::default();
    cache_settings.max_messages = 1000;

    let client = serenity::ClientBuilder::new(config.token, intents)
        .framework(framework)
        .cache_settings(cache_settings)
        .await;
    client.unwrap().start().await.unwrap();
    Ok(())
}
