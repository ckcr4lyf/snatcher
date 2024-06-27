pub struct TorrentleechTracker {

}

#[derive(Debug)]
pub struct TorrentleechTorrent {
    name: String,
    uploader: String,
    url: String,
    freeleech: bool,
}

impl super::Torrent for TorrentleechTorrent {
    fn name(&self) -> &str {
        return &self.name;
    }
}


impl super::Tracker for TorrentleechTracker {
    fn parse_message(msg: &str) -> Option<TorrentleechTorrent> {
        // println!("Going ot par")
        let name_start_index = msg.find("Name:'")?;
        let name_end_index = msg.find("' uploaded by '")?;
        let (uploader_end_index, freeleech, lenn) = match msg.find("' - ") {
            Some(pos) => (pos, false, 11),
            None => {
                match msg.find("' freeleech - ") {
                    Some(pos) => (pos, true, 21),
                    None => return None
                }
            }
        };

        Some(TorrentleechTorrent{
            name: msg[name_start_index+6..name_end_index].to_owned(),
            uploader: msg[name_end_index+15..uploader_end_index].to_owned(),
            url: msg[uploader_end_index+lenn..].to_owned(),
            freeleech: freeleech,
        })
    }
}