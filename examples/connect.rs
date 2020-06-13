use futures::TryStreamExt;

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN");

    let (_client, stream) = noob::Client::connect(&token)
        .await
        .expect("Failed to connect to Discord");
    stream
        .try_for_each(|evt| {
            println!("{:?}", evt);
            futures::future::ready(Ok(()))
        })
        .await
        .unwrap();
}
