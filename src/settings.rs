use std::net::SocketAddr;

#[cfg(feature = "clap")]
use clap::Parser;

use crate::socket_addr_v4;

const DEFAULT_LISTEN_ADDR: SocketAddr = socket_addr_v4(0, 0, 0, 0, 53);
const DEFAULT_PRIMARY_ADDR: SocketAddr = socket_addr_v4(8, 8, 8, 8, 53);
const DEFAULT_SECONDARY_ADDR: SocketAddr = socket_addr_v4(192, 168, 0, 1, 53);

pub struct Settings {
    /// Address that will be used to listen incoming UDP requests
    pub listen: SocketAddr,
    /// Address of primary target server
    pub primary: SocketAddr,
    /// Address of secondary target server
    pub secondary: SocketAddr,
    /// Connection keepalive timeout in milliseconds
    pub keepalive_timeout: u64,
    #[cfg(feature = "tracing")]
    pub log_level: tracing::metadata::LevelFilter,
    #[cfg(feature = "tracing")]
    pub without_ansi_color: bool,
}

#[cfg(feature = "clap")]
#[derive(Parser)]
#[clap(author = "Kirill K.")]
#[clap(version, about, long_about = None)]
struct Cli {
    /// Address that will be used to listen incoming UDP requests
    #[clap(short, long, env = "LISTEN_ADDR", default_value_t = DEFAULT_LISTEN_ADDR)]
    listen: SocketAddr,

    /// Address of primary target server
    #[clap(short, long, env = "PRIMARY_ADDR", default_value_t = DEFAULT_PRIMARY_ADDR)]
    primary: SocketAddr,

    /// Address of secondary target server
    #[clap(short, long, env = "SECONDARY_ADDR", default_value_t = DEFAULT_SECONDARY_ADDR)]
    secondary: SocketAddr,

    /// Connection keepalive timeout in milliseconds
    #[arg(
        short,
        long,
        value_name = "MILLISECONDS",
        env = "KEEPALIVE_TIMEOUT",
        default_value_t = 500
    )]
    keepalive_timeout: u64,

    #[cfg(feature = "tracing")]
    #[clap(long, env = "LOG_LEVEL", default_value = "info")]
    log_level: tracing::metadata::LevelFilter,

    #[cfg(feature = "tracing")]
    #[arg(long, env = "WITHOUT_ANSI_COLOR", default_value_t = false)]
    without_ansi_color: bool,
}

impl Default for Settings {
    fn default() -> Self {
        #[cfg(feature = "clap")]
        return Self::from_cli();
        #[cfg(not(feature = "clap"))]
        Self::from_env()
    }
}

impl Settings {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(feature = "clap")]
    fn from_cli() -> Self {
        let cli: Cli = Cli::parse();
        Self {
            listen: cli.listen,
            primary: cli.primary,
            secondary: cli.secondary,
            keepalive_timeout: cli.keepalive_timeout,
            #[cfg(feature = "tracing")]
            log_level: cli.log_level,
            #[cfg(feature = "tracing")]
            without_ansi_color: cli.without_ansi_color,
        }
    }

    #[cfg(not(feature = "clap"))]
    fn from_env() -> Self {
        Self {
            listen: get_from_env("LISTEN_ADDR", Some(DEFAULT_LISTEN_ADDR)),
            primary: get_from_env("PRIMARY_ADDR", Some(DEFAULT_PRIMARY_ADDR)),
            secondary: get_from_env("SECONDARY_ADDR", Some(DEFAULT_SECONDARY_ADDR)),
            keepalive_timeout: get_from_env("KEEPALIVE_TIMEOUT", Some(500)),
            #[cfg(feature = "tracing")]
            log_level: get_from_env("LOG_LEVEL", Some(tracing::metadata::LevelFilter::INFO)),
            #[cfg(feature = "tracing")]
            without_ansi_color: std::env::var("WITHOUT_ANSI_COLOR").is_ok(),
        }
    }
}

#[cfg(not(feature = "clap"))]
fn get_from_env<T: std::str::FromStr>(name: &str, default: Option<T>) -> T {
    match std::env::var(name).ok() {
        Some(v) => v
            .parse()
            .unwrap_or_else(|_| panic!("Invalid value for environment variable {name}: {v}.")),
        None => default.unwrap_or_else(|| panic!("Environment variable {name} is not set.")),
    }
}
