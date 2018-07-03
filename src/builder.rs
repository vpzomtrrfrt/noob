pub struct MessageBuilder<'a> {
    content: &'a str
}

impl<'a> MessageBuilder<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content
        }
    }

    pub fn to_request_body(&self, channel: &str) -> String {
        json!({
            "content": self.content,
            "channel_id": channel
        }).to_string()
    }
}
