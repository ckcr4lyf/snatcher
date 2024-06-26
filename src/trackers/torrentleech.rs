pub struct TorrentleechTracker {

}

pub struct TorrentleechTorrent {
    name: String
}

impl super::Torrent for TorrentleechTorrent {
    fn name(&self) -> &str {
        return &self.name;
    }
}


impl super::Tracker for TorrentleechTracker {
    fn parse_message(msg: &str) -> Option<Box<dyn super::Torrent>> {
        let nameStartIndex = msg.find("Name:'")?;
        let nameEndIndex = msg.find("' uploaded by '")?;

        Some(Box::new(TorrentleechTorrent{
            name: msg[nameStartIndex..nameEndIndex].to_owned()
        }))
    }
}