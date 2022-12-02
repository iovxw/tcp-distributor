use std::net::IpAddr;
use std::net::SocketAddr;

use ipnet::IpNet;
use serde::Deserialize;
use serde_with::{serde_as, DeserializeAs, OneOrMany};

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct Config {
    pub listen: SocketAddr,
    #[serde_as(deserialize_as = "Remotes")]
    pub remotes: Vec<SocketAddr>,
}

#[serde_as]
#[derive(Deserialize, Debug)]
struct Remote {
    #[serde_as(as = "OneOrMany<_>")]
    ip: Vec<IpNet>,
    #[serde_as(as = "OneOrMany<_>")]
    port: Vec<u16>,
}

impl Remote {
    fn expand(&self) -> impl Iterator<Item = SocketAddr> + '_ {
        self.ip
            .iter()
            .map(|ip: &IpNet| ip.hosts())
            .flatten()
            .map(|ip: IpAddr| {
                self.port
                    .iter()
                    .map(move |&port: &u16| SocketAddr::from((ip, port)))
            })
            .flatten()
    }
}

#[derive(Deserialize, Debug)]
struct Remotes(Vec<Remote>);

impl Remotes {
    fn to_socket_addrs(&self) -> Vec<SocketAddr> {
        self.0
            .iter()
            .map(|remote| remote.expand())
            .flatten()
            .collect()
    }
}

impl<'de> DeserializeAs<'de, Vec<SocketAddr>> for Remotes {
    fn deserialize_as<D>(deserializer: D) -> Result<Vec<SocketAddr>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let r = Remotes::deserialize(deserializer)?;
        Ok(r.to_socket_addrs())
    }
}
