use futures::StreamExt;
use irc::client::{data::Config, Client};
use log::{debug, info};

pub struct IptTracker {

}
#[derive(Debug)]
pub struct IptTorrent {

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

    async fn parse_message(&self, msg: &str) -> Option<Self::Torrent> {
        todo!();
    }

    async fn monitor(&self) -> Result<(), failure::Error> {
        let config = Config{
            nickname: Some("poiasd_autodl".to_owned()),
            server: Some("irc.iptorrents.com".to_owned()),
            port: Some(6667),
            channels: vec!["#ipt.announce".to_owned()],
            ..Config::default()
        };

        info!("Connecting to IRC...");

        let mut client = Client::from_config(config).await?;
        client.identify()?;
        let mut stream = client.stream()?;

        while let Some(message) = stream.next().await.transpose()? {
            debug!("Got message: {}", message);
        }

        Ok(())
    }
}