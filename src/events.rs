pub type Snowflake = String;

#[derive(Debug)]
pub enum Event {
    Ready(ReadyData),
    MessageCreate(ReceivedMessage)
}

#[derive(Debug)]
pub struct ReadyData {
    pub user: Myself
}

#[derive(Debug, Deserialize)]
pub struct ReceivedMessage {
    pub id: Snowflake,
    pub channel_id: Snowflake,
    pub content: String,
    timestamp: String
}

#[derive(Debug, Deserialize)]
pub struct Myself {
    pub id: Snowflake,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
    pub bot: bool,
    pub mfa_enabled: bool,
    pub verified: bool,
    pub email: Option<String>
}
