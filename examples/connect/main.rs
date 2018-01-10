extern crate tokio_discord;
extern crate tokio_core;

fn main() {
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let task = tokio_discord::Client::login_bot(
        &core.handle(),
        &std::env::var("DISCORD_TOKEN")
        .expect("Missing DISCORD_TOKEN")
        )
        .and_then(|client|client.run());
    core.run(task).unwrap();
}
