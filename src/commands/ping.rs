use crate::Context;
use anyhow::Result;
use std::time::Instant;

/// Check bot's ping
#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_GUILD",
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn ping(ctx: Context<'_>) -> Result<()> {
    let start = Instant::now();
    let reply = ctx.say("Pong! 🏓").await?;

    let latency = start.elapsed();

    let shard_latency = ctx
        .framework()
        .shard_manager
        .runners
        .lock()
        .await
        .get(&ctx.serenity_context().shard_id)
        .and_then(|runner| runner.latency)
        .map_or_else(|| "?ms".to_string(), |d| format!("{}ms", d.as_millis()));

    reply
        .edit(
            ctx,
            poise::CreateReply::default().content(format!(
                "🏓 Pong!\nMessage round-trip: `{}ms`\nGateway latency: `{}`",
                latency.as_millis(),
                shard_latency
            )),
        )
        .await?;

    Ok(())
}
