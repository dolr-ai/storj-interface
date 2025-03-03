# Build Stage
FROM rust:latest AS builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# Minimal Runtime Stage
FROM alpine:3.21.3
WORKDIR /root/
COPY --from=builder /usr/src/app/target/release/storj-interface /usr/bin/storj-interface
RUN apk add --no-cache curl
RUN curl -L https://github.com/storj/storj/releases/latest/download/uplink_linux_amd64.zip -o uplink_linux_amd64.zip
RUN unzip -o uplink_linux_amd64.zip
RUN install uplink /usr/local/bin/uplink
EXPOSE 3000
ENTRYPOINT ["/usr/bin/storj-interface"]

