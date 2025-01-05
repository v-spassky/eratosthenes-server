use crate::cli::Args;
use std::{net::SocketAddr, str::FromStr};
use url::Url;

pub fn fake_args() -> Args {
    Args {
        quickwit_url: Url::from_str("http://127.0.0.1:7280")
            .expect("Failed co construct fake Quickwit URL."),
        listen_address: SocketAddr::from_str("0.0.0.0:3030")
            .expect("Failed co construct fake listen address."),
        jwt_signing_key: String::from("testKey"),
    }
}
