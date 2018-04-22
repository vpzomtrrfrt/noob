#[derive(Serialize)]
pub struct Embed {
    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
    timestamp: Option<String>,
    color: Option<u32>,
    footer: Option<EmbedFooter>,
    image: Option<String>,
    thumbnail: Option<String>,
    author: Option<EmbedAuthor>,
    fields: Vec<EmbedField>
}

impl Embed {
    pub fn new() -> Self {
        Embed {
            title: None,
            description: None,
            url: None,
            timestamp: None,
            color: None,
            footer: None,
            image: None,
            thumbnail: None,
            author: None,
            fields: Vec::new()
        }
    }
    pub fn set_title(&mut self, value: String) -> &mut Self {
        self.title = Some(value);
        self
    }
    pub fn set_description(&mut self, value: String) -> &mut Self {
        self.description = Some(value);
        self
    }
    pub fn set_url(&mut self, value: String) -> &mut Self {
        self.url = Some(value);
        self
    }
    pub fn set_timestamp(&mut self, value: String) -> &mut Self {
        self.timestamp = Some(value);
        self
    }
    pub fn set_color(&mut self, value: u32) -> &mut Self {
        self.color = Some(value);
        self
    }
    pub fn set_footer(&mut self, value: EmbedFooter) -> &mut Self {
        self.footer = Some(value);
        self
    }
    pub fn set_image(&mut self, value: String) -> &mut Self {
        self.image = Some(value);
        self
    }
    pub fn set_thumbnail(&mut self, value: String) -> &mut Self {
        self.thumbnail = Some(value);
        self
    }
    pub fn set_author(&mut self, value: EmbedAuthor) -> &mut Self {
        self.author = Some(value);
        self
    }
    pub fn add_field(&mut self, field: EmbedField) -> &mut Self {
        self.fields.push(field);
        self
    }
}

#[derive(Serialize)]
pub struct EmbedAuthor {
    name: Option<String>,
    url: Option<String>,
    icon_url: Option<String>
}

impl EmbedAuthor {
    pub fn new() -> Self {
        EmbedAuthor {
            name: None,
            url: None,
            icon_url: None
        }
    }
    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
    }
    pub fn set_url(&mut self, url: String) -> &mut Self {
        self.url = Some(url);
        self
    }
    pub fn set_icon_url(&mut self, url: String) -> &mut Self {
        self.icon_url = Some(url);
        self
    }
}

#[derive(Serialize)]
pub struct EmbedFooter {
    text: String,
    icon_url: Option<String>
}

impl EmbedFooter {
    pub fn new(text: String) -> Self {
        EmbedFooter {
            text,
            icon_url: None
        }
    }
    pub fn set_icon_url(&mut self, url: String) -> &mut Self {
        self.icon_url = Some(url);
        self
    }
}

#[derive(Serialize)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    pub inline: bool
}

impl EmbedField {
    pub fn new(name: String, value: String) -> Self {
        EmbedField::new_internal(name, value, false)
    }
    pub fn new_inline(name: String, value: String) -> Self {
        EmbedField::new_internal(name, value, true)
    }
    fn new_internal(name: String, value: String, inline: bool) -> Self {
        EmbedField {
            name, value, inline
        }
    }
}
