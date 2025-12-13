package cmd

import (
	"context"
	"fmt"
	
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	sdk "github.com/cosmos/cosmos-sdk/types"
	
	"github.com/spf13/cobra"
	
	ibccmd "github.com/cosmos/ibc-go/v8/modules/core/client/cli"
	ibctransfercmd "github.com/cosmos/ibc-go/v8/modules/apps/transfer/client/cli"
	
	sultantypes "github.com/wollnbergen/sultan-cosmos-module/x/sultan/types"
)

// QueryCmd returns the query commands for this module
func QueryCmd() *cobra.Command {
	queryCmd := &cobra.Command{
		Use:                        "query",
		Aliases:                    []string{"q"},
		Short:                      "Querying subcommands",
		DisableFlagParsing:         true,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}
	
	// Add Sultan custom commands
	queryCmd.AddCommand(
		BalanceQueryCmd(),
		BlockchainInfoQueryCmd(),
	)
	
	// Add IBC module query commands (production-grade)
	queryCmd.AddCommand(
		ibccmd.GetQueryCmd(),            // IBC core queries (clients, connections, channels)
		ibctransfercmd.GetQueryCmd(),    // IBC transfer queries (denoms, escrow)
	)
	
	return queryCmd
}

// BalanceQueryCmd returns the command to query an account balance
func BalanceQueryCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "balance [address]",
		Short: "Query account balance",
		Long: `Query the balance of an account on the Sultan blockchain.

Example:
  sultand query balance cosmos1alice...
  
This will return the current balance of the specified account.`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			
			address := args[0]
			
			// Validate address
			if _, err := sdk.AccAddressFromBech32(address); err != nil {
				return fmt.Errorf("invalid address: %w", err)
			}
			
			// Create query client
			queryClient := sultantypes.NewQueryClient(clientCtx)
			
			// Query balance
			req := &sultantypes.QueryBalanceRequest{
				Address: address,
			}
			
			res, err := queryClient.Balance(context.Background(), req)
			if err != nil {
				return err
			}
			
			return clientCtx.PrintProto(res)
		},
	}
	
	flags.AddQueryFlagsToCmd(cmd)
	
	return cmd
}

// BlockchainInfoQueryCmd returns the command to query blockchain info
func BlockchainInfoQueryCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "info",
		Short: "Query blockchain information",
		Long: `Query general information about the Sultan blockchain.

Returns:
- Chain ID
- Current block height
- Additional metadata

Example:
  sultand query info`,
		Args: cobra.NoArgs,
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			
			// Create query client
			queryClient := sultantypes.NewQueryClient(clientCtx)
			
			// Query blockchain info
			req := &sultantypes.QueryBlockchainInfoRequest{}
			
			res, err := queryClient.BlockchainInfo(context.Background(), req)
			if err != nil {
				return err
			}
			
			return clientCtx.PrintProto(res)
		},
	}
	
	flags.AddQueryFlagsToCmd(cmd)
	
	return cmd
}
