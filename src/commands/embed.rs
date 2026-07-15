use crate::embed::{EmbedDraft, buttons::build_button_rows};
use crate::{Data, Error};
use poise::Modal;

#[derive(Debug, poise::Modal)]
struct EmbedBasicsModal {
    title: Option<String>,
    #[paragraph]
    description: Option<String>,
    #[name = "Color (hex, e.g. FF5733)"]
    color: Option<String>,
    url: Option<String>,
    footer: Option<String>,
}

/// Open a builder to construct an embedded message
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_MESSAGES",
    default_member_permissions = "MANAGE_MESSAGES"
)]
pub async fn embed(ctx: poise::ApplicationContext<'_, Data, Error>) -> Result<(), Error> {
    let data = EmbedBasicsModal::execute(ctx).await?;
    let Some(data) = data else {
        return Ok(());
    };

    let draft = EmbedDraft {
        title: data.title,
        description: data.description,
        color: data
            .color
            .and_then(|c| u32::from_str_radix(c.trim_start_matches('#'), 16).ok()),
        url: data.url,
        footer: data.footer,
        ..Default::default()
    };

    ctx.data
        .embed_drafts
        .lock()
        .await
        .insert(ctx.interaction.user.id, draft.clone());

    ctx.send(
        poise::CreateReply::default()
            .embed(draft.to_embed())
            .components(build_button_rows())
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
