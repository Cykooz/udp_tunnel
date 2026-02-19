use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use udp_tunnel::run_tunnel;

#[tokio::test]
async fn test_run_tunnel() {
    let env = TestEnv::new(500).await;

    env.client_socket.send(b"request").await.unwrap();

    let mut buf = [0u8; 1024];
    let (len, sender) = env.primary_socket.recv_from(&mut buf).await.unwrap();
    assert_eq!(&buf[..len], b"request");
    let len = env.secondary_socket.recv(&mut buf).await.unwrap();
    assert_eq!(&buf[..len], b"request");

    env.primary_socket
        .send_to(b"response", sender)
        .await
        .unwrap();
    let len = env.client_socket.recv(&mut buf).await.unwrap();
    assert_eq!(&buf[..len], b"response");
}

#[tokio::test]
async fn test_keepalive() {
    let env = TestEnv::new(50).await;
    env.client_socket.send(b"request").await.unwrap();

    let mut buf = [0u8; 1024];
    let (len, sender) = env.primary_socket.recv_from(&mut buf).await.unwrap();
    assert_eq!(&buf[..len], b"request");

    sleep(Duration::from_millis(100)).await;

    env.primary_socket
        .send_to(b"response", sender)
        .await
        .unwrap();

    sleep(Duration::from_millis(10)).await;
    let res = env.client_socket.try_recv(&mut buf);
    assert!(res.is_err()); // The tunnel already forgot about the client.
}

struct TestEnv {
    pub client_socket: UdpSocket,
    pub primary_socket: UdpSocket,
    pub secondary_socket: UdpSocket,
    task_handle: JoinHandle<anyhow::Result<()>>,
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        self.task_handle.abort();
    }
}

impl TestEnv {
    pub async fn new(keepalive: u64) -> Self {
        let primary_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let secondary_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let listen_addr = UdpSocket::bind("127.0.0.1:0")
            .await
            .unwrap()
            .local_addr()
            .unwrap();

        let task_handle = tokio::spawn(run_tunnel(
            listen_addr,
            primary_socket.local_addr().unwrap(),
            secondary_socket.local_addr().unwrap(),
            keepalive,
        ));

        let client_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        client_socket.connect(listen_addr).await.unwrap();

        Self {
            client_socket,
            primary_socket,
            secondary_socket,
            task_handle,
        }
    }
}
