use serde_derive::Deserialize;

/// Type used for IDs
pub type Snowflake = String;

#[derive(Debug)]
/// Known events that may be received
pub enum Event {
    /// Connection established
    Ready(ReadyData),
    /// Message received
    MessageCreate(ReceivedMessage),
}

#[derive(Debug)]
/// Object contained in [`Event::Ready`]
pub struct ReadyData {
    /// Authenticated user info
    pub user: Myself,
}

#[derive(Debug, Deserialize)]
/// Message received from a channel
pub struct ReceivedMessage {
    /// Message ID
    pub id: Snowflake,
    /// ID of the origin channel
    pub channel_id: Snowflake,
    /// Text content of the message
    pub content: String,
    /// Whether this is a TTS message
    pub tts: bool,
    /// Author of the message
    pub author: User,
    timestamp: String,
}

#[derive(Debug, Deserialize)]
/// Data about the current user. ([relevant Discord docs](https://discordapp.com/developers/docs/resources/user#user-object))
pub struct Myself {
    /// User ID
    pub id: Snowflake,
    /// Username, not unique
    pub username: String,
    /// 4-digit Discord tag
    pub discriminator: String,
    /// User's [avatar hash](https://discordapp.com/developers/docs/reference#image-formatting)
    pub avatar: Option<String>,
    /// Whether this is a bot account
    #[serde(default)]
    pub bot: bool,
    /// Whether the account has MFA enabled
    pub mfa_enabled: bool,
    /// Whether the email address on this account has been verified
    pub verified: bool,
    /// Email address
    pub email: Option<String>,
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

impl Into<User> for Myself {
    fn into(self) -> User {
        User {
            id: self.id,
            username: self.username,
            discriminator: self.discriminator,
            avatar: self.avatar,
        }
    }
}
