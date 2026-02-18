use std::collections::HashMap;
use std::net::SocketAddr;
use std::process;
use std::sync::Arc;

use tokio::signal;
use tokio::sync::mpsc;
#[cfg(feature = "tracing")]
pub use tracing::{debug, error, info};

use crate::settings::Settings;
use crate::udp_connection::{Connection, EMPTY_ADDR};
use crate::udp_socket::UdpSocketWithTimeouts;

#[macro_use]
mod macros;

mod settings;
mod udp_connection;
mod udp_socket;
mod wait_timeout;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::new();

    #[cfg(feature = "tracing")]
    {
        use tracing_subscriber::fmt::Subscriber;
        use tracing_subscriber::util::SubscriberInitExt;
        let builder = Subscriber::builder()
            .with_max_level(settings.log_level)
            .with_target(false)
            .without_time();

        let subscriber = builder.finish();
        subscriber.try_init()?;
    };

    tokio::spawn(run_tunnel(
        settings.listen,
        settings.primary,
        settings.secondary,
        settings.keepalive_timeout,
    ));

    // Wait for the Ctrl+C signal
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Received Ctrl+C, starting shutdown...");
            Ok(())
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {err}");
            // Exit with an error code if the signal handler couldn't be installed
            process::exit(1);
        }
    }
}

async fn run_tunnel(
    listen_addr: SocketAddr,
    primary_addr: SocketAddr,
    secondary_addr: SocketAddr,
    keepalive_timeout: u64,
) -> anyhow::Result<()> {
    let socket = UdpSocketWithTimeouts::bind(listen_addr).await?;
    let listen_socket = Arc::new(socket);
    info!(
        "UDP tunnel is listening on {listen_addr}. \
        Primary address: {primary_addr}, \
        secondary address: {secondary_addr}. \
        Keepalive timeout: {keepalive_timeout} ms."
    );
    info!("Press Ctrl+C to stop the tunnel.");

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
