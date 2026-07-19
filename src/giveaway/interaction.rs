use anyhow::Result;
use poise::serenity_prelude as serenity;

pub async fn handle_component(
    ctx: &serenity::Context,
    data: &crate::Data,
    component: &serenity::ComponentInteraction,
) -> Result<()> {
    let Some(id_str) = component.data.custom_id.strip_prefix("giveaway_enter_") else {
        return Ok(());
    };
    let Ok(giveaway_id) = id_str.parse::<i64>() else {
        return Ok(());
    };

    let now_in =
        crate::db::giveaway::toggle_entry(&data.db, giveaway_id, component.user.id).await?;
    let count = crate::db::giveaway::count_entries(&data.db, giveaway_id).await?;

    let label = format!("🎉 Enter ({count})");
    let updated_button = serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(component.data.custom_id.clone())
            .label(label)
            .style(serenity::ButtonStyle::Primary),
    ]);

    let status_msg = if now_in {
        "You're entered!"
    } else {
        "You've left the giveaway."
    };
    component
        .create_response(
            ctx,
            serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new()
                    .content(status_msg)
                    .ephemeral(true),
            ),
        )
        .await?;

    // Update the original message's button label to reflect the new count
    component
        .channel_id
        .edit_message(
            ctx,
            component.message.id,
            serenity::EditMessage::new().components(vec![updated_button]),
        )
        .await?;

    Ok(())
}
