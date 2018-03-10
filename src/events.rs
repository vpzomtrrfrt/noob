#[derive(Clone, PartialEq, Debug)]
pub enum Event<'a> {
    Ready(ReadyData<'a>),
    MessageCreate(Message)
}

pub type Snowflake = String;

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct User {
    pub id: Snowflake,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
    pub bot: Option<bool>,
    pub mfa_enabled: Option<bool>,
    pub verified: Option<bool>,
    pub email: Option<String>
}

#[derive(Debug, PartialEq, Clone)]
pub struct ReadyData<'a> {
    pub guilds: &'a [Snowflake],
    pub session_id: &'a str,
    pub user: User
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Message {
    pub id: Snowflake,
    pub channel_id: Snowflake,
    pub author: User,
    pub content: String,
    pub timestamp: String
}
