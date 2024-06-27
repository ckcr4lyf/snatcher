use irc::client::prelude::*;
use futures::prelude::*;

mod trackers;
use trackers::Tracker;

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

    // let x = trackers::torrentleech::TorrentleechTracker{};

    while let Some(message) = stream.next().await.transpose()? {
        match message.command {
            Command::PRIVMSG(p1, p2) => {
                let x = trackers::torrentleech::TorrentleechTracker::parse_message(&p2);

                if let Some(x) = x {
                    println!("Got new release: {:?}", x);
                } else {
                    println!("Failed to parse {}", p2);
                }
            },
            other => {
                // println!("got something else: {:?}", other)
            }
        }
    }

    Ok(())
}