use std::net::SocketAddr;

use rand::prelude::*;
use rand::rngs::SmallRng;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use toml;

mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = std::env::args()
        .skip(1)
        .next()
        .expect("Please specify a config file");

    let config_file = std::fs::read_to_string(config_path)?;
    let config: config::Config = toml::de::from_str(&config_file)?;

    dbg!(config.listen);

    let listener = TcpListener::bind(config.listen).await?;
    let mut rng = SmallRng::from_entropy();

    loop {
        let (socket, _) = listener.accept().await?;
        let remote_addr = config.remotes[rng.gen_range(0..config.remotes.len())];
        dbg!(remote_addr);

        tokio::spawn(async move {
            if let Err(e) = do_forward(socket, remote_addr).await {
                dbg!(e);
            }
        });
    }
}

async fn do_forward(mut local_socket: TcpStream, remote_addr: SocketAddr) -> std::io::Result<()> {
    let mut remote_socket = tokio::net::TcpStream::connect(remote_addr).await?;
    dbg!();
    tokio::io::copy_bidirectional(&mut local_socket, &mut remote_socket).await?;
    dbg!();
    Ok(())
}
