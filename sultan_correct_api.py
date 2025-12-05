#!/usr/bin/env python3
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
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        self.end_headers()
        
    def do_GET(self):
        if self.path == '/':
            self.send_response(200)
            self.send_header('Content-Type', 'text/html')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            html = '''
<!DOCTYPE html>
<html>
<head>
    <title>Sultan Chain API - ACTUAL Economics</title>
    <meta charset="UTF-8">
    <style>
        body { 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; 
            padding: 40px; 
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            line-height: 1.6;
        }
        .container { max-width: 900px; margin: 0 auto; }
        h1 { 
            font-size: 2.5em;
            margin-bottom: 10px;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
        }
        .subtitle {
            font-size: 1.2em;
            margin-bottom: 30px;
            opacity: 0.9;
        }
        .economics-box {
            background: rgba(255,255,255,0.1);
            backdrop-filter: blur(10px);
            padding: 25px;
            border-radius: 15px;
            margin: 20px 0;
            border: 1px solid rgba(255,255,255,0.2);
        }
        .highlight { 
            color: #10b981; 
            font-weight: bold;
            font-size: 1.1em;
        }
        .method {
            background: rgba(0,0,0,0.3);
            padding: 15px;
            margin: 15px 0;
            border-radius: 10px;
            border-left: 4px solid #10b981;
        }
        .method h3 {
            margin: 0 0 10px 0;
            color: #10b981;
        }
        code {
            background: rgba(0,0,0,0.5);
            padding: 10px;
            border-radius: 5px;
            display: block;
            margin: 10px 0;
            font-family: 'Courier New', monospace;
            overflow-x: auto;
        }
        .warning {
            background: rgba(255,193,7,0.2);
            border: 1px solid #ffc107;
            padding: 15px;
            border-radius: 10px;
            margin: 20px 0;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 Sultan Chain API</h1>
        <div class="subtitle">Zero Gas Fees • Real APY from Codebase • Mobile Validators</div>
        
        <div class="economics-box">
            <h2>💰 ACTUAL Economics (from Sultan Chain codebase)</h2>
            <p>✅ <span class="highlight">8% Annual Inflation</span> - Subsidizes ALL gas fees (blockchain.rs)</p>
            <p>📈 <span class="highlight">12% APY for Validators</span> (rewards.rs: 0.12)</p>
            <p>💵 <span class="highlight">8% APY for Delegators</span> (rewards.rs: 0.08)</p>
            <p>📱 <span class="highlight">30% Mobile Validator Target</span> (architecture docs)</p>
            <p>⛽ <span class="highlight">ZERO Gas Fees Forever</span> (subsidized by 8% inflation)</p>
        </div>

        <div class="warning">
            ⚠️ <strong>Note:</strong> These are the ACTUAL values from the Sultan Chain source code (not the 26.67% example)
        </div>
        
        <div class="method">
            <h3>Chain Status</h3>
            <code>POST /<br>{"jsonrpc":"2.0","method":"chain_status","id":1}</code>
        </div>
        
        <div class="method">
            <h3>Get Economics</h3>
            <code>POST /<br>{"jsonrpc":"2.0","method":"get_economics","id":1}</code>
        </div>
        
        <div class="method">
            <h3>Create Wallet</h3>
            <code>POST /<br>{"jsonrpc":"2.0","method":"wallet_create","params":["username"],"id":1}</code>
        </div>
        
        <div class="method">
            <h3>Transfer (Zero Fees!)</h3>
            <code>POST /<br>{"jsonrpc":"2.0","method":"token_transfer","params":["from","to",100],"id":1}</code>
        </div>
    </div>
</body>
</html>
            '''
            self.wfile.write(html.encode())
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
            
            # Handle methods with ACTUAL values from codebase
            if method == 'chain_status':
                result = {
                    'name': 'Sultan Chain',
                    'height': 123456,
                    'validators': 100,
                    'mobile_validators': 30,  # 30% target
                    'tps': 10000,
                    'gas_fees': 0,
                    'inflation_rate': '8%',  # ACTUAL from blockchain.rs
                    'validator_apy': '12%',  # ACTUAL from rewards.rs
                    'delegator_apy': '8%',   # ACTUAL from rewards.rs
                    'ibc_enabled': True
                }
            elif method == 'get_economics':
                result = {
                    'inflation_rate': '8% annually',
                    'validator_apy': '12%',
                    'delegator_apy': '8%',
                    'mobile_validator_target': '30%',
                    'user_gas_fees': 0,
                    'fee_subsidy_source': '8% inflation pool',
                    'source': 'Actual values from Sultan Chain codebase'
                }
            elif method == 'wallet_create':
                username = params[0] if params else 'user'
                result = {
                    'address': f'sultan1{uuid.uuid4().hex[:16]}',
                    'username': username,
                    'balance': 1000000,
                    'gas_fees': 0,
                    'apy_eligible': '8% (12% if validator)',
                    'created': datetime.now().isoformat()
                }
            elif method == 'token_transfer':
                from_addr = params[0] if len(params) > 0 else 'sender'
                to_addr = params[1] if len(params) > 1 else 'receiver'
                amount = params[2] if len(params) > 2 else 100
                result = {
                    'success': True,
                    'from': from_addr,
                    'to': to_addr,
                    'amount': amount,
                    'tx_hash': f'0x{uuid.uuid4().hex}',
                    'fee': 0,
                    'fee_note': 'Gas subsidized by 8% inflation',
                    'message': 'Transfer complete - ZERO FEES!'
                }
            elif method == 'get_apy':
                # This is the OLD method - update it to show correct values
                result = {
                    'validator_apy': '12%',  # ACTUAL from rewards.rs
                    'delegator_apy': '8%',   # ACTUAL from rewards.rs
                    'mobile_validator_target': '30%',
                    'note': 'Values updated from codebase (rewards.rs)'
                }
            else:
                result = {
                    'error': 'Method not found',
                    'available_methods': [
                        'chain_status',
                        'get_economics', 
                        'wallet_create',
                        'token_transfer',
                        'get_apy'
                    ]
                }
            
            response = {
                'jsonrpc': '2.0',
                'result': result,
                'id': request_id
            }
            
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            self.wfile.write(json.dumps(response, indent=2).encode())
            
        except Exception as e:
            self.send_error(500, str(e))

print('🚀 Sultan Chain API - Running with CORRECT Economics')
print('📍 Server: http://0.0.0.0:3030')
print('')
print('✅ ACTUAL VALUES from codebase:')
print('  • 8% inflation (blockchain.rs)')
print('  • 12% validator APY (rewards.rs)')
print('  • 8% delegator APY (rewards.rs)')
print('  • Zero gas fees (subsidized)')
print('')

server = HTTPServer(('0.0.0.0', 3030), SultanHandler)
server.serve_forever()
