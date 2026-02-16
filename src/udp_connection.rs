use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use thiserror::Error;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
#[cfg(feature = "tracing")]
pub use tracing::{debug, error};

use crate::udp_socket::{SocketError, UdpSocketWithTimeouts};
use crate::wait_timeout;
use crate::wait_timeout::Timeout;

pub type Connection = (JoinHandle<()>, mpsc::Sender<Arc<Vec<u8>>>);

pub const EMPTY_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);

pub fn create_new_connection(
    client_addr: SocketAddr,
    target_addr: SocketAddr,
    response_sender: mpsc::Sender<(SocketAddr, Vec<u8>)>,
    keepalive_timeout: u64,
) -> Connection {
    debug!("Create a new connection from {client_addr} to {target_addr}.");
    let (sender, receiver) = mpsc::channel::<Arc<Vec<u8>>>(256);
    let join_handler = tokio::spawn(async move {
        let res = process_connection(
            client_addr,
            target_addr,
            receiver,
            response_sender,
            keepalive_timeout,
        )
        .await;
        if let Err(e) = res {
            error!("{e}");
        }
    });
    (join_handler, sender)
}

async fn process_connection(
    client_addr: SocketAddr,
    target_addr: SocketAddr,
    input: mpsc::Receiver<Arc<Vec<u8>>>,
    output: mpsc::Sender<(SocketAddr, Vec<u8>)>,
    keepalive_timeout: u64,
) -> anyhow::Result<()> {
    let socket = Arc::new(UdpSocketWithTimeouts::bind(EMPTY_ADDR).await?);
    socket.connect(target_addr).await?;

    let read_handle = tokio::spawn(read_responses(
        socket.clone(),
        client_addr,
        target_addr,
        output,
    ));
    let result = send_requests(socket.clone(), input, keepalive_timeout).await;
    read_handle.abort();

    match result {
        Ok(_) => debug!("Tunnel from {client_addr} to {target_addr} closed."),
        Err(SendRequestsError::ReceiveRequestTimeout(Timeout { duration, .. })) => {
            debug!(
                "Tunnel from {client_addr} to {target_addr} \
                     was closed due to inactivity ({duration} ms)."
            )
        }
        Err(SendRequestsError::SendRequestError(e)) => {
            error!("Tunnel from {client_addr} to {target_addr} was closed due to error: {e}.")
        }
    }

    Ok(())
}

#[derive(Error, Debug)]
enum SendRequestsError {
    #[error(transparent)]
    ReceiveRequestTimeout(#[from] Timeout),
    #[error(transparent)]
    SendRequestError(#[from] SocketError),
}

async fn send_requests(
    socket: Arc<UdpSocketWithTimeouts>,
    mut requests: mpsc::Receiver<Arc<Vec<u8>>>,
    keepalive_timeout: u64,
) -> Result<(), SendRequestsError> {
    while let Some(request) = wait_timeout::wait(
        keepalive_timeout,
        requests.recv(),
        || "receive data from channel",
    )
    .await?
    {
        socket.send(&request).await?;
    }
    Ok(())
}

async fn read_responses(
    socket: Arc<UdpSocketWithTimeouts>,
    client_addr: SocketAddr,
    target_addr: SocketAddr,
    responses: mpsc::Sender<(SocketAddr, Vec<u8>)>,
) -> anyhow::Result<()> {
    let mut response_buf = vec![0u8; 65536];
    loop {
        let len = socket.recv(&mut response_buf).await?;
        debug!("Received response from {target_addr}, {len} bytes.");
        let response = response_buf[..len].to_vec();
        responses.send((client_addr, response)).await?;
    }
}
