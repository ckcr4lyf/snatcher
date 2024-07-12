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

    let cfg: Box<Config> = Box::new(confy::load("snatcher", "snatcher").unwrap());
    let leaked_config = Box::leak(cfg);

    let filter = Box::new(filters::Filter {
        valid_regexes: RegexSet::new(&leaked_config.valid_regexes).unwrap(),
        size_max: leaked_config.max_size,
    });
    let leaked_filter: &'static filters::Filter = Box::leak(filter);

    let tl = trackers::torrentleech::TorrentleechTracker::new(&leaked_config.torrentleech);
    let ipt = trackers::ipt::IptTracker::new(&leaked_config.ipt);

    let tl_t = tokio::spawn(async move {
        tl.monitor(&leaked_filter).await
    });
    let ipt_t = tokio::spawn(async move {
        ipt.monitor(&leaked_filter).await
    });

    // We don't care about the result (should we?)
    let _ = join!(tl_t, ipt_t);

    Ok(())
}
