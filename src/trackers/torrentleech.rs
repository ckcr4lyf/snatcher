pub struct TorrentleechTracker {
    rss_key: String,
}

impl TorrentleechTracker {
    pub fn new(rss_key: &str) -> Self {
        TorrentleechTracker{
            rss_key: rss_key.to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct TorrentleechTorrent {
    name: String,
    uploader: String,
    url: String,
    freeleech: bool,
    id: String,
}

impl super::Torrent for TorrentleechTorrent {
    fn name(&self) -> &str {
        return &self.name;
    }
}


impl super::Tracker for TorrentleechTracker {
    type Torrent = TorrentleechTorrent;

    fn parse_message(&self, msg: &str) -> Option<Self::Torrent> {
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

        let name_original: String = msg[name_start_index+6..name_end_index].to_owned();
        let name_dot = name_original.replace(" ", ".");
        let url: String = msg[uploader_end_index+lenn..].to_owned();

        let id_index = url.rfind("/")?;
        let id: String = url[id_index+1..].to_owned();

        Some(TorrentleechTorrent{
            name: name_dot,
            uploader: msg[name_end_index+15..uploader_end_index].to_owned(),
            url,
            freeleech,
            id,
        })
    }
}