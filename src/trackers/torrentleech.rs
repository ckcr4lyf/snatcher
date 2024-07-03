use std::{env::temp_dir, ffi::{OsStr, OsString}, io::Write, sync::Arc};

use futures::StreamExt;
use irc::{client::{data::Config, Client}, proto::Command};
use log::{debug, error, info, trace};
use serde_bencode::de;

use crate::{action::add_to_qbit, filters, torrent, trackers::Torrent};

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
    raw_torrent: Vec<u8>,
    // path: OsString,
    size: i64,
}

impl super::Torrent for TorrentleechTorrent {
    fn name(&self) -> &str {
        return &self.name;
    }

    fn path(&self) -> &OsStr {
        todo!()
        // return &self.path;
    }

    fn size(&self) -> i64 {
        return self.size
    }
}


impl super::Tracker for TorrentleechTracker {
    type Torrent = TorrentleechTorrent;

    async fn download(&self, torrent: Self::Torrent) -> Result<OsString, failure::Error> {
        let filename = format!("{}.torrent", &torrent.name);
        let p = temp_dir().as_path().join(&filename);
        let mut f = std::fs::File::create(&p).unwrap();
        
        match f.write_all(&torrent.raw_torrent) {
            Ok(_) => {
                debug!("wrote to file {}", filename);
                return Ok(p.into_os_string());
            },
            Err(e) => {
                error!("fail to write to file; {}", e);
                return Err(e.into());
            }
        }
    }

    async fn parse_message(&self, msg: &str) -> Option<Self::Torrent> {
        todo!()
    }

    async fn monitor(&self, filter: Arc<filters::Filter>) -> Result<(), failure::Error> {
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

    
        while let Some(message) = stream.next().await.transpose()? {
            let rss_key = self.rss_key.to_owned();
            let filter = filter.clone();

            tokio::spawn(async move {
                match message.command {
                    Command::PRIVMSG(_, p2) => {
                        let x = parse_message(&rss_key, &p2).await;
                        if let Some(x) = x {
                            debug!("Got new release: {:?}", x);

                            if filter.check(x) == true {
                                debug!("Passed filter, we should get it");
                            } else {
                                debug!("Did not pass filter")
                            }
                        } else {
                            error!("Filed to parse message: {}", p2);
                        }
                    },
                    _ => {
                        // noop
                    }
                }
            });
        }
    
        Ok(())
    }
}

async fn parse_message(rss_key: &str, msg: &str) -> Option<TorrentleechTorrent> {
    trace!("parse_message for {} ({:2X?})", msg, msg.as_bytes());

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

    let download_url = format!("https://www.torrentleech.org/rss/download/{}/{}/{}.torrent", id, rss_key, name_dot);

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

    return Some(TorrentleechTorrent{
        name: name_dot,
        uploader: msg[name_end_index+15..uploader_end_index].to_owned(),
        url,
        freeleech,
        id,
        raw_torrent: bytes.to_vec(),
        // path: p.into(),
        size: t.size(),
    });
}