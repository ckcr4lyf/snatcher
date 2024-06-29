use std::env;

use action::add_to_qbit;
use irc::client::prelude::*;
use futures::prelude::*;

mod trackers;
use log::{debug, error, info};
use tokio::join;
use trackers::{Torrent, Tracker};
mod torrent;
mod action;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    env_logger::init();
    // We can also load the Config at runtime via Config::load("path/to/config.toml")

    let tl = trackers::torrentleech::TorrentleechTracker::new(&env::var("TL_RSS_KEY").unwrap());

    join!(tl.monitor());

    Ok(())
}