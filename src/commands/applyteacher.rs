use crate::{Data, Error};
use poise::Modal;
use poise::serenity_prelude as serenity;

#[derive(Debug, poise::Modal)]
struct TeacherApplicationModal {
    roblox_username: String,
    in_game_name: String,
    #[name = "Do you use voice chat"]
    uses_voice_chat: String,
    #[paragraph]
    #[name = "What subject(s) would you like to teach"]
    subjects: String,
    #[paragraph]
    #[name = "How often are you available to roleplay"]
    roleplay_times: String,
}

#[poise::command(slash_command)]
pub async fn applyteacher(ctx: poise::ApplicationContext<'_, Data, Error>) -> Result<(), Error> {
    let data = TeacherApplicationModal::execute(ctx).await?;
    let Some(data) = data else {
        return Ok(());
    };

    let Some(guild_id) = ctx.guild_id() else {
        ctx.send(
            poise::CreateReply::default()
                .content("This command can only be used inside of a guild.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };

    let channel_id = {
        let cfg = ctx.data().application_responses.lock().await;
        cfg.get(&guild_id).copied()
    };

    let Some(channel_id) = channel_id else {
        ctx.send(
            poise::CreateReply::default()
                .content("Teacher applications have not been setup for this server.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    };

    let author_name = ctx.author().name.clone();

    channel_id
        .send_message(
            ctx.http(),
            serenity::CreateMessage::default().embed(create_embed(data, &author_name)),
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

fn create_embed(data: TeacherApplicationModal, author_name: &str) -> serenity::CreateEmbed {
    serenity::CreateEmbed::default()
        .title(author_name)
        .field("Roblox username", data.roblox_username, false)
        .field("In game name", data.in_game_name, false)
        .field("Uses voice chat", data.uses_voice_chat, false)
        .field("Subjects", data.subjects, false)
        .field("Roleplay times", data.roleplay_times, false)
}
