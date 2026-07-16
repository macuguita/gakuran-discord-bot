use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Post a reaction-role message, or attach one to an existing message
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_ROLES",
    default_member_permissions = "MANAGE_ROLES"
)]
pub async fn reactionrole(
    ctx: Context<'_>,
    #[description = "Role to give"] role: serenity::Role,
    #[description = "Emoji to react with"] emoji: String,
    #[description = "Content for a new message"] message_content: Option<String>,
    #[description = "ID of an existing message to react to instead"] message_id: Option<String>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let msg = match (message_content, message_id) {
        (Some(_), Some(_)) => {
            ctx.say("Please provide either `message_content` or `message_id`, not both.")
                .await?;
            return Ok(());
        }
        (None, None) => {
            ctx.channel_id()
                .say(
                    ctx,
                    format!("React with {emoji} to get the {} role!", role.name),
                )
                .await?
        }
        (Some(content), None) => ctx.channel_id().say(ctx, content).await?,
        (None, Some(id_str)) => {
            let msg_id: u64 = id_str.parse().map_err(|_| "Invalid message ID")?;
            ctx.channel_id()
                .message(ctx, serenity::MessageId::new(msg_id))
                .await?
        }
    };

    msg.react(ctx, serenity::ReactionType::try_from(emoji.as_str())?)
        .await?;

    let key = format!("{}:{}", msg.id, emoji);
    if let Some(guild_id) = ctx.guild_id() {
        crate::db::set_reaction_role(&ctx.data().db, guild_id, &key, role.id.get()).await?;
    }

    ctx.say("Done!").await?;
    Ok(())
}
