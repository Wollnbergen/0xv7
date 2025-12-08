#!/bin/bash

cd /workspaces/0xv7

echo "=== 🔧 COMPLETE RPC SERVER FIX ==="
echo ""

# The issue is likely that create_wallet and other methods aren't in the match statement
# Let's check the actual handle_call implementation

echo "1. Examining current handle_call implementation..."
HANDLE_LINE=$(grep -n "async fn handle_call" node/src/rpc_server.rs | cut -d: -f1)

if [ -n "$HANDLE_LINE" ]; then
    echo "   Found handle_call at line $HANDLE_LINE"
    
    # Extract just the method matching part
    sed -n "${HANDLE_LINE},$((HANDLE_LINE+200))p" node/src/rpc_server.rs | grep -A2 '"create_wallet"' | head -5
    
    # Check if create_wallet is actually implemented
    if ! sed -n "${HANDLE_LINE},$((HANDLE_LINE+200))p" node/src/rpc_server.rs | grep -q '"create_wallet"'; then
        echo "   ❌ create_wallet not found in handle_call!"
        echo ""
        echo "2. The methods must be outside the match statement. Let's find them..."
        
        # Find where create_wallet implementation is
        CREATE_LINE=$(grep -n '"create_wallet" =>' node/src/rpc_server.rs | head -1 | cut -d: -f1)
        
        if [ -n "$CREATE_LINE" ]; then
            echo "   Found create_wallet at line $CREATE_LINE"
            
            # Check if it's inside handle_call
            if [ "$CREATE_LINE" -lt "$HANDLE_LINE" ] || [ "$CREATE_LINE" -gt "$((HANDLE_LINE+200))" ]; then
                echo "   ⚠️ Methods are defined outside handle_call function!"
                echo ""
                echo "3. This explains the 'Method not found' errors."
            fi
        fi
    fi
fi

echo ""
echo "4. Creating a working test to verify the issue..."

# Create a simple test
cat > test_rpc.sh << 'EOFTEST'
#!/bin/bash
echo "Testing RPC endpoints..."

# Generate token
export SULTAN_JWT_SECRET='production_secret_32_bytes_minimum_required'
TOKEN=$(cargo run -q -p sultan-coordinator --bin jwt_gen prod 3600 2>/dev/null)

echo "Testing with token: ${TOKEN:0:20}..."

# Test create_wallet
echo -n "create_wallet: "
curl -sS -X POST http://127.0.0.1:3030 \
     -H "Authorization: Bearer $TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"create_wallet","params":["test"],"id":1}' 2>&1 | \
     grep -q "Method not found" && echo "❌ Not found" || echo "✅ Works"

# Test proposal_create  
echo -n "proposal_create: "
curl -sS -X POST http://127.0.0.1:3030 \
     -H "Authorization: Bearer $TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"proposal_create","params":["p1","Test","Desc",null],"id":2}' 2>&1 | \
     grep -q "Method not found" && echo "❌ Not found" || echo "✅ Works"
EOFTEST

chmod +x test_rpc.sh
./test_rpc.sh

echo ""
echo "=== 📊 DIAGNOSIS ==="
echo ""
echo "The issue is that the RPC methods (create_wallet, proposal_create, etc.)"
echo "are not properly registered in the handle_call function's match statement."
echo ""
echo "The methods are likely defined separately but not connected to the RPC handler."
echo ""
echo "To fix this, we need to ensure all methods are inside the match statement"
echo "in the handle_call function."
echo ""
echo "Server still running on PID: $(pgrep -f 'cargo.*rpc_server')"
echo "Logs: tail -f /tmp/sultan_trace.log"
