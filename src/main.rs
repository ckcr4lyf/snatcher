use std::{env, sync::Arc};

use action::add_to_qbit;
use futures::prelude::*;
use irc::client::prelude::*;

mod trackers;
use log::{debug, error, info};
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use tokio::join;
use trackers::{ipt::ipt_monitor, Torrent, Tracker};
mod action;
mod filters;
mod torrent;

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
struct Config {
    // Per Tracker Configs
    ipt: IptConfig,
    torrentleech: TorrentleechConfig,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
struct FilterConfig {
    valid_regexes: Vec<String>,
    max_size: i64,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
struct IptConfig {
    username: String,
    passkey: String,
    filter: FilterConfig,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
struct TorrentleechConfig {
    username: String,
    rss_key: String,
    filter: FilterConfig,
}

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    env_logger::init();

    let cfg: Box<Config> = Box::new(confy::load("snatcher", "snatcher").unwrap());
    let leaked_config: &'static Config = Box::leak(cfg);

    info!("Got config as {:?}", &leaked_config);

    let tl_filter = Box::new(filters::Filter {
        valid_regexes: RegexSet::new(&leaked_config.torrentleech.filter.valid_regexes).unwrap(),
        size_max: leaked_config.torrentleech.filter.max_size,
    });
    let leaked_tl_filter: &'static filters::Filter = Box::leak(tl_filter);

    let ipt_filter = Box::new(filters::Filter {
        valid_regexes: RegexSet::new(&leaked_config.ipt.filter.valid_regexes).unwrap(),
        size_max: leaked_config.ipt.filter.max_size,
    });
    let leaked_ipt_filter: &'static filters::Filter = Box::leak(ipt_filter);

    let tl_t = tokio::spawn(async move {
        let tl = trackers::torrentleech::TorrentleechTracker::new(&leaked_config.torrentleech);
        tl.monitor(&leaked_tl_filter).await
    });
    let ipt_t = tokio::spawn(async move {
        ipt_monitor(&leaked_config.ipt, &leaked_ipt_filter).await
    });

    // We don't care about the result (should we?)
    let _ = join!(tl_t, ipt_t);

    Ok(())
}
