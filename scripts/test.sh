#!/bin/bash
# Test script for ConHub services (Linux/macOS)

echo "=== ConHub Services Health Check ==="

# Test Frontend
echo -n "🔄 Testing Frontend... "
if response=$(curl -s http://localhost:3000 2>/dev/null); then
    echo "✅ Running"
else
    echo "❌ Not responding"
fi

# Test Backend
echo -n "🔄 Testing Backend... "
if response=$(curl -s http://localhost:3001/health 2>/dev/null); then
    echo "✅ Running"
    echo "   Response: $response"
else
    echo "❌ Not responding"
fi

# Test LangChain Service
echo -n "🔄 Testing LangChain Service... "
if response=$(curl -s http://localhost:3003/health 2>/dev/null); then
    echo "✅ Running"
    echo "   Response: $response"
else
    echo "❌ Not responding"
fi

# Test Haystack Service
echo -n "🔄 Testing Haystack Service... "
if response=$(curl -s http://localhost:8001/health 2>/dev/null); then
    echo "✅ Running"
    echo "   Response: $response"
else
    echo "❌ Not responding"
fi

echo ""
echo "=== Service Endpoints ==="
echo "🔗 Frontend: http://localhost:3000"
echo "🔗 Backend: http://localhost:3001"
echo "   - Health: GET /health"
echo "🔗 LangChain Service: http://localhost:3003"
echo "   - Health: GET /health"
echo "   - Index Repository: POST /index/repository"
echo "   - Search: POST /search"
echo "🔗 Haystack Service: http://localhost:8001"
echo "   - Health: GET /health"
echo "   - Upload Document: POST /documents/upload"
echo "   - Search: POST /search"