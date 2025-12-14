#!/bin/bash

# Install Ignite CLI (formerly Starport)
echo "Installing Ignite CLI for Cosmos SDK..."

# Check if already installed
if ! command -v ignite &> /dev/null; then
    curl -L https://get.ignite.com/cli | bash
    export PATH=$PATH:$HOME/.ignite/bin
else
    echo "Ignite CLI already installed"
fi

# Verify installation
ignite version 2>/dev/null || echo "Ignite CLI installation pending"
