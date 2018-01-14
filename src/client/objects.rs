use serde_json;

pub type Snowflake = String;

#[derive(Debug, Deserialize)]
pub struct Message {
    id: Snowflake,
    channel_id: Snowflake,
    //author: User,
    content: String,
    timestamp: String,
    edited_timestamp: Option<String>,
    tts: bool,
    mention_everyone: bool,
    //mentions: Box<[User]>,
    mention_roles: Box<[Snowflake]>,
    //attachments: Box<[Attachment]>,
    //embeds: Box<[Embeds]>,
    //reactions: Vec<Reaction>,
    nonce: Option<Snowflake>,
    pinned: bool,
    webhook_id: Option<String>,
    #[serde(rename = "type")]
    message_type: u8
}

#[derive(Debug, Deserialize)]
pub struct Channel {
    id: Snowflake,
    #[serde(rename = "type")]
    channel_type: u8,
    guild_id: Option<Snowflake>,
    position: u16,
    //permission_overwrites: [PermissionOverwrite],
    name: String,
    topic: Option<String>,
    #[serde(default)]
    nsfw: bool,
    last_message_id: Option<Snowflake>,
    bitrate: Option<u32>,
    user_limit: Option<u32>,
    //recipient: [User],
    icon: Option<String>,
    owner_id: Option<Snowflake>,
    application_id: Option<Snowflake>,
    parent_id: Option<Snowflake>,
    last_pin_timestamp: Option<String>
}

#[derive(Debug, Deserialize)]
pub struct Activity {
    name: String,
    activity_type: u8,
    url: Option<String>
}

#[derive(Debug, Deserialize)]
struct PresencePartial {
    //user: UserPartial,
    status: String,
    game: Option<Activity>
}

#[derive(Debug, Deserialize)]
pub struct Guild {
    id: Snowflake,
    name: String,
    icon: Option<String>,
    splash: Option<String>,
    #[serde(default)]
    owner: bool,
    owner_id: Snowflake,
    permissions: Option<u64>,
    region: String,
    afk_channel_id: Option<Snowflake>,
    afk_timeout: u64,
    embed_enabled: Option<bool>,
    embed_channel_id: Option<Snowflake>,
    verification_level: u8,
    default_message_notifications: u8,
    explicit_content_filter: u8,
    //roles: [Role],
    //emojis: [Emoji],
    features: Vec<String>,
    mfa_level: u8,
    application_id: Option<Snowflake>,
    widget_enabled: Option<bool>,
    widget_channel_id: Option<Snowflake>,
    system_channel_id: Option<Snowflake>,
    joined_at: String,
    large: bool,
    unavailable: bool,
    member_count: u64,
    //voice_states: [VoiceStatePartial],
    //members: [GuildMember],
    channels: Vec<Channel>,
    //presences: [PresencePartial]
}
