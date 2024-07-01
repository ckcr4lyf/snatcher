use futures::StreamExt;
use irc::{client::{data::Config, Client}, proto::Command};
use log::{debug, error, info, trace};

pub struct IptTracker {

}
#[derive(Debug)]
pub struct IptTorrent {
    name: String,
    id: String,
    freeleech: bool,
    size: String, // TODO: Parsing logic?
}

impl super::Torrent for IptTorrent {
    fn name(&self) -> &str {
        todo!()
    }

    fn path(&self) -> &std::ffi::OsStr {
        todo!()
    }

    fn size(&self) -> i64 {
        todo!()
    }
}

impl IptTracker {
    pub fn new() -> Self {
        IptTracker{
        
        }
    }
}

impl super::Tracker for IptTracker {
    type Torrent = IptTorrent;

    // 5[Music/Packs]10 MP3 0Day 2024-06-30 4FREELEECH - https://www.iptorrents.com/details.php?id=6013681 -4 38.63 GB
    // 5[Anime]10 [Moozzi2] Shomin Sample - 12 END (BD 1920x1080 x 264 FLACx2) - https://www.iptorrents.com/details.php?id=6013682 -4 1.14 GB
    // Althought not visible in the "string", IRC formatting means there are control characters beteen e.g. the "-" and "4" (e.g. 0x03)
    // so we cant use e.g find("-4")
    async fn parse_message(&self, msg: &str) -> Option<Self::Torrent> {
        trace!("parse_message for {} ({:2X?})", msg, msg.as_bytes());

        let name_start_index = msg.find("]")? + 5;

        // First check if we can find FREELEECH, if not then normal ending
        let (name_end_index, freeleech, https_index) = match msg.find("4FREELEECH - https") {
            Some(pos) => (pos-2, true, pos + 13),
            None => {
                let pos = msg.find("https://www.iptorrents")?;
                (pos - 4, false, pos)
            }
        };

        let size_index = https_index + msg[https_index..].find(" ")? + 5;
        let name = msg[name_start_index..name_end_index].to_owned();
        let torrent_id = msg[https_index+42..size_index-5].to_owned();
        let size = msg[size_index..].to_owned();

        Some(IptTorrent{
            name,
            id: torrent_id,
            freeleech,
            size,
        })
    }

    async fn monitor(&self) -> Result<(), failure::Error> {
        let config = Config{
            nickname: Some("snatcherdev_bot".to_owned()),
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

        while let Some(message) = stream.next().await.transpose()? {
            match message.command {
                Command::PRIVMSG(_, p2) => {
                    debug!("Got message: {}", p2);
                    if let Some(x) = self.parse_message(&p2).await {
                        debug!("Got new release: {:?}", x);
                    } else {
                        error!("Failed to parse message: {}", p2);
                    }
                },
                _ => {
                    // noop
                }
            }
        }

        Ok(())
    }
}