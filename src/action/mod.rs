// TODO: maybe trait?

use std::ffi::OsStr;
use log::{debug, error, info, trace};

pub fn add_to_qbit_v2(path: &OsStr) {
    trace!("Going to add {:?}", path);

    // Assume qbit-race.
    // mix args and arg so we can pass str and OsStr
    match std::process::Command::new("qbit-race")
        .args(["add", "-p"])
        .arg(&path)
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                info!("Added {:?} to qbittorrent", path);
                debug!(
                    "Added to qbittorrent: {}",
                    String::from_utf8_lossy(&output.stdout)
                );
            } else {
                error!("Failed to add. Program return non-zero exit status: {:?}\nstderr: {}", output.status.code(), String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            error!("Failed to add: {}", e);
        }
    }
}
