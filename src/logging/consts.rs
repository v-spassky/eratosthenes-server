use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub const DEFAULT_CLIENT_IP: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
pub const DEFAULT_LOGGING_BUFFER_SIZE: usize = 500;
pub const QUICKWIT_LOGGING_CHANNEL_CAPACITY: usize = 512;
