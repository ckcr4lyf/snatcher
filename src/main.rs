use std::{env, sync::Arc};

use action::add_to_qbit;
use futures::prelude::*;
use irc::client::prelude::*;

mod trackers;
use log::{debug, error, info};
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use tokio::join;
use trackers::{Torrent, Tracker};
mod action;
mod filters;
mod torrent;

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct Config {
    valid_regexes: Vec<String>,
    max_size: i64,

    // Per Tracker Configs
    ipt: IptConfig,
    torrentleech: TorrentleechConfig,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct IptConfig {
    username: String,
    passkey: String,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct TorrentleechConfig {
    username: String,
    rss_key: String,
}

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    env_logger::init();

    let cfg: Config = confy::load("snatcher", "snatcher").unwrap();
    let tl = trackers::torrentleech::TorrentleechTracker::new(&cfg.torrentleech.rss_key);
    let ipt = trackers::ipt::IptTracker::new(&cfg.ipt.passkey);

    let filter = Arc::new(filters::Filter {
        valid_regexes: RegexSet::new(&cfg.valid_regexes).unwrap(),
        size_max: cfg.max_size,
    });

    // let tl_t = tokio::spawn(async move {
    //     tl.monitor(&filter).await;
    // });
    let ipt_t = tokio::spawn(async move {
        // ipt.monitor(filter).await;
    });

    // join!(tl_t, ipt_t);
    join!(ipt_t);

    Ok(())
}
