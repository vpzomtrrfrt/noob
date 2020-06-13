use crate::Error;
use serde_derive::Serialize;

/// Object used to construct outgoing messages
pub struct MessageBuilder<'a> {
    content: &'a str,
    embed: Option<&'a EmbedBuilder<'a>>,
}

impl<'a> MessageBuilder<'a> {
    /// Create a new MessageBuilder
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            embed: None,
        }
    }

    /// Set an embed for this message
    pub fn set_embed(&mut self, embed: &'a EmbedBuilder<'a>) {
        self.embed = Some(embed);
    }

    /// Set an embed for this message
    pub fn with_embed(mut self, embed: &'a EmbedBuilder<'a>) -> Self {
        self.set_embed(embed);
        self
    }

    #[doc(hidden)]
    pub fn to_request_body(&self, channel: &str) -> Result<String, Error> {
        #[derive(Serialize, Debug)]
        struct MessageCreateBody<'a> {
            content: &'a str,
            channel: &'a str,
            embed: Option<&'a EmbedBuilder<'a>>,
        }
        serde_json::to_string(&MessageCreateBody {
            content: self.content,
            channel,
            embed: self.embed,
        }).map_err(|e| {
            Error::Other(format!(
                "Failed to serialize message creation body: {:?}",
                e
            ))
        })
    }
}

#[derive(Default, Serialize, Debug)]
/// Builder for a message embed
pub struct EmbedBuilder<'a> {
    title: Option<&'a str>,
    description: Option<&'a str>,
    url: Option<&'a str>,
    timestamp: Option<&'a str>,
    color: Option<u32>,
    footer: Option<&'a EmbedFooter<'a>>,
    image: Option<&'a str>,
    thumbnail: Option<&'a str>,
    author: Option<&'a EmbedAuthor<'a>>,
    fields: Vec<&'a EmbedField<'a>>,
}

#[allow(missing_docs)]
impl<'a> EmbedBuilder<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_title(&mut self, title: &'a str) {
        self.title = Some(title);
    }

    pub fn with_title(mut self, title: &'a str) -> Self {
        self.set_title(title);
        self
    }

    pub fn set_author(&mut self, author: &'a EmbedAuthor<'a>) {
        self.author = Some(author);
    }

    pub fn with_author(mut self, author: &'a EmbedAuthor<'a>) -> Self {
        self.set_author(author);
        self
    }

    pub fn set_color(&mut self, color: u32) {
        self.color = Some(color);
    }

    pub fn with_color(mut self, color: u32) -> Self {
        self.set_color(color);
        self
    }

    pub fn add_field(&mut self, field: &'a EmbedField<'a>) {
        self.fields.push(field);
    }

    pub fn with_field(mut self, field: &'a EmbedField<'a>) -> Self {
        self.add_field(field);
        self
    }

    pub fn set_footer(&mut self, footer: &'a EmbedFooter<'a>) {
        self.footer = Some(footer);
    }

    pub fn with_footer(mut self, footer: &'a EmbedFooter<'a>) -> Self {
        self.set_footer(footer);
        self
    }

    pub fn set_description(&mut self, description: &'a str) {
        self.description = Some(description);
    }

    pub fn with_description(mut self, description: &'a str) -> Self {
        self.set_description(description);
        self
    }

    pub fn set_timestamp(&mut self, timestamp: &'a str) {
        self.timestamp = Some(timestamp);
    }

    pub fn with_timestamp(mut self, timestamp: &'a str) -> Self {
        self.set_timestamp(timestamp);
        self
    }
}

#[derive(Default, Serialize, Debug)]
/// Representation of an embed author
pub struct EmbedAuthor<'a> {
    /// Name of author
    pub name: Option<&'a str>,
    /// URL of author
    pub url: Option<&'a str>,
    /// URL of author icon
    pub icon_url: Option<&'a str>,
}

impl<'a> EmbedAuthor<'a> {
    /// Create an empty embed author
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Serialize, Debug)]
/// Representation of an embed footer
pub struct EmbedFooter<'a> {
    text: &'a str,
    icon_url: Option<&'a str>,
}

impl<'a> EmbedFooter<'a> {
    /// Create a text-only footer
    pub fn new(text: &'a str) -> Self {
        EmbedFooter {
            text,
            icon_url: None,
        }
    }
    /// Create a footer with an icon
    pub fn new_with_icon(text: &'a str, icon_url: &'a str) -> Self {
        EmbedFooter {
            text,
            icon_url: Some(icon_url),
        }
    }
}

#[derive(Serialize, Debug)]
/// Representation of an [embed field](https://discordapp.com/developers/docs/resources/channel#embed-object-embed-field-structure)
pub struct EmbedField<'a> {
    name: &'a str,
    value: &'a str,
    inline: bool,
}

impl<'a> EmbedField<'a> {
    /// Create a new embed field
    pub fn new(name: &'a str, value: &'a str) -> Self {
        EmbedField::new_internal(name, value, false)
    }
    /// Create a new inline embed field
    pub fn new_inline(name: &'a str, value: &'a str) -> Self {
        EmbedField::new_internal(name, value, true)
    }
    fn new_internal(name: &'a str, value: &'a str, inline: bool) -> Self {
        EmbedField {
            name,
            value,
            inline,
        }
    }
}
