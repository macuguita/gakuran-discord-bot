use poise::serenity_prelude as serenity;

pub fn build_button_rows() -> Vec<serenity::CreateActionRow> {
    vec![
        serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new("embed_edit_basics")
                .label("Edit Title/Color/URL")
                .style(serenity::ButtonStyle::Primary),
            serenity::CreateButton::new("embed_edit_description")
                .label("Edit Description")
                .style(serenity::ButtonStyle::Primary),
            serenity::CreateButton::new("embed_add_field")
                .label("Add Field")
                .style(serenity::ButtonStyle::Secondary),
        ]),
        serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new("embed_set_image")
                .label("Set Image")
                .style(serenity::ButtonStyle::Secondary),
            serenity::CreateButton::new("embed_set_author")
                .label("Set Author")
                .style(serenity::ButtonStyle::Secondary),
            serenity::CreateButton::new("embed_post")
                .label("Post")
                .style(serenity::ButtonStyle::Success),
            serenity::CreateButton::new("embed_cancel")
                .label("Cancel")
                .style(serenity::ButtonStyle::Danger),
        ]),
    ]
}
