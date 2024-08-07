use std::{env::temp_dir, fmt::format, io::Write, sync::Arc};

use futures::StreamExt;
use irc::{
    client::{data::Config, Client},
    proto::Command,
};
use log::{debug, error, info, trace};

use crate::{
    action::{add_to_qbit, add_to_qbit_v2},
    filters, IptConfig,
};
#[derive(Debug, Clone)]
pub struct IptTorrent {
    name: String,
    id: String,
    freeleech: bool,
    size: String, // TODO: Parsing logic?
}

impl super::Torrent for IptTorrent {
    fn name(&self) -> &str {
        return self.name.as_str();
    }

    fn path(&self) -> &std::ffi::OsStr {
        todo!()
    }

    fn size(&self) -> i64 {
        let parts: Vec<&str> = self.size.split(" ").collect();
        trace!("Got parts as {:?}", parts);

        let float_part: f32 = parts[0].parse::<f32>().unwrap();

        let multiplier = match parts[1] {
            "MB" => 1 << 20,
            "GB" => 1 << 30,
            _ => 1 << 10,
        };

        return (float_part * (multiplier as f32)) as i64;
    }
}

async fn parse_message(msg: &str) -> Option<IptTorrent> {
    trace!("parse_message for {} ({:2X?})", msg, msg.as_bytes());

    let name_start_index = msg.find("]")? + 5;

    // First check if we can find FREELEECH, if not then normal ending
    let (name_end_index, freeleech, https_index) = match msg.find("4FREELEECH - https") {
        Some(pos) => (pos - 2, true, pos + 13),
        None => {
            let pos = msg.find("https://www.iptorrents")?;
            (pos - 4, false, pos)
        }
    };

    let size_index = https_index + msg[https_index..].find(" ")? + 5;
    let name = msg[name_start_index..name_end_index].to_owned(); // This is name WITH spaces
    let torrent_id = msg[https_index + 42..size_index - 5].to_owned();
    let size = msg[size_index..].to_owned();

    Some(IptTorrent {
        name,
        id: torrent_id,
        freeleech,
        size,
    })
}

async fn download_torrent(
    passkey: &str,
    torrent: &IptTorrent,
) -> Result<std::ffi::OsString, failure::Error> {
    let download_url = format!(
        "https://iptorrents.com/download.php/{}/{}.torrent?torrent_pass={}",
        torrent.id, torrent.name, passkey
    );

    trace!("Going to download torrent from {}", download_url);
    let res = reqwest::get(download_url).await;

    let response = match res {
        Ok(v) => v,
        Err(e) => {
            error!("Got error from HTTP request: {}", e);
            return Err(e.into());
        }
    };

    trace!("Got HTTP {} from IPT", response.status());
    let bytes = response.bytes().await.unwrap();

    // No need to bencode-decode here...
    let filename = format!("{}.torrent", torrent.name);
    let p = temp_dir().as_path().join(&filename);
    let mut f = std::fs::File::create(&p).unwrap();

    match f.write_all(&bytes) {
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

pub async fn ipt_monitor(tracker_config: &'static IptConfig) -> Result<(), failure::Error> {
    let config = Config {
        nickname: Some(tracker_config.username.to_owned()),
        server: Some("irc.iptorrents.com".to_owned()),
        port: Some(6667),
        channels: vec!["#ipt.announce".to_owned()],
        use_tls: Some(false),
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
                    debug!("Got message: {}", p2);
                    if let Some(x) = parse_message(&p2).await {
                        debug!("Got new release: {} (Size: {})", x.name, x.size);

                        if filter.check(&x) == true {
                            debug!("Passed filter, we should get it");

                            match download_torrent(&tracker_config.passkey, &x).await {
                                Ok(p) => {
                                    debug!("Downloaded to {:?}", &p);
                                    add_to_qbit_v2(&p);
                                }
                                Err(e) => {
                                    error!("Failed to download: {}", e)
                                }
                            }
                        } else {
                            debug!("Did not pass filter");
                        }
                    } else {
                        error!("Failed to parse message: {}", p2);
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
