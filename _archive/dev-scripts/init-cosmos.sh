#!/bin/sh
set -e

HOME_DIR=/root/.wasmd

# Initialize if not already done
if [ ! -f "$HOME_DIR/config/genesis.json" ]; then
  wasmd init sultan-node --chain-id sultan-1
  wasmd keys add validator --keyring-backend test
  wasmd genesis add-genesis-account $(wasmd keys show validator -a --keyring-backend test) 1000000000usltn,1000000000stake
  wasmd genesis gentx validator 100000000stake --keyring-backend test --chain-id sultan-1
  wasmd genesis collect-gentxs
  wasmd genesis validate || exit 1

    # Bind RPC and P2P to all interfaces and enable API
    sed -i 's#^laddr = ".*26657"#laddr = "tcp://0.0.0.0:26657"#' "$HOME_DIR/config/config.toml" || true
    sed -i 's#^laddr = ".*26656"#laddr = "tcp://0.0.0.0:26656"#' "$HOME_DIR/config/config.toml" || true
    sed -i 's#^cors_allowed_origins = \[\]#cors_allowed_origins = ["*"]#' "$HOME_DIR/config/config.toml" || true

    # app.toml: enable REST API and gRPC
    if [ -f "$HOME_DIR/config/app.toml" ]; then
      sed -i 's/^enable = false/enable = true/' "$HOME_DIR/config/app.toml" || true
      sed -i 's#^address = ".*1317"#address = "tcp://0.0.0.0:1317"#' "$HOME_DIR/config/app.toml" || true
      sed -i 's#^address = ".*:9090"#address = "0.0.0.0:9090"#' "$HOME_DIR/config/app.toml" || true
      # minimum-gas-prices
      grep -q '^minimum-gas-prices' "$HOME_DIR/config/app.toml" && \
        sed -i 's/^minimum-gas-prices.*/minimum-gas-prices = "0usltn"/' "$HOME_DIR/config/app.toml" || \
        echo 'minimum-gas-prices = "0usltn"' >> "$HOME_DIR/config/app.toml"
    fi
    # Set client defaults for convenience
    CLIENT_TOML="$HOME_DIR/config/client.toml"
    if [ -f "$CLIENT_TOML" ]; then
      sed -i 's/^chain-id = .*/chain-id = "sultan-1"/' "$CLIENT_TOML" || true
      sed -i 's#^node = ".*"#node = "tcp://localhost:26657"#' "$CLIENT_TOML" || true
      sed -i 's/^broadcast-mode = .*/broadcast-mode = "sync"/' "$CLIENT_TOML" || true
      sed -i 's/^keyring-backend = .*/keyring-backend = "test"/' "$CLIENT_TOML" || true
    fi
fi

# Start the node
exec wasmd start --minimum-gas-prices 0usltn
