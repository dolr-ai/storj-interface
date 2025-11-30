FROM alpine:3.21.3

WORKDIR /app

# install uplink and ffmpeg
RUN apk add --no-cache curl ffmpeg
RUN curl -L https://github.com/storj/storj/releases/latest/download/uplink_linux_amd64.zip -o uplink_linux_amd64.zip
RUN unzip -o uplink_linux_amd64.zip
RUN install uplink /usr/local/bin/uplink

# use the build from the host machine
COPY target/x86_64-unknown-linux-musl/release/storj-interface .

EXPOSE 3000

ENTRYPOINT ["./storj-interface"]

