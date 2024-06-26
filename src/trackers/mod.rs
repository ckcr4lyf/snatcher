pub mod torrentleech;

pub trait Torrent {
    fn name(&self) -> &str;
}

pub trait Tracker {
    fn parse_message(msg: &str) -> Option<Box<dyn Torrent>>;
}
