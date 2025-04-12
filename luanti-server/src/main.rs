//! Luanti protocol implemented in Rust
// #![expect(clippy::expect_used, reason = "//TODO improve error handling")]

#![expect(
    clippy::todo,
    clippy::expect_used,
    reason = "//TODO remove before completion of the prototype"
)]

pub mod authentication;
mod client_connection;
mod server;
mod world;

use anyhow::bail;
use authentication::dummy::DummyAuthenticator;
use clap::ArgGroup;
use clap::Parser;
use server::LuantiWorldServer;
use std::net::SocketAddr;
use std::time::Duration;
use world::generation::flat::MapgenFlat;
use world::storage::dummy::DummyStorage;

/// luanti-shark - Luanti proxy that gives detailed inspection of protocol
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(ArgGroup::new("source").required(true).args(["listen", "bind"])))]
struct Args {
    /// Listen on port
    #[arg(group = "source", short, long)]
    listen: Option<u16>,

    /// Listen with specific bind address (ip:port)
    #[arg(group = "source", short, long)]
    bind: Option<SocketAddr>,

    /// Verbosity level (up to -vvv)
    #[arg(short, long, default_value_t = 0, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tokio::main makes rust-analyzer fragile,
    // so put the code in a separate place.
    real_main().await
}

async fn real_main() -> anyhow::Result<()> {
    // TODO make this configurable through command line arguments
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let args = Args::parse();

    let bind_addr: SocketAddr = if let Some(listen_port) = args.listen {
        // TODO(kawogi) re-enable IPv6 support
        if true {
            format!("0.0.0.0:{listen_port}").parse()?
        } else {
            format!("[::]:{listen_port}").parse()?
        }
    } else if let Some(bind_addr) = args.bind {
        bind_addr
    } else {
        bail!("One of --listen or --bind must be specified");
    };

    let world_generator = MapgenFlat;
    let storage = DummyStorage;

    let mut server = LuantiWorldServer::new(bind_addr, args.verbose);
    server.start(DummyAuthenticator);
    #[expect(
        clippy::infinite_loop,
        reason = "// TODO implement a cancellation mechanism"
    )]
    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}
