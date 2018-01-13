use client::objects;

#[derive(Debug)]
pub enum Event {
    Ready,
    GuildCreate(objects::Guild)
}
