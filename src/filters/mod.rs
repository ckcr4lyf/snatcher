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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummyTorrent {
        size: i64,
    }

    impl trackers::Torrent for DummyTorrent {
        fn size(&self) -> i64 {
            return self.size;
        }

        fn name(&self) -> &str {
            unimplemented!()
        }

        fn path(&self) -> &std::ffi::OsStr {
            unimplemented!()
        }
    }

    #[test]
    fn check(){
        let filter = Filter{
            size_max: 4000,
        };
        
        let dummy = DummyTorrent{
            size: 100
        };

        assert_eq!(filter.check(dummy), true);
    }
}