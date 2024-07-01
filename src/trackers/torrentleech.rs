use std::{env::temp_dir, ffi::{OsStr, OsString}, io::Write};

use futures::StreamExt;
use irc::{client::{data::Config, Client}, proto::Command};
use log::{debug, error, info, trace};
use serde_bencode::de;

use crate::{action::add_to_qbit, torrent, trackers::Torrent};

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
    path: OsString,
    size: i64,
}

impl super::Torrent for TorrentleechTorrent {
    fn name(&self) -> &str {
        return &self.name;
    }

    fn path(&self) -> &OsStr {
        return &self.path;
    }

    fn size(&self) -> i64 {
        return self.size
    }
}


impl super::Tracker for TorrentleechTracker {
    type Torrent = TorrentleechTorrent;

    async fn parse_message(&self, msg: &str) -> Option<Self::Torrent> {
        trace!("parse_message for {}", msg);

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

        trace!("Going to download torrent from {}", download_url);
        let res = reqwest::get(download_url).await;

        let response = match res {
            Ok(v) => v,
            Err(e) => {
                error!("Got error from HTTP request: {}", e);
                return None;
            }
        };

        trace!("Got HTTP {} from TL", response.status());
        let bytes = response.bytes().await.unwrap();

        // Try and parse torrent
        trace!("Going to bencode-decode torrent");
        let t = match de::from_bytes::<torrent::Torrent>(&bytes) {
            Ok(t) => {
                trace!("Parsed torrent, got {:?}", t);
                t
            },
            Err(e) => {
                error!("Error bencode-decoding torrent: {}", e);
                return None;
            }
        };

        // TODO: Only download torrent if match filters?
        let filename = format!("{}.torrent", name_dot);
        let p = temp_dir().as_path().join(&filename);
        let mut f = std::fs::File::create(&p).unwrap();
        
        match f.write_all(&bytes) {
            Ok(_) => {
                debug!("wrote to file {}", filename);
                Some(TorrentleechTorrent{
                    name: name_dot,
                    uploader: msg[name_end_index+15..uploader_end_index].to_owned(),
                    url,
                    freeleech,
                    id,
                    path: p.into(),
                    size: t.size(),
                })
            },
            Err(e) => {
                error!("fail to write to file; {}", e);
                None
            }
        }
    }

    async fn monitor(&self) -> Result<(), failure::Error> {
        let config = Config {
            nickname: Some("snatcherdev_bot".to_owned()),
            server: Some("irc.torrentleech.org".to_owned()),
            port: Some(7021),
            channels: vec!["#tlannounces".to_owned()],
            ..Config::default()
        };

        info!("Connecting to IRC...");    
        let mut client = Client::from_config(config).await?;
        client.identify()?;
        let mut stream = client.stream()?;
        info!("Connected");
        // let x = trackers::torrentleech::TorrentleechTracker{};
        // let tl = trackers::torrentleech::TorrentleechTracker::new(&env::var("TL_RSS_KEY").unwrap());
    
    
        while let Some(message) = stream.next().await.transpose()? {
            match message.command {
                Command::PRIVMSG(p1, p2) => {
                    let x = self.parse_message(&p2).await;
                    if let Some(x) = x {
                        debug!("Got new release: {:?}", x);
    
                        // At this step, should apply the filtering rules?
    
                        // Then call .download()
                        // For some trackers (like TL), it will be no-op since part of parse_msg (to get size)
                        // For others, it will only then trigger the DL
    
                        // Optimization: For TL, write to disk after calling `.download()`? (keep in memory before that)
                        if x.size() < ((1 << 30) * 4) {
                            debug!("Size is less than 4GiB ({}), adding...", x.size());
                            // add_to_qbit(x);
                        } else {
                            debug!("Size is too large, skipping... ({})", x.size());
                        }
                    } else {
                        error!("Failed to parse message: {}", p2);
                    }
                },
                other => {
                    // println!("got something else: {:?}", other)
                }
            }
        }
    
        Ok(())
    }
}