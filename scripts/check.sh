#!/bin/bash
# ConHub Services Health Check (Linux/macOS)

echo "=== ConHub Services Health Check ==="

# Test Frontend
if curl -s http://localhost:3000 > /dev/null; then
    echo "✅ Frontend: Running"
else
    echo "❌ Frontend: Not responding"
fi

# Test Backend
if curl -s http://localhost:3001/health > /dev/null; then
    echo "✅ Backend: Running"
else
    echo "❌ Backend: Not responding"
fi

# Test LangChain Service
if curl -s http://localhost:3003/health > /dev/null; then
    echo "✅ LangChain Service: Running"
else
    echo "❌ LangChain Service: Not responding"
fi

# Test Haystack Service
if curl -s http://localhost:8001/health > /dev/null; then
    echo "✅ Haystack Service: Running"
else
    echo "❌ Haystack Service: Not responding"
fi

echo ""
echo "🔗 Services are running on:"
echo "   Frontend:  http://localhost:3000"
echo "   Backend:   http://localhost:3001"
echo "   LangChain: http://localhost:3003"
echo "   Haystack:  http://localhost:8001"