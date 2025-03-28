use std::{env, sync::Arc, time::Duration};

use action::add_to_qbit;
use failure::Error;
use futures::prelude::*;
use irc::client::prelude::*;

mod trackers;
use log::{debug, error, info};
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use tokio::{join, task::JoinHandle};
use trackers::{ipt, torrentleech, Torrent, Tracker};
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

    debug!("Got config as {:?}", &leaked_config);

    let tl_t = tokio::spawn(async {monitor_wrapper(torrentleech::monitor, &leaked_config.torrentleech, "torrentleech").await});
    let ipt_t = tokio::spawn(async {monitor_wrapper(ipt::ipt_monitor, &leaked_config.ipt, "iptorrents").await});

    // We don't care about the result (should we?)
    let (torrentleech_join_result, ipt_join_result) = join!(tl_t, ipt_t);
    error!("We joined the threads, something went wrong!\nTorrentleech: {:?}\nIPT: {:?}", torrentleech_join_result, ipt_join_result);

    Ok(())
}

async fn monitor_wrapper<F, Fut, T>(monitor_fn: F, config: &'static T, tracker_name: &'static str) -> Result<(), failure::Error>
where 
    F: Fn(&'static T) -> Fut,
    Fut: Future<Output = Result<(), failure::Error>>
{
    loop {
        info!("[{}] going to connect (monitor) in loop...", tracker_name);
        match monitor_fn(config).await {
            Ok(_) => {
                error!("monitor resolved w/ Ok(), should be impossible!");
                // we will sleep for 60s and then reconnect
                tokio::time::sleep(Duration::from_millis(60000)).await;
            },
            Err(e) => match e.downcast_ref::<irc::error::Error>() {
                Some(irc::error::Error::PingTimeout) => {
                    error!("got a ping timeout! will try and reconnect")
                },
                Some(other) => {
                    error!("Got some other IRC error: {:?}", other);
                    // we will sleep for 60s and then reconnect
                    tokio::time::sleep(Duration::from_millis(60000)).await;
                }
                None => {
                    error!("Got non-irc error: {:?}", e);
                    // we will sleep for 60s and then reconnect
                    tokio::time::sleep(Duration::from_millis(60000)).await;
                }
            },
        }
    }
}
