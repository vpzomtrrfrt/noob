pub type Snowflake = String;

#[derive(Debug)]
pub enum Event {
    Ready(ReadyData),
    MessageCreate(ReceivedMessage)
}

#[derive(Debug)]
pub struct ReadyData {

}

#[derive(Debug, Deserialize)]
pub struct ReceivedMessage {
    pub id: Snowflake,
    pub channel_id: Snowflake,
    pub content: String,
    timestamp: String
}
