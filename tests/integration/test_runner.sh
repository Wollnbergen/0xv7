#!/bin/bash

echo "Running integration tests..."

# Run tests for the Sovereign chain
echo "Running Sovereign chain tests..."
go test ./sovereign-chain/sovereign/app -v

# Run other integration tests
echo "Running end-to-end tests..."
node tests/integration/end_to_end_test.js

echo "All tests completed."