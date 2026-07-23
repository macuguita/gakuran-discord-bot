mod applications;
mod auto_delete;
mod commands;
mod config;
mod db;
mod embed;
mod giveaway;
mod mod_log;
mod reaction_roles;
mod auto_role;

use anyhow::Result;
use config::Config;
use poise::serenity_prelude as serenity;
use std::collections::HashMap;

type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

pub struct Data {
    pub db: sqlx::SqlitePool,
    pub embed_drafts: tokio::sync::Mutex<HashMap<serenity::UserId, embed::EmbedDraft>>,
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, anyhow::Error>,
    data: &Data,
) -> Result<()> {
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
            applications::interactions::handle_component(ctx, data, component).await?;
            applications::tutor_interactions::handle_component(ctx, data, component).await?;
            giveaway::interaction::handle_component(ctx, data, component).await?;
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
        serenity::FullEvent::Message { new_message } => {
            auto_delete::handle_message(ctx, data, new_message).await?;
        }
        serenity::FullEvent::GuildMemberAddition { new_member } => {
            auto_role::handle_new_member(ctx, data, new_member).await?;
        }
        _ => {}
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env();
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let db_pool = db::init("gakuran-bot.db").await?;
    let db_for_task = db_pool.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::ping(),
                commands::reactionrole(),
                commands::embed(),
                commands::apply(),
                commands::appconfig(),
                commands::autodelete_add(),
                commands::autodelete_remove(),
                commands::applytutor(),
                commands::giveaway(),
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
            let db_pool = db_pool.clone();
            Box::pin(async move {
                // let guild_id = serenity::GuildId::new(1525578372367777945);
                // poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id)
                //     .await?;
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    db: db_pool,
                    embed_drafts: tokio::sync::Mutex::new(HashMap::new()),
                })
            })
        })
        .build();

    let mut cache_settings = serenity::cache::Settings::default();
    cache_settings.max_messages = 1000;

    let mut client = serenity::ClientBuilder::new(config.token, intents)
        .framework(framework)
        .cache_settings(cache_settings)
        .await?;
    let http = client.http.clone();

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            if let Err(e) = giveaway::process_due_giveaways(&http, &db_for_task).await {
                eprintln!("Error processing giveaways: {e:?}");
            }
        }
    });

    client.start().await?;

    Ok(())
}
