use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[cfg(feature = "clap")]
use clap::Parser;

const DEFAULT_LISTEN_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 53);

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
    pub log_ansi_color: bool,
}

#[cfg(feature = "clap")]
#[derive(Parser)]
#[clap(author = "Kirill K.")]
#[clap(version, about, long_about = None)]
struct Cli {
    /// Address of primary target server
    #[clap(value_parser)]
    primary: SocketAddr,

    /// Address of secondary target server
    #[clap(value_parser)]
    secondary: SocketAddr,

    /// Address that will be used to listen incoming UDP requests
    #[clap(value_parser, default_value_t = DEFAULT_LISTEN_ADDR)]
    listen: SocketAddr,

    /// Connection keepalive timeout in milliseconds
    #[arg(short, long, value_name = "MILLISECONDS", default_value_t = 500)]
    keepalive_timeout: u64,

    #[cfg(feature = "tracing")]
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    #[cfg(feature = "tracing")]
    #[arg(long, default_value_t = true)]
    log_ansi_color: bool,
}

impl Settings {
    pub fn new() -> Self {
        #[cfg(feature = "clap")]
        return Self::from_cli();
        #[cfg(not(feature = "clap"))]
        Self::from_env()
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
            log_level: cli.verbose.into(),
            #[cfg(feature = "tracing")]
            log_ansi_color: cli.log_ansi_color,
        }
    }

    #[cfg(not(feature = "clap"))]
    fn from_env() -> Self {
        Self {
            listen: get_from_env("LISTEN_ADDR", Some(DEFAULT_LISTEN_ADDR)),
            primary: get_from_env("PRIMARY_ADDR", None),
            secondary: get_from_env("SECONDARY_ADDR", None),
            keepalive_timeout: get_from_env("KEEPALIVE_TIMEOUT", Some(500)),
            #[cfg(feature = "tracing")]
            log_level: get_from_env("LOG_LEVEL", Some(tracing::metadata::LevelFilter::INFO)),
            #[cfg(feature = "tracing")]
            log_ansi_color: get_from_env("LOG_ANSI_COLOR", Some(true)),
        }
    }
}

fn get_from_env<T: std::str::FromStr>(name: &str, default: Option<T>) -> T {
    match std::env::var(name).ok() {
        Some(v) => v
            .parse()
            .unwrap_or_else(|_| panic!("Invalid value for environment variable {name}: {v}.")),
        None => default.unwrap_or_else(|| panic!("Environment variable {name} is not set.")),
    }
}
