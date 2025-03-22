//! Luanti protocol implemented in Rust
#![expect(clippy::expect_used, reason = "//TODO improve error handling")]

mod proxy;

use anyhow::bail;
use clap::ArgGroup;
use clap::Parser;
use log::info;
use luanti_protocol::audit_on;
use proxy::LuantiProxy;
use std::net::SocketAddr;
use std::time::Duration;

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

    /// Target server (address:port)
    #[arg(short, long, required = true)]
    target: SocketAddr,

    /// Verbosity level (up to -vvv)
    #[arg(short, long, default_value_t = 0, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Enable audit mode
    #[arg(short, long, default_value_t = false)]
    audit: bool,
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
        .filter_level(log::LevelFilter::Debug)
        .init();

    let args = Args::parse();

    if args.audit {
        audit_on();
        info!("Auditing is ON.");
        info!("Proxy will terminate if an invalid packet is received,");
        info!("or if serialization/deserialization do not match exactly.");
    }

    let bind_addr: SocketAddr = if let Some(listen_port) = args.listen {
        if args.target.is_ipv4() {
            format!("0.0.0.0:{listen_port}").parse()?
        } else {
            format!("[::]:{listen_port}").parse()?
        }
    } else if let Some(bind_addr) = args.bind {
        bind_addr
    } else {
        bail!("One of --listen or --bind must be specified");
    };

    let _proxy = LuantiProxy::new(bind_addr, args.target, args.verbose);
    #[expect(
        clippy::infinite_loop,
        reason = "// TODO implement a cancellation mechanism"
    )]
    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}
