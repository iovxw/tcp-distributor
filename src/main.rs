use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};

use rand::prelude::*;
use rand::rngs::SmallRng;
use tokio::net::{TcpListener, TcpStream};
use toml;

mod config;

static COUNT: AtomicU64 = AtomicU64::new(0);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config_path = std::env::args()
        .skip(1)
        .next()
        .expect("Please specify a config file");

    let config_file = std::fs::read_to_string(config_path)?;
    let config: config::Config = toml::de::from_str(&config_file)?;

    log::info!("Listen on: {}", config.listen);

    let listener = TcpListener::bind(config.listen).await?;
    let mut rng = SmallRng::from_entropy();

    loop {
        let (socket, _) = listener.accept().await?;
        let remote_addr = config.remotes[rng.gen_range(0..config.remotes.len())];
        log::debug!("new connection: {}", remote_addr);

        tokio::spawn(async move {
            if let Err(e) = do_forward(socket, remote_addr).await {
                log::error!("{} {}", remote_addr, e);
            }
        });
    }
}

async fn do_forward(mut local_socket: TcpStream, remote_addr: SocketAddr) -> std::io::Result<()> {
    let mut remote_socket = tokio::net::TcpStream::connect(remote_addr).await?;
    let c = COUNT.fetch_add(1, Ordering::AcqRel);
    log::debug!("connected: {}, COUNT: {}", remote_addr, c);
    struct Defer<T>(T)
    where
        T: FnMut();
    impl<T> Drop for Defer<T>
    where
        T: FnMut(),
    {
        fn drop(&mut self) {
            self.0();
        }
    }
    let _x = Defer(|| {
        let c = COUNT.fetch_sub(1, Ordering::AcqRel);
        log::debug!("disconnected: {}, COUNT: {}", remote_addr, c);
    });
    tokio::io::copy_bidirectional(&mut local_socket, &mut remote_socket).await?;
    Ok(())
}
