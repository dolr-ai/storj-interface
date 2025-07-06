#!/bin/bash

set -e

echo "🚀 Setting up Sia services..."

# Check if docker-compose is available
if ! command -v docker-compose &> /dev/null; then
    echo "❌ docker-compose is required but not installed."
    exit 1
fi

# Check if .env file exists
if [ ! -f .env ]; then
    echo "📝 Creating .env file from template..."
    cp .env.example .env
    echo "⚠️  Please edit .env file with your actual configuration before continuing"
    echo "   Especially update the seed phrases and API passwords!"
    exit 1
fi

echo "🔧 Starting Sia services..."
docker-compose up -d siad renterd walletd

echo "⏳ Waiting for services to start..."
sleep 30

echo "🔍 Checking service status..."
docker-compose ps

# Check if renterd is responding
echo "🧪 Testing renterd connection..."
if curl -f -H "Authorization: Basic $(echo -n :${RENTERD_API_PASSWORD:-changeme123} | base64)" \
    http://localhost:9980/api/autopilot/config 2>/dev/null; then
    echo "✅ renterd is running and accessible"
else
    echo "❌ renterd is not responding. Check logs with: docker-compose logs renterd"
fi

# Check if walletd is responding
echo "🧪 Testing walletd connection..."
if curl -f -H "Authorization: Basic $(echo -n :${WALLETD_API_PASSWORD:-changeme123} | base64)" \
    http://localhost:9983/api/consensus/network 2>/dev/null; then
    echo "✅ walletd is running and accessible"
else
    echo "❌ walletd is not responding. Check logs with: docker-compose logs walletd"
fi

echo ""
echo "📋 Next steps:"
echo "1. Wait for Sia consensus to sync (this can take a while)"
echo "2. Create wallets using the walletd API"
echo "3. Fund your wallets with Siacoin"
echo "4. Update .env with wallet IDs"
echo "5. Build and run the sia-interface service"
echo ""
echo "🔗 Service URLs:"
echo "   - renterd:  http://localhost:9980"
echo "   - walletd:  http://localhost:9983"
echo "   - siad:     http://localhost:9981"
echo ""
echo "📖 View logs with: docker-compose logs -f [service-name]"