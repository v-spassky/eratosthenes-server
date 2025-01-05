use clap::Parser;
use std::net::SocketAddr;
use url::Url;
#[cfg(test)]
pub mod tests;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long)]
    #[arg(default_value = "http://127.0.0.1:7280")]
    pub quickwit_url: Url,
    #[arg(long)]
    #[arg(default_value = "0.0.0.0:3030")]
    pub listen_address: SocketAddr,
    #[arg(long)]
    pub jwt_signing_key: String,
}
