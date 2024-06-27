pub mod torrentleech;

pub trait Torrent: std::fmt::Debug {
    fn name(&self) -> &str;

}

pub trait Tracker {
    type Torrent: Torrent;

    fn parse_message(&self, msg: &str) -> Option<Self::Torrent>;
}
