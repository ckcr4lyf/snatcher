use std::{
    env::temp_dir,
    ffi::{OsStr, OsString},
    io::Write,
    sync::Arc,
};

use futures::StreamExt;
use irc::{
    client::{data::Config, Client},
    proto::Command,
};
use log::{debug, error, info, trace};
use serde_bencode::de;

use crate::{
    action::{add_to_qbit, add_to_qbit_v2},
    filters, torrent,
    trackers::Torrent,
    TorrentleechConfig,
};

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
        return self.size;
    }
}

async fn parse_message(rss_key: &str, msg: &str) -> Option<TorrentleechTorrent> {
    trace!("parse_message for {} ({:2X?})", msg, msg.as_bytes());

    let message_fields = parse_fields_from_msg(msg)?;

    let download_url = format!(
        "https://www.torrentleech.org/rss/download/{}/{}/{}.torrent",
        message_fields.id, rss_key, message_fields.name_dot,
    );

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
        }
        Err(e) => {
            error!("Error bencode-decoding torrent: {}", e);
            return None;
        }
    };

    return Some(TorrentleechTorrent {
        name: message_fields.name_dot,
        uploader: message_fields.uploader,
        url: message_fields.url,
        freeleech: message_fields.freeleech,
        id: message_fields.id,
        raw_torrent: bytes.to_vec(),
        // path: p.into(),
        size: t.size(),
    });
}

async fn download_torrent(torrent: &TorrentleechTorrent) -> Result<OsString, failure::Error> {
    let filename = format!("{}.torrent", &torrent.name);
    let p = temp_dir().as_path().join(&filename);
    let mut f = std::fs::File::create(&p).unwrap();

    match f.write_all(&torrent.raw_torrent) {
        Ok(_) => {
            debug!("wrote to file {}", filename);
            return Ok(p.into_os_string());
        }
        Err(e) => {
            error!("fail to write to file; {}", e);
            return Err(e.into());
        }
    }
}

pub async fn monitor(tracker_config: &'static TorrentleechConfig) -> Result<(), failure::Error> {
    let config = Config {
        nickname: Some(tracker_config.username.to_owned()),
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

    let filter: &'static filters::Filter = Box::leak(Box::new(filters::Filter {
        valid_regexes: regex::RegexSet::new(&tracker_config.filter.valid_regexes).unwrap(),
        size_max: tracker_config.filter.max_size,
    }));

    while let Some(message) = stream.next().await.transpose()? {
        tokio::spawn(async move {
            match message.command {
                Command::PRIVMSG(_, p2) => {
                    let x = parse_message(&tracker_config.rss_key, &p2).await;
                    if let Some(x) = x {
                        debug!("Got new release: {} (Size: {})", x.name, x.size);

                        if filter.check(&x) == true {
                            debug!("Passed filter, we should get it");

                            match download_torrent(&x).await {
                                Ok(p) => {
                                    debug!("Downloaded to {:?}", &p);
                                    add_to_qbit_v2(&p);
                                }
                                Err(e) => {
                                    error!("Failed to download: {}", e)
                                }
                            }
                        } else {
                            debug!("Did not pass filter")
                        }
                    } else {
                        error!("Filed to parse message: {}", p2);
                    }
                }
                _ => {
                    // noop
                }
            }
        });
    }

    Ok(())
}

#[derive(PartialEq, Eq, Debug)]
struct MessagedParsedFields {
    name_dot: String,
    uploader: String,
    id: String,
    url: String,
    freeleech: bool,
}

fn parse_fields_from_msg(msg: &str) -> Option<MessagedParsedFields> {
    let name_start_index = msg.find("Name:'")?;
    let name_end_index = msg.find("' uploaded by '")?;

    // lenn is number of characters after uploader end before the URL
    let (uploader_end_index, freeleech, lenn) = match msg.rfind("' - ") {
        Some(pos) => (pos, false, 10),
        None => match msg.find("' freeleech - ") {
            Some(pos) => (pos, true, 20),
            None => return None,
        },
    };

    let name_original: String = msg[name_start_index + 6..name_end_index].to_owned();
    let name_dot = name_original.replace(" ", ".");
    let url: String = msg[uploader_end_index + lenn..].to_owned();

    let id_index = url.rfind("/")?;
    let id: String = url[id_index + 1..].to_owned();

    return Some(MessagedParsedFields{
        name_dot: name_dot,
        uploader: msg[name_end_index + 15..uploader_end_index].to_owned(),
        id: id,
        url: url,
        freeleech: freeleech,
    })
}

#[cfg(test)]
mod tests {
    use crate::trackers::torrentleech::*;


    #[test]
    fn test_parse_freeleech(){
        let mock_message = "00,04New Torrent Announcement:00,12 <Movies :: Bluray>  Name:'A Kid Like Jake 2018 1080p BluRay REMUX AVC DTS-HD MA 5 1-PiRAMiDHEAD' uploaded by 'Wardaddy007' freeleech - 01,15 https://www.torrentleech.org/torrent/241304583";
        assert_eq!(parse_fields_from_msg(mock_message), Some(MessagedParsedFields{
            name_dot: "A.Kid.Like.Jake.2018.1080p.BluRay.REMUX.AVC.DTS-HD.MA.5.1-PiRAMiDHEAD".to_string(),
            uploader: "Wardaddy007".to_string(),
            id: "241304583".to_string(),
            url: "https://www.torrentleech.org/torrent/241304583".to_string(),
            freeleech: true
        }))
    }

    #[test]
    fn test_parse_normal(){
        let mock_message = "00,04New Torrent Announcement:00,12 <TV :: Episodes HD>  Name:'The Kardashians S05E06 1080p WEB h264-ETHEL' uploaded by 'Anonymous' - 01,15 https://www.torrentleech.org/torrent/241304584";
        assert_eq!(parse_fields_from_msg(mock_message), Some(MessagedParsedFields{
            name_dot: "The.Kardashians.S05E06.1080p.WEB.h264-ETHEL".to_string(),
            uploader: "Anonymous".to_string(),
            id: "241304584".to_string(),
            url: "https://www.torrentleech.org/torrent/241304584".to_string(),
            freeleech: false
        }))
    }

    #[test]
    fn test_parse_dodgy_japanese(){
        let mock_message = "00,04New Torrent Announcement:00,12 <Music :: Audio>  Name:' - 華昇リ (2020, FLAC) [BeesKnees]' uploaded by 'lukemattle' - 01,15 https://www.torrentleech.org/torrent/241452475";
        assert_eq!(parse_fields_from_msg(mock_message), Some(MessagedParsedFields{
            name_dot: ".-.華昇リ.(2020,.FLAC).[BeesKnees]".to_string(),
            uploader: "lukemattle".to_string(),
            id: "241452475".to_string(),
            url: "https://www.torrentleech.org/torrent/241452475".to_string(),
            freeleech: false
        }))
    }
}