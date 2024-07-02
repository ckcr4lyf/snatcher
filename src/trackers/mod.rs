use std::ffi::{OsStr, OsString};

pub mod torrentleech;
pub mod ipt;

pub trait Torrent: std::fmt::Debug {
    fn name(&self) -> &str;
    fn path(&self) -> &OsStr;
    fn size(&self) -> i64;
    async fn download(&self) -> Result<OsString, failure::Error>;
}

pub trait Tracker {
    type Torrent: Torrent;

    async fn monitor(&self) -> Result<(), failure::Error>;
    async fn parse_message(&self, msg: &str) -> Option<Self::Torrent>;
}
