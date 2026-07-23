use anyhow::Result;
use poise::serenity_prelude as serenity;

pub async fn handle_new_member(
    ctx: &serenity::Context,
    data: &crate::Data,
    new_member: &serenity::Member,
) -> Result<()> {
    if new_member.user.bot {
        return Ok(());
    }
    let guild_id = new_member.guild_id;
    let Some(cfg) = crate::db::appconfig::get_app_config(&data.db, guild_id).await? else {
        return Ok(());
    };
    let Some(auto_role) = cfg.auto_role else {
        return Ok(());
    };

    new_member.add_role(&ctx.http, auto_role).await?;

    Ok(())
}