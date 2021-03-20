use serde_derive::Deserialize;

/// Type used for IDs
pub type Snowflake = String;

#[derive(Debug, Deserialize)]
/// Represents a message sent in a channel
/// ([relevant Discord docs](https://discord.com/developers/docs/resources/channel#message-object))
pub struct Message {
    /// Message ID
    pub id: Snowflake,
    /// ID of the origin channel
    pub channel_id: Snowflake,
    /// Author of the message
    pub author: User,
    /// Text content of the message
    pub content: String,
    timestamp: String,
    edited_timestamp: Option<String>,
    /// Whether this is a TTS message
    pub tts: bool,
    /// Whether this message mentions everyone
    pub mention_everyone: bool,
    /// Users specifically mentioned in the message
    pub mentions: Vec<User>,
    /// Roles specifically mentioned in this message
    pub mention_roles: Vec<Snowflake>,
}

#[derive(Debug, Deserialize)]
/// Data about a Discord User. ([relevant Discord docs](https://discordapp.com/developers/docs/resources/user#user-object))
pub struct User {
    /// User ID
    pub id: Snowflake,
    /// Username, not unique
    pub username: String,
    /// 4-digit Discord tag
    pub discriminator: String,
    /// User's [avatar hash](https://discordapp.com/developers/docs/reference#image-formatting)
    pub avatar: Option<String>,
}
