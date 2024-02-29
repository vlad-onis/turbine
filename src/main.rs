mod config;
mod http;
mod resolver;
mod server;

use anyhow::Result as AnyhowResult;
use clap::Parser;

use crate::config::{Args, Config};
use crate::server::Server;

fn main() -> AnyhowResult<()> {
    let args = Args::parse();
    println!("{:?}", args);
    let config = Config::new(args.config_file)?;
    {
        Server::run(config)?;
    }

    Ok(())
}
