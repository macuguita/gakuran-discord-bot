use crate::Context;
use anyhow::Result;
use poise::serenity_prelude as serenity;

fn parse_duration(input: &str) -> Option<i64> {
    // accepts things like "30m", "2h", "1d", "1h30m"
    let mut total = 0i64;
    let mut num = String::new();
    for c in input.chars() {
        if c.is_ascii_digit() {
            num.push(c);
        } else {
            let n: i64 = num.parse().ok()?;
            num.clear();
            total += match c {
                'd' => n * 86400,
                'h' => n * 3600,
                'm' => n * 60,
                's' => n,
                _ => return None,
            };
        }
    }
    if total == 0 { None } else { Some(total) }
}

/// Start a giveaway
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_GUILD",
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn giveaway(
    ctx: Context<'_>,
    #[description = "What's being given away"] prize: String,
    #[description = "How long it runs, e.g. 1h30m, 2d"] duration: String,
    #[description = "Number of winners"] winners: i64,
) -> Result<()> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.say("This command only works in a server.").await?;
        return Ok(());
    };

    let Some(seconds) = parse_duration(&duration) else {
        ctx.say("Couldn't parse that duration. Try something like `1h30m`, `2d`, or `45m`.")
            .await?;
        return Ok(());
    };

    if winners < 1 {
        ctx.say("Winner count must be at least 1.").await?;
        return Ok(());
    }

    let end_time = chrono::Utc::now().timestamp() + seconds;

    let id = crate::db::giveaway::insert_giveaway(
        &ctx.data().db,
        guild_id,
        ctx.channel_id(),
        &prize,
        winners,
        end_time,
        ctx.author().id,
    )
    .await?;

    let embed = build_giveaway_embed(&prize, winners, end_time, 0);
    let button = serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(format!("giveaway_enter_{id}"))
            .label("🎉 Enter (0)")
            .style(serenity::ButtonStyle::Primary),
    ]);

    let msg = ctx
        .channel_id()
        .send_message(
            ctx,
            serenity::CreateMessage::new()
                .embed(embed)
                .components(vec![button]),
        )
        .await?;

    crate::db::giveaway::set_giveaway_message_id(&ctx.data().db, id, msg.id).await?;

    ctx.send(
        poise::CreateReply::default()
            .content("Giveaway started!")
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

pub fn build_giveaway_embed(
    prize: &str,
    winner_count: i64,
    end_time: i64,
    entry_count: i64,
) -> serenity::CreateEmbed {
    serenity::CreateEmbed::new()
        .title(prize)
        .description(format!(
            "Winners: **{winner_count}**\nEnds: <t:{end_time}:R>\nEntries: **{entry_count}**"
        ))
        .color(0x2E_CC_71)
}
