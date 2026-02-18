# udp_tunnel

[![github](https://img.shields.io/badge/github-Cykooz%2Fudp__tunnel-8da0cb?logo=github)](https://github.com/Cykooz/udp_tunnel)

Application for tunneling UDP traffic between two hosts with function of mirroring
input traffic to secondary address.

[CHANGELOG](https://github.com/Cykooz/udp_tunnel/blob/main/CHANGELOG.md)

## Environment variables

`LISTEN_ADDR`
: Address to listen for incoming UDP packets (default: `0.0.0.0:53`).

`PRIMARY_ADDR`
: Primary address (<host>:<port>) for sending UDP packets that have been sent by
the client to `LISTEN_ADDR`.
All packets received from this address will be sent back to the client.

`SECONDARY_ADDR`
: Secondary address (<host>:<port>) for sending UDP packets that have been sent by
the client to `LISTEN_ADDR`.
Packets received from this address are ignored.

`KEEPALIVE_TIMEOUT`
: Keepalive timeout in milliseconds (default: `500`).
This timeout is used to keep mapping between the client's port that is used
to send packets to `LISTEN_ADDR` and the port that is used to send these
packets to `PRIMARY_ADDR`.

`LOG_LEVEL`
: Maximal level of log messages. Available values: `debug`, `info`, `error`
(default: `info`).

`LOG_ANSI_COLOR`
: Enable or disable ANSI color in log messages (default: `true`).
