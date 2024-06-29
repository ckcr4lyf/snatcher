// TODO: maybe trait?

use log::{debug, error, trace};

use crate::trackers;

pub fn add_to_qbit(torrent: impl trackers::Torrent) {
    trace!("Going to add {}", torrent.name());

    // Assume qbit-race.
    // mix args and arg so we can pass str and OsStr
    match std::process::Command::new("qbit-race").args(["add", "-p"]).arg(torrent.path()).output() {
        Ok(output) => {
            debug!("Added to qbittorrent: {}", String::from_utf8_lossy(&output.stdout));
        },
        Err(e) => {
            error!("Failed to add: {}", e);
        }
    }
}