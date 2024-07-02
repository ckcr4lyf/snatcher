use log::debug;

use crate::trackers;

pub struct Filter {
    size_max: i64,
}

impl Filter {
    fn check(&self, torrent: impl trackers::Torrent) -> bool {
        if torrent.size() > self.size_max {
            debug!("Torrent size {} is larger than size_max {}. Skipping", torrent.size(), self.size_max);
            return false;
        }

        debug!("All checks pass!");
        return true;
    }
}