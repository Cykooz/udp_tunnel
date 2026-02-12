use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use clap::Parser;
use tokio::sync::mpsc;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::Subscriber;
use tracing_subscriber::util::SubscriberInitExt;
use udp_connection::{Connection, EMPTY_ADDR};
use udp_socket::UdpSocketWithTimeouts;

mod udp_connection;
mod udp_socket;
mod wait_timeout;

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
    #[clap(value_parser, default_value = "0.0.0.0:53")]
    listen: SocketAddr,

    /// Connection keepalive timeout in milliseconds
    #[arg(short, long, value_name = "MILLISECONDS", default_value_t = 500)]
    keepalive_timeout: u64,

    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();

    let builder = Subscriber::builder()
        .with_max_level(cli.verbose)
        .with_target(false)
        .without_time()
        .with_env_filter(EnvFilter::from_default_env());

    let subscriber = builder.finish();
    subscriber.try_init()?;

    run_tunnel(
        cli.listen,
        cli.primary,
        cli.secondary,
        cli.keepalive_timeout,
    )
    .await
}

async fn run_tunnel(
    listen_addr: SocketAddr,
    primary_addr: SocketAddr,
    secondary_addr: SocketAddr,
    keepalive_timeout: u64,
) -> anyhow::Result<()> {
    let socket = UdpSocketWithTimeouts::bind(listen_addr).await?;
    let listen_socket = Arc::new(socket);
    info!("UDP tunnel is listening on {listen_addr}.");

    let secondary_socket = Arc::new(UdpSocketWithTimeouts::bind(EMPTY_ADDR).await?);
    secondary_socket.connect(secondary_addr).await?;
    debug!("Connected to secondary address: {secondary_addr}.");

    let mut connections: HashMap<SocketAddr, Connection> = HashMap::new();

    let (response_sender, response_receiver) = mpsc::channel::<(SocketAddr, Vec<u8>)>(256);
    tokio::spawn(send_responses_to_client(
        listen_socket.clone(),
        response_receiver,
    ));

    let mut buf = vec![0u8; 65536];
    loop {
        let (len, client_addr) = listen_socket.socket.recv_from(&mut buf).await?;
        debug!("Received {len} bytes from {client_addr}.");

        // Remove finished connections.
        connections.retain(|_, (handle, _)| !handle.is_finished());

        let (_, primary_sender) = connections.entry(client_addr).or_insert_with(|| {
            udp_connection::create_new_connection(
                client_addr,
                primary_addr,
                response_sender.clone(),
                keepalive_timeout,
            )
        });

        let data = Arc::new(buf[..len].to_vec());
        debug!("Send datagram from {client_addr} to {primary_addr}.");
        primary_sender.send(data.clone()).await?;

        debug!("Send datagram from {client_addr} to {secondary_addr}.");
        let m_socket = secondary_socket.clone();
        tokio::spawn(async move { m_socket.send(&data).await });
    }
}

pub async fn send_responses_to_client(
    socket: Arc<UdpSocketWithTimeouts>,
    mut channel: mpsc::Receiver<(SocketAddr, Vec<u8>)>,
) -> anyhow::Result<()> {
    while let Some((addr, data)) = channel.recv().await {
        debug!("Send response to client {addr}.");
        socket.send_to(&data, &addr).await?;
    }
    Ok(())
}
