#[derive(Clone, PartialEq, Debug)]
pub enum Event<'a> {
    Ready(ReadyData<'a>)
}

pub type Snowflake = String;

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct User {
    id: Snowflake,
    username: String,
    discriminator: String,
    avatar: Option<String>,
    bot: Option<bool>,
    mfa_enabled: Option<bool>,
    verified: Option<bool>,
    email: Option<String>
}

#[derive(Debug, PartialEq, Clone)]
pub struct ReadyData<'a> {
    pub guilds: &'a [Snowflake],
    pub session_id: &'a str,
    pub user: User
}
