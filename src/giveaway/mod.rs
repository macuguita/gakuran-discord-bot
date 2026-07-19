pub mod interaction;

use anyhow::Result;
use poise::serenity_prelude as serenity;
use rand::prelude::*;
use sqlx::SqlitePool;
use std::sync::Arc;

pub async fn process_due_giveaways(http: &Arc<serenity::Http>, db: &SqlitePool) -> Result<()> {
    let now = chrono::Utc::now().timestamp();
    let due = crate::db::giveaway::get_due_giveaways(db, now).await?;

    for g in due {
        let Some(message_id_str) = &g.message_id else {
            continue;
        };
        let Ok(message_id) = message_id_str.parse::<u64>() else {
            continue;
        };
        let Ok(channel_id) = g.channel_id.parse::<u64>() else {
            continue;
        };
        let channel_id = serenity::ChannelId::new(channel_id);
        let message_id = serenity::MessageId::new(message_id);

        let entries = crate::db::giveaway::get_entries(db, g.id).await?;
        let winners: Vec<String> = {
            let mut rng = rand::rng();
            let winner_count = usize::try_from(g.winner_count).unwrap_or(1);
            entries.iter().cloned().sample(&mut rng, winner_count)
        };

        let winners_text = if winners.is_empty() {
            "No one entered this giveaway.".to_string()
        } else {
            winners
                .iter()
                .map(|id| format!("<@{id}>"))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let embed = serenity::CreateEmbed::new()
            .title(&g.prize)
            .description(format!("Winners: {winners_text}"))
            .color(0xED_42_45);

        let _ = channel_id
            .edit_message(
                http,
                message_id,
                serenity::EditMessage::new()
                    .content("🎉 Giveaway ended! 🎉")
                    .embed(embed)
                    .components(vec![]),
            )
            .await;

        crate::db::giveaway::mark_giveaway_ended(db, g.id).await?;
    }

    Ok(())
}
