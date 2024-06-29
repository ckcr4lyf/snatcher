use std::env;

use action::add_to_qbit;
use irc::client::prelude::*;
use futures::prelude::*;

mod trackers;
use log::{debug, error, info};
use trackers::{Torrent, Tracker};
mod torrent;
mod action;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    env_logger::init();
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
    let tl = trackers::torrentleech::TorrentleechTracker::new(&env::var("TL_RSS_KEY").unwrap());

    info!("Connecting to IRC...");

    while let Some(message) = stream.next().await.transpose()? {
        match message.command {
            Command::PRIVMSG(p1, p2) => {
                let x = tl.parse_message(&p2).await;
                if let Some(x) = x {
                    debug!("Got new release: {:?}", x);

                    // At this step, should apply the filtering rules?

                    // Then call .download()
                    // For some trackers (like TL), it will be no-op since part of parse_msg (to get size)
                    // For others, it will only then trigger the DL

                    // Optimization: For TL, write to disk after calling `.download()`? (keep in memory before that)
                    if x.size() < ((1 << 30) * 4) {
                        debug!("Size is less than 4GiB ({}), adding...", x.size());
                        add_to_qbit(x);
                    } else {
                        debug!("Size is too large, skipping... ({})", x.size());
                    }
                } else {
                    error!("Failed to parse message: {}", p2);
                }
            },
            other => {
                // println!("got something else: {:?}", other)
            }
        }
    }

    Ok(())
}