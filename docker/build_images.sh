#!/usr/bin/env bash
set -e

CURDIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "${CURDIR}"

docker buildx build --platform linux/amd64 \
    -t udp_tunnel:latest \
    -f Dockerfile .
