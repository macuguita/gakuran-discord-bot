use poise::serenity_prelude as serenity;
use std::collections::HashMap;

pub type ReactionRoles = HashMap<serenity::GuildId, HashMap<String, u64>>;

pub fn save(data: &ReactionRoles) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    std::fs::write("reaction_roles.json", json)?;
    Ok(())
}

pub fn load() -> ReactionRoles {
    std::fs::read_to_string("reaction_roles.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub async fn handle_reaction_add(
    ctx: &serenity::Context,
    data: &crate::Data,
    add_reaction: &serenity::Reaction,
) -> Result<(), crate::Error> {
    let key = format!("{}:{}", add_reaction.message_id, add_reaction.emoji);

    let role_id = if let Some(guild_id) = add_reaction.guild_id {
        let rr = data.reaction_roles.lock().await;
        rr.get(&guild_id).and_then(|inner| inner.get(&key)).copied()
    } else {
        None
    };

    if let (Some(role_id), Some(guild_id), Some(user_id)) =
        (role_id, add_reaction.guild_id, add_reaction.user_id)
    {
        guild_id
            .member(ctx, user_id)
            .await?
            .add_role(ctx, serenity::RoleId::new(role_id))
            .await?;
    }
    Ok(())
}

pub async fn handle_reaction_remove(
    ctx: &serenity::Context,
    data: &crate::Data,
    removed_reaction: &serenity::Reaction,
) -> Result<(), crate::Error> {
    let key = format!("{}:{}", removed_reaction.message_id, removed_reaction.emoji);

    if let (Some(guild_id), Some(user_id)) = (removed_reaction.guild_id, removed_reaction.user_id) {
        let role_id = {
            let rr = data.reaction_roles.lock().await;
            rr.get(&guild_id).and_then(|inner| inner.get(&key)).copied()
        };
        if let Some(role_id) = role_id {
            guild_id
                .member(ctx, user_id)
                .await?
                .remove_role(ctx, serenity::RoleId::new(role_id))
                .await?;
        }
    }
    Ok(())
}
