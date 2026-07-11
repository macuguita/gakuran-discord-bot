pub mod buttons;
pub mod interactions;

use poise::serenity_prelude as serenity;

#[derive(Default, Clone)]
pub struct EmbedDraft {
    pub title: Option<String>,
    pub description: Option<String>,
    pub color: Option<u32>,
    pub url: Option<String>,
    pub footer: Option<String>,
    pub image: Option<String>,
    pub author: Option<String>,
    pub fields: Vec<(String, String, bool)>,
}

impl EmbedDraft {
    #[allow(clippy::unreadable_literal)]
    pub fn to_embed(&self) -> serenity::CreateEmbed {
        let mut e = serenity::CreateEmbed::new().color(self.color.unwrap_or(0x5865F2));
        if let Some(t) = &self.title {
            e = e.title(t);
        }
        if let Some(d) = &self.description {
            e = e.description(d);
        }
        if let Some(u) = &self.url {
            e = e.url(u);
        }
        if let Some(f) = &self.footer {
            e = e.footer(serenity::CreateEmbedFooter::new(f));
        }
        if let Some(img) = &self.image {
            e = e.image(img);
        }
        if let Some(a) = &self.author {
            e = e.author(serenity::CreateEmbedAuthor::new(a));
        }
        for (name, value, inline) in &self.fields {
            e = e.field(name, value, *inline);
        }
        e
    }
}
