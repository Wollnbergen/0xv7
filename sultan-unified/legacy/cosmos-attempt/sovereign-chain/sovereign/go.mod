module sovereign

go 1.21

require (
	cosmossdk.io/api v0.7.4
	cosmossdk.io/core v0.11.1
	cosmossdk.io/depinject v1.0.0
	cosmossdk.io/log v1.3.1
	cosmossdk.io/math v1.3.0
	github.com/cometbft/cometbft v0.38.6
	github.com/cosmos/cosmos-sdk v0.50.6
	github.com/cosmos/ibc-go/v8 v8.1.0
	google.golang.org/grpc v1.64.1
	google.golang.org/protobuf v1.34.2
)

require (
	github.com/davecgh/go-spew v1.1.2-0.20180830191138-d8f796af33cc // indirect
	github.com/pmezard/go-difflib v1.0.1-0.20181226105442-5d4384ee4fb2 // indirect
	github.com/stretchr/testify v1.11.1 // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)

// For quantum resistance (when available)
// replace github.com/pq-crystals/dilithium => ./lib/dilithium

replace github.com/cosmos/cosmos-sdk/x/epochs => cosmossdk.io/x/epochs v0.2.0-rc.1
