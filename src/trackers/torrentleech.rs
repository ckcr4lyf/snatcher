use std::{env::temp_dir, io::Write};

use serde_bencode::de;

use crate::torrent;

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

    async fn parse_message(&self, msg: &str) -> Option<Self::Torrent> {
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

        let download_url = format!("https://www.torrentleech.org/rss/download/{}/{}/{}.torrent", id, self.rss_key, name_dot);
        println!("Download url is {}", download_url);

        let res = reqwest::get(download_url).await;

        match res {
            Ok(v) => {
                println!("Got HTTP {}", v.status());
                let p = temp_dir().as_path().join(format!("{}.torrent", name_dot));
                let mut f = std::fs::File::create(p).unwrap();

                let bytes = v.bytes().await.unwrap();

                match de::from_bytes::<torrent::Torrent>(&bytes) {
                    Ok(t) => {
                        println!("[SIZE = {}] Got {:?}", t.size(), t)
                    },
                    Err(e) => {
                        println!("Fucked {}", e)
                    }
                }
                match f.write_all(&bytes) {
                    Ok(_) => {
                        println!("wrote to file {}", name_dot)
                    },
                    Err(e) => {
                        println!("fail to write to file.")
                    }
                }
            },
            Err(e) => {
                println!("Got error {}", e);
            }
        }
        // TODO: use reqwest

        Some(TorrentleechTorrent{
            name: name_dot,
            uploader: msg[name_end_index+15..uploader_end_index].to_owned(),
            url,
            freeleech,
            id,
        })
    }
}