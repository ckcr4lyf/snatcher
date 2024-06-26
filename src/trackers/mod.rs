pub trait Torrent {
    fn name(&self) -> String;
}

pub trait Tracker {
    fn parse_message(msg: &str) -> dyn Torrent;
}
