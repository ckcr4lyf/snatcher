use log::debug;
use regex;

use crate::trackers;

pub struct Filter {
    pub valid_regexes: regex::RegexSet,
    pub size_max: i64, // we probably want to make this optional in the future.
}

impl Filter {
    pub fn check(&self, torrent: &impl trackers::Torrent) -> bool {
        if torrent.size() > self.size_max {
            debug!(
                "Torrent size {} is larger than size_max {}. Skipping",
                torrent.size(),
                self.size_max
            );
            return false;
        }

        if self.valid_regexes.matches(torrent.name()).matched_any() == false {
            debug!(
                "Torrent name {} did not match any of the regexes!",
                torrent.name()
            );
            return false;
        }

        debug!("All checks pass!");
        return true;
    }
}

#[cfg(test)]
mod tests {
    use regex::RegexSet;

    use super::*;

    #[derive(Debug)]
    struct DummyTorrent {
        size: i64,
        name: String,
    }

    impl trackers::Torrent for DummyTorrent {
        fn size(&self) -> i64 {
            return self.size;
        }

        fn name(&self) -> &str {
            return &self.name;
        }

        fn path(&self) -> &std::ffi::OsStr {
            unimplemented!()
        }
    }

    #[test]
    fn check() {
        let filter = Filter {
            size_max: 4000,
            valid_regexes: RegexSet::new(&[r"^YOLO$"]).unwrap(),
        };

        let dummy_valid = DummyTorrent {
            size: 100,
            name: "YOLO".to_owned(),
        };

        let dummy_invalid = DummyTorrent {
            size: 100,
            name: "XD".to_owned(),
        };

        assert_eq!(filter.check(&dummy_valid), true);
        assert_eq!(filter.check(&dummy_invalid), false);
    }
}
