#!/bin/bash

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║         SULTAN CHAIN - MAKING SERVICES PUBLIC                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Kill any existing processes
pkill -f "python.*3030" 2>/dev/null
pkill -f "sultan_demo" 2>/dev/null
sleep 2

# Start the demo server with proper binding
echo "🚀 Starting Sultan Demo Server on all interfaces..."
python3 - << 'PYTHON' &
from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import uuid
from datetime import datetime

class SultanHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        return  # Suppress logs
        
    def do_OPTIONS(self):
        self.send_response(200)
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type, Authorization')
        self.end_headers()
        
    def do_GET(self):
        if self.path == '/':
            self.send_response(200)
            self.send_header('Content-Type', 'text/html')
            self.end_headers()
            html = '''
<!DOCTYPE html>
<html>
<head>
    <title>Sultan Chain API</title>
    <style>
        body { font-family: Arial; padding: 20px; background: #1a1a2e; color: white; }
        .container { max-width: 800px; margin: 0 auto; }
        h1 { color: #667eea; }
        .endpoint { background: #16213e; padding: 15px; margin: 10px 0; border-radius: 8px; }
        code { background: #0f3460; padding: 2px 6px; border-radius: 3px; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 Sultan Chain API</h1>
        <p>Zero Gas Fees • 13.33% APY • Mobile Validators</p>
        
        <div class="endpoint">
            <h3>Chain Status</h3>
            <code>POST /</code>
            <pre>{"jsonrpc":"2.0","method":"chain_status","id":1}</pre>
        </div>
        
        <div class="endpoint">
            <h3>Create Wallet</h3>
            <code>POST /</code>
            <pre>{"jsonrpc":"2.0","method":"wallet_create","params":["username"],"id":1}</pre>
        </div>
        
        <div class="endpoint">
            <h3>Transfer (Zero Fees!)</h3>
            <code>POST /</code>
            <pre>{"jsonrpc":"2.0","method":"token_transfer","params":["from","to",100],"id":1}</pre>
        </div>
    </div>
</body>
</html>
            '''
            self.wfile.write(html.encode())
        elif self.path == '/health':
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            self.wfile.write(json.dumps({"status": "healthy", "chain": "sultan"}).encode())
        else:
            self.send_error(404)
            
    def do_POST(self):
        try:
            content_length = self.headers.get('Content-Length')
            if content_length:
                post_data = self.rfile.read(int(content_length))
                request = json.loads(post_data)
            else:
                request = {"method": "unknown"}
            
            method = request.get('method', '')
            params = request.get('params', [])
            request_id = request.get('id', 1)
            
            # Handle methods
            if method == 'chain_status':
                result = {
                    'name': 'Sultan Chain',
                    'height': 123456,
                    'validators': 100,
                    'mobile_validators': 40,
                    'tps': 10000,
                    'zero_fees': True,
                    'ibc_enabled': True
                }
            elif method == 'wallet_create':
                result = {
                    'address': f'sultan1{uuid.uuid4().hex[:16]}',
                    'balance': 1000000,
                    'created': datetime.now().isoformat()
                }
            elif method == 'token_transfer':
                result = {
                    'success': True,
                    'tx_hash': f'0x{uuid.uuid4().hex}',
                    'fee': 0,
                    'message': 'Transfer complete - NO GAS FEES!'
                }
            elif method == 'get_apy':
                result = {
                    'base_apy': '13.33%',
                    'mobile_validator_bonus': '40%',
                    'total_possible': '18.66%'
                }
            else:
                result = {'error': 'Method not found'}
            
            response = {
                'jsonrpc': '2.0',
                'result': result,
                'id': request_id
            }
            
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            self.wfile.write(json.dumps(response).encode())
            
        except Exception as e:
            self.send_error(500, str(e))

print('🚀 Sultan Chain Demo Server')
print('📍 Running on http://0.0.0.0:3030')
print('')
print('Features:')
print('  ✅ Zero Gas Fees')
print('  ✅ 13.33% APY Staking')
print('  ✅ Mobile Validators (40% bonus)')
print('  ✅ 10,000+ TPS')
print('')

# Bind to all interfaces for Codespaces
server = HTTPServer(('0.0.0.0', 3030), SultanHandler)
server.serve_forever()
PYTHON

DEMO_PID=$!
echo "✅ Server started with PID: $DEMO_PID"
sleep 3

# Test local access
echo ""
echo "🧪 Testing local access..."
if curl -s http://127.0.0.1:3030/health | grep -q "healthy"; then
    echo "✅ Local access working"
else
    echo "❌ Local access failed"
fi

# Make port public in Codespaces
echo ""
echo "📡 Configuring Codespaces port visibility..."
gh codespace ports visibility 3030:public -c $CODESPACE_NAME 2>/dev/null || true

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                    SULTAN CHAIN IS LIVE!                      ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "🌍 PUBLIC URLs:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📍 API Endpoint:"
echo "   https://${CODESPACE_NAME}-3030.app.github.dev/"
echo ""
echo "📍 Health Check:"
echo "   https://${CODESPACE_NAME}-3030.app.github.dev/health"
echo ""
echo "📍 Local Access:"
echo "   http://127.0.0.1:3030"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📝 TEST COMMANDS:"
echo ""
echo "# Test chain status:"
echo "curl -X POST https://${CODESPACE_NAME}-3030.app.github.dev/ \\"
echo "  -H 'Content-Type: application/json' \\"
echo "  -d '{\"jsonrpc\":\"2.0\",\"method\":\"chain_status\",\"id\":1}'"
echo ""
echo "# Create wallet:"
echo "curl -X POST https://${CODESPACE_NAME}-3030.app.github.dev/ \\"
echo "  -H 'Content-Type: application/json' \\"
echo "  -d '{\"jsonrpc\":\"2.0\",\"method\":\"wallet_create\",\"params\":[\"testuser\"],\"id\":1}'"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Server running with PID: $DEMO_PID"
echo "🛑 To stop: kill $DEMO_PID"
