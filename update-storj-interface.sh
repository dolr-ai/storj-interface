#!/bin/bash

set -e

echo "Downloading latest binary"
wget https://github.com/dolr-ai/storj-interface/releases/latest/download/storj-interface -O /tmp/storj-interface

echo "Installing the binary"
install -m 755 /tmp/storj-interface /usr/local/bin/storj-interface

echo "Requesting a restart with systemd"
systemctl restart storj-interface && \
systemctl is-active --wait storj-interface && \
echo "waiting for startup" && \
sleep 10 && \
systemctl is-active --quiet storj-interface
