# Stage 1: Builder stage to fetch the Tini binary
FROM --platform=$BUILDPLATFORM alpine:latest AS builder
ARG TARGETPLATFORM
ARG TARGETARCH
ARG TARGETOS
ARG TARGETVARIANT

# Set Tini version (check for the latest version on the Tini GitHub releases page)
ARG TINI_VERSION=v0.19.0

# Download the static Tini binary
RUN apk add --no-cache curl openssl
RUN curl -L -o /tini https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini-static-$TARGETARCH

COPY $TARGETARCH/udp_tunnel /

# Make the binary executable
RUN chmod +x /tini /udp_tunnel

# ===================
# Stage 2: Final scratch image
FROM --platform=$BUILDPLATFORM scratch
ARG TARGETPLATFORM
ARG TARGETARCH
ARG TARGETOS
ARG TARGETVARIANT

# Copy the Tini binary from the builder stage to the final image
COPY --from=builder /tini /udp_tunnel /

ENV LISTEN_ADDR="0.0.0.0:53"
ENV PRIMARY_ADDR="8.8.8.8:53"
ENV SECONDARY_ADDR="192.168.0.1:53"
ENV KEEPALIVE_TIMEOUT=500
ENV LOG_LEVEL="info"

ENTRYPOINT ["/tini", "--"]
CMD ["/udp_tunnel"]
