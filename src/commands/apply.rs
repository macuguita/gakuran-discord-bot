use crate::{Data, Error};
use poise::Modal;
use poise::serenity_prelude as serenity;

#[derive(Debug, Modal)]
struct ApplicationModal {
    roblox_username: String,
    #[name = "What is your in-game character's name?"]
    in_game_name: String,
    #[name = "What is your real age?"]
    age: String,
    #[name = "Do you mainly use voice chat and text chat?"]
    uses_vc: String,
    #[name = "How often are you available to roleplay?"]
    roleplay_availability: String,
}

#[poise::command(slash_command)]
pub async fn apply(ctx: poise::ApplicationContext<'_, Data, Error>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        ctx.send(
            poise::CreateReply::default()
                .content("This command can only be used inside of a server.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };

    if crate::db::has_pending_application(&ctx.data.db, guild_id, ctx.interaction.user.id).await? {
        ctx.send(
            poise::CreateReply::default()
                .content("You already have a pending application.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let cfg = crate::db::get_app_config(&ctx.data.db, guild_id).await?;
    let Some(channel_id) = cfg.and_then(|c| c.response_channel) else {
        ctx.send(
            poise::CreateReply::default()
                .content("Applications have not been set up for this server yet.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };

    let data = ApplicationModal::execute(ctx).await?;
    let Some(data) = data else {
        return Ok(());
    };

    let app_id = crate::db::insert_application(
        &ctx.data.db,
        guild_id,
        ctx.interaction.user.id,
        &data.roblox_username,
        &data.in_game_name,
        &data.age,
        &data.uses_vc,
        &data.roleplay_availability,
    )
    .await?;

    let author = &ctx.interaction.user;

    let embed = serenity::CreateEmbed::default()
        .title(author.name.clone())
        .field("Discord user", format!("<@{}>", author.id), false)
        .field("Roblox username", &data.roblox_username, false)
        .field("In game name", &data.in_game_name, false)
        .field("Real age", &data.age, false)
        .field("Uses voice chat or text chat", &data.uses_vc, false)
        .field("Roleplay availability", &data.roleplay_availability, false)
        .footer(serenity::CreateEmbedFooter::new(format!(
            "Application #{app_id}"
        )))
        .color(0x58_65_F2);

    let buttons = serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(format!("app_accept_{app_id}"))
            .label("Accept")
            .style(serenity::ButtonStyle::Success),
        serenity::CreateButton::new(format!("app_deny_{app_id}"))
            .label("Deny")
            .style(serenity::ButtonStyle::Danger),
    ]);

    channel_id
        .send_message(
            ctx.http(),
            serenity::CreateMessage::default()
                .embed(embed)
                .components(vec![buttons]),
        )
        .await?;

    ctx.send(
        poise::CreateReply::default()
            .content("Your application has been submitted!")
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
