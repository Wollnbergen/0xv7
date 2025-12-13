#!/bin/bash
echo "🚀 Launching Sultan Chain..."
cd api && node sultan_api.js &
echo "✅ API running at http://localhost:3030"
echo "🌐 Public: https://orange-telegram-pj6qgwgv59jjfrj9j-3030.app.github.dev/"
wait
