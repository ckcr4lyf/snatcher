use std::env;

use action::add_to_qbit;
use irc::client::prelude::*;
use futures::prelude::*;

mod trackers;
use log::{debug, error, info};
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use tokio::join;
use trackers::{Torrent, Tracker};
mod torrent;
mod action;
mod filters;

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct Config {
    valid_regexes: Vec<String>,
    max_size: i64,
}

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    env_logger::init();
    // We can also load the Config at runtime via Config::load("path/to/config.toml")

    let cfg: Config = confy::load("snatcher", "snatcher").unwrap();

    let tl = trackers::torrentleech::TorrentleechTracker::new(&env::var("TL_RSS_KEY").unwrap());
    let ipt = trackers::ipt::IptTracker::new(&env::var("IPT_PASSKEY").unwrap());

    let filter = filters::Filter{
        valid_regexes: RegexSet::new(&cfg.valid_regexes).unwrap(),
        size_max: cfg.max_size,
    };

    // let tl_t = tokio::spawn(async move {
    //     tl.monitor(filter).await;
    // });
    let ipt_t = tokio::spawn(async move {
        ipt.monitor(filter).await;
    });

    // join!(tl_t, ipt_t);
    join!(ipt_t);

    Ok(())
}