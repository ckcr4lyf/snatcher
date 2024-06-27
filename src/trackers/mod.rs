pub mod torrentleech;

pub trait Torrent: std::fmt::Debug {
    fn name(&self) -> &str;

}

pub trait Tracker {
    fn parse_message(msg: &str) -> Option<impl Torrent>;
}
