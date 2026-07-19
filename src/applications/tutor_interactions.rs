use crate::db::tutor_application::{get_application, set_application_status};
use anyhow::Result;
use poise::serenity_prelude as serenity;

pub async fn handle_component(
    ctx: &serenity::Context,
    data: &crate::Data,
    component: &serenity::ComponentInteraction,
) -> Result<()> {
    let custom_id = component.data.custom_id.as_str();

    let (action, app_id) = if let Some(id) = custom_id.strip_prefix("tutor_app_accept_") {
        ("accept", id)
    } else if let Some(id) = custom_id.strip_prefix("tutor_app_deny_") {
        ("deny", id)
    } else {
        return Ok(());
    };

    let Ok(app_id) = app_id.parse::<i64>() else {
        return Ok(());
    };
    let Some(guild_id) = component.guild_id else {
        return Ok(());
    };

    let Some(app) = get_application(&data.db, app_id).await? else {
        return Ok(());
    };

    if app.status != "pending" {
        component
            .create_response(
                ctx,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("This application has already been reviewed.")
                        .ephemeral(true),
                ),
            )
            .await?;
        return Ok(());
    }

    let applicant_id: u64 = app.user_id.parse().unwrap_or_default();
    let applicant = serenity::UserId::new(applicant_id);

    if action == "accept" {
        set_application_status(&data.db, app_id, "accepted", component.user.id).await?;

        let cfg = crate::db::appconfig::get_app_config(&data.db, guild_id).await?;
        if let Some(role_id) = cfg.and_then(|c| c.tutor_accepted_role)
            && let Ok(member) = guild_id.member(ctx, applicant).await
        {
            let _ = member.add_role(ctx, role_id).await;
        }

        if let Ok(dm) = applicant.create_dm_channel(ctx).await {
            let _ = dm
                .send_message(
                    ctx,
                    serenity::CreateMessage::new().content(format!(
                        "Your tutor application (#{app_id}) has been accepted! Welcome in."
                    )),
                )
                .await;
        }
    } else {
        set_application_status(&data.db, app_id, "denied", component.user.id).await?;

        if let Ok(dm) = applicant.create_dm_channel(ctx).await {
            let _ = dm
                .send_message(
                    ctx,
                    serenity::CreateMessage::new().content(format!(
                        "Your tutor application (#{app_id}) was not accepted this time."
                    )),
                )
                .await;
        }
    }

    let status_text = if action == "accept" {
        "✅ Accepted"
    } else {
        "❌ Denied"
    };

    let mut edit = serenity::EditMessage::new()
        .content(format!("{status_text} by <@{}>", component.user.id))
        .components(vec![]);

    for attachment in &component.message.attachments {
        edit = edit.keep_existing_attachment(attachment.id);
    }

    component
        .channel_id
        .edit_message(ctx, component.message.id, edit)
        .await?;

    component
        .create_response(ctx, serenity::CreateInteractionResponse::Acknowledge)
        .await?;

    Ok(())
}
