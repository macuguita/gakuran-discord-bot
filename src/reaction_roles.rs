use poise::serenity_prelude as serenity;

pub async fn handle_reaction_add(
    ctx: &serenity::Context,
    data: &crate::Data,
    add_reaction: &serenity::Reaction,
) -> Result<(), crate::Error> {
    let key = format!("{}:{}", add_reaction.message_id, add_reaction.emoji);

    let role_id = if let Some(guild_id) = add_reaction.guild_id {
        crate::db::get_reaction_role(&data.db, guild_id, &key).await?
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
        let role_id = crate::db::get_reaction_role(&data.db, guild_id, &key).await?;

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
