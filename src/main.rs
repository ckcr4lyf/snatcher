use irc::client::prelude::*;
use futures::prelude::*;

mod trackers;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // We can also load the Config at runtime via Config::load("path/to/config.toml")
    let config = Config {
        nickname: Some("snatcherdev_bot".to_owned()),
        server: Some("irc.torrentleech.org".to_owned()),
        port: Some(7021),
        channels: vec!["#tlannounces".to_owned()],
        ..Config::default()
    };

    let mut client = Client::from_config(config).await?;
    client.identify()?;

    let mut stream = client.stream()?;

    while let Some(message) = stream.next().await.transpose()? {
        match message.command {
            Command::PRIVMSG(p1, p2) => {
                println!("got stuff: {} and {} (RAW: {:2X?})", p1, p2, p2.as_bytes())
            },
            other => {
                println!("got something else: {:?}", other)
            }
        }
        // print!("{}", message.co);
    }

    Ok(())
}