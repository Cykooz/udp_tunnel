use std::fmt::Display;
use std::io;

use thiserror::Error;
use tokio::net::{ToSocketAddrs, UdpSocket};

use crate::wait_timeout;
use crate::wait_timeout::Timeout;

pub struct UdpSocketWithTimeouts {
    pub socket: UdpSocket,
    pub connect_timeout: u64,
    pub send_timeout: u64,
    pub recv_timeout: u64,
}

impl UdpSocketWithTimeouts {
    pub async fn bind<A: ToSocketAddrs + Display>(addr: A) -> Result<Self, SocketError> {
        let socket = wait_timeout::wait(100, UdpSocket::bind(&addr), || {
            format!("bind socket to {addr}")
        })
        .await??;
        Ok(Self {
            socket,
            connect_timeout: 500,
            send_timeout: 500,
            recv_timeout: 0,
        })
    }

    pub async fn connect<A: ToSocketAddrs + Display>(&self, addr: A) -> Result<(), SocketError> {
        wait_timeout::wait(self.connect_timeout, self.socket.connect(&addr), || {
            format!("connect to address {addr}")
        })
        .await??;
        Ok(())
    }

    pub async fn send(&self, buf: &[u8]) -> Result<usize, SocketError> {
        let len = wait_timeout::wait(self.send_timeout, self.socket.send(buf), || "send datagram")
            .await??;
        Ok(len)
    }

    pub async fn send_to<A: ToSocketAddrs + Display>(
        &self,
        buf: &[u8],
        addr: A,
    ) -> Result<usize, SocketError> {
        let len = wait_timeout::wait(self.send_timeout, self.socket.send_to(buf, &addr), || {
            format!("send datagram to {addr}")
        })
        .await??;
        Ok(len)
    }

    pub async fn recv(&self, buf: &mut [u8]) -> Result<usize, SocketError> {
        let len = wait_timeout::wait(
            self.recv_timeout,
            self.socket.recv(buf),
            || "receive datagram",
        )
        .await??;
        Ok(len)
    }
}

#[derive(Error, Debug)]
pub enum SocketError {
    #[error(transparent)]
    Timeout(#[from] Timeout),
    #[error(transparent)]
    IoError(#[from] io::Error),
}
