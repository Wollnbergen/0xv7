module sultan

go 1.21

require (
    cosmossdk.io/api v0.7.3
    cosmossdk.io/core v0.11.0
    cosmossdk.io/errors v1.0.1
    cosmossdk.io/log v1.3.1
    cosmossdk.io/math v1.3.0
    cosmossdk.io/store v1.0.2
    github.com/cosmos/cosmos-db v1.0.0
    github.com/cosmos/cosmos-sdk v0.50.5
    github.com/cosmos/gogoproto v1.4.11
    github.com/spf13/cobra v1.8.0
    google.golang.org/grpc v1.62.1
    google.golang.org/protobuf v1.33.0
)

replace (
    github.com/gogo/protobuf => github.com/regen-network/protobuf v1.3.3-alpha.regen.1
)
