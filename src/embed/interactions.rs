use super::buttons::build_button_rows;
use poise::serenity_prelude as serenity;

#[allow(clippy::too_many_lines)]
pub async fn handle_component(
    ctx: &serenity::Context,
    data: &crate::Data,
    component: &serenity::ComponentInteraction,
) -> Result<(), crate::Error> {
    match component.data.custom_id.as_str() {
        "embed_post" => {
            let draft = {
                let mut drafts = data.embed_drafts.lock().await;
                drafts.remove(&component.user.id)
            };
            if let Some(draft) = draft {
                let builder = serenity::CreateMessage::new().embed(draft.to_embed());
                component.channel_id.send_message(ctx, builder).await?;
                component
                    .create_response(
                        ctx,
                        serenity::CreateInteractionResponse::UpdateMessage(
                            serenity::CreateInteractionResponseMessage::new()
                                .content("Posted!")
                                .components(vec![]),
                        ),
                    )
                    .await?;
            }
        }
        "embed_cancel" => {
            data.embed_drafts.lock().await.remove(&component.user.id);
            component
                .create_response(
                    ctx,
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .content("Cancelled.")
                            .embeds(vec![])
                            .components(vec![]),
                    ),
                )
                .await?;
        }
        "embed_add_field" => {
            let modal = serenity::CreateQuickModal::new("Add Field")
                .field(
                    serenity::CreateInputText::new(
                        serenity::InputTextStyle::Short,
                        "Field name",
                        "field_name",
                    )
                    .required(true),
                )
                .field(
                    serenity::CreateInputText::new(
                        serenity::InputTextStyle::Paragraph,
                        "Field value",
                        "field_value",
                    )
                    .required(true),
                )
                .field(
                    serenity::CreateInputText::new(
                        serenity::InputTextStyle::Short,
                        "Inline? (yes/no)",
                        "field_inline",
                    )
                    .required(false),
                );

            if let Some(response) = component.quick_modal(ctx, modal).await? {
                let inputs = response.inputs;
                let name = inputs[0].clone();
                let value = inputs[1].clone();
                let inline = inputs[2].eq_ignore_ascii_case("yes");

                let mut drafts = data.embed_drafts.lock().await;
                if let Some(draft) = drafts.get_mut(&component.user.id) {
                    draft.fields.push((name, value, inline));
                    response
                        .interaction
                        .create_response(
                            ctx,
                            serenity::CreateInteractionResponse::UpdateMessage(
                                serenity::CreateInteractionResponseMessage::new()
                                    .embed(draft.to_embed())
                                    .components(build_button_rows()),
                            ),
                        )
                        .await?;
                }
            }
        }
        "embed_set_image" | "embed_set_author" => {
            let field_label = if component.data.custom_id == "embed_set_image" {
                "Image URL"
            } else {
                "Author name"
            };
            let modal = serenity::CreateQuickModal::new(field_label).field(
                serenity::CreateInputText::new(
                    serenity::InputTextStyle::Short,
                    field_label,
                    "value",
                )
                .required(true),
            );

            if let Some(response) = component.quick_modal(ctx, modal).await? {
                let value = response.inputs[0].clone();
                let mut drafts = data.embed_drafts.lock().await;
                if let Some(draft) = drafts.get_mut(&component.user.id) {
                    if component.data.custom_id == "embed_set_image" {
                        draft.image = Some(value);
                    } else {
                        draft.author = Some(value);
                    }
                    response
                        .interaction
                        .create_response(
                            ctx,
                            serenity::CreateInteractionResponse::UpdateMessage(
                                serenity::CreateInteractionResponseMessage::new()
                                    .embed(draft.to_embed())
                                    .components(build_button_rows()),
                            ),
                        )
                        .await?;
                }
            }
        }
        "embed_edit_description" => {
            let current = {
                let drafts = data.embed_drafts.lock().await;
                drafts
                    .get(&component.user.id)
                    .and_then(|d| d.description.clone())
                    .unwrap_or_default()
            };

            let modal = serenity::CreateQuickModal::new("Edit Description").field(
                serenity::CreateInputText::new(
                    serenity::InputTextStyle::Paragraph,
                    "Description",
                    "description",
                )
                .value(current)
                .required(false),
            );

            if let Some(response) = component.quick_modal(ctx, modal).await? {
                let new_description = response.inputs[0].clone();
                let mut drafts = data.embed_drafts.lock().await;
                if let Some(draft) = drafts.get_mut(&component.user.id) {
                    draft.description = if new_description.is_empty() {
                        None
                    } else {
                        Some(new_description)
                    };
                    response
                        .interaction
                        .create_response(
                            ctx,
                            serenity::CreateInteractionResponse::UpdateMessage(
                                serenity::CreateInteractionResponseMessage::new()
                                    .embed(draft.to_embed())
                                    .components(build_button_rows()),
                            ),
                        )
                        .await?;
                }
            }
        }
        "embed_edit_basics" => {
            let (cur_title, cur_color, cur_url, cur_footer) = {
                let drafts = data.embed_drafts.lock().await;
                let d = drafts.get(&component.user.id).cloned().unwrap_or_default();
                (
                    d.title.unwrap_or_default(),
                    d.color.map(|c| format!("{c:06X}")).unwrap_or_default(),
                    d.url.unwrap_or_default(),
                    d.footer.unwrap_or_default(),
                )
            };

            let modal = serenity::CreateQuickModal::new("Edit Title/Color/URL/Footer")
                .field(
                    serenity::CreateInputText::new(
                        serenity::InputTextStyle::Short,
                        "Title",
                        "title",
                    )
                    .value(cur_title)
                    .required(false),
                )
                .field(
                    serenity::CreateInputText::new(
                        serenity::InputTextStyle::Short,
                        "Color (hex)",
                        "color",
                    )
                    .value(cur_color)
                    .required(false),
                )
                .field(
                    serenity::CreateInputText::new(serenity::InputTextStyle::Short, "URL", "url")
                        .value(cur_url)
                        .required(false),
                )
                .field(
                    serenity::CreateInputText::new(
                        serenity::InputTextStyle::Short,
                        "Footer",
                        "footer",
                    )
                    .value(cur_footer)
                    .required(false),
                );

            if let Some(response) = component.quick_modal(ctx, modal).await? {
                let inputs = response.inputs;
                let mut drafts = data.embed_drafts.lock().await;
                if let Some(draft) = drafts.get_mut(&component.user.id) {
                    draft.title = (!inputs[0].is_empty()).then(|| inputs[0].clone());
                    draft.color = u32::from_str_radix(inputs[1].trim_start_matches('#'), 16).ok();
                    draft.url = (!inputs[2].is_empty()).then(|| inputs[2].clone());
                    draft.footer = (!inputs[3].is_empty()).then(|| inputs[3].clone());

                    response
                        .interaction
                        .create_response(
                            ctx,
                            serenity::CreateInteractionResponse::UpdateMessage(
                                serenity::CreateInteractionResponseMessage::new()
                                    .embed(draft.to_embed())
                                    .components(build_button_rows()),
                            ),
                        )
                        .await?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}
