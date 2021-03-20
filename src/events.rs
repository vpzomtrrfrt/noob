use crate::types::{Message, Snowflake, User};
use serde_derive::Deserialize;

#[derive(Debug)]
/// Known events that may be received
pub enum Event {
    /// Connection established
    Ready(ReadyData),
    /// Message received
    MessageCreate(Message),
}

#[derive(Debug)]
/// Object contained in [`Event::Ready`]
pub struct ReadyData {
    /// Authenticated user info
    pub user: Myself,
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
