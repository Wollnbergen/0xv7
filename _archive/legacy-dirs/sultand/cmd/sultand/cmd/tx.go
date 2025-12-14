package cmd

import (
	"fmt"
	"strconv"
	"time"
	
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/tx"
	authcmd "github.com/cosmos/cosmos-sdk/x/auth/client/cli"
	
	"github.com/spf13/cobra"
	
	ibccorecmd "github.com/cosmos/ibc-go/v8/modules/core/client/cli"
	ibctransfercmd "github.com/cosmos/ibc-go/v8/modules/apps/transfer/client/cli"
	
	sultantypes "github.com/wollnbergen/sultan-cosmos-module/x/sultan/types"
)

// TxCmd returns the transaction commands for this module
func TxCmd() *cobra.Command {
	txCmd := &cobra.Command{
		Use:                        "tx",
		Short:                      "Transaction subcommands",
		DisableFlagParsing:         true,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}
	
	// Add Sultan custom transaction commands
	txCmd.AddCommand(
		SendTxCmd(),
		CreateValidatorTxCmd(),
	)
	
	// Add auth module signing commands (production-grade)
	txCmd.AddCommand(
		authcmd.GetSignCommand(),
		authcmd.GetMultiSignCommand(),
		authcmd.GetValidateSignaturesCommand(),
		authcmd.GetBroadcastCommand(),
	)
	
	// Add IBC module transaction commands (production-grade)
	txCmd.AddCommand(
		ibccorecmd.GetTxCmd(),           // IBC core tx (create-client, update-client, etc.)
		ibctransfercmd.NewTransferTxCmd(), // IBC transfer tx (send cross-chain tokens)
	)
	
	return txCmd
}

// SendTxCmd returns a CLI command handler for creating a MsgSend transaction
func SendTxCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "send [from_address] [to_address] [amount]",
		Short: "Send tokens from one account to another (zero gas fees!)",
		Long: `Send tokens from one account to another on the Sultan blockchain.

Sultan blockchain has ZERO gas fees - all transactions are completely free!

Example:
  sultand tx send cosmos1alice... cosmos1bob... 1000 --from alice
  
This will send 1000 tokens from alice to bob with no transaction fees.`,
		Args: cobra.ExactArgs(3),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}
			
			fromAddr := args[0]
			toAddr := args[1]
			
			// Parse amount
			amount, err := strconv.ParseUint(args[2], 10, 64)
			if err != nil {
				return fmt.Errorf("invalid amount: %w", err)
			}
			
			// Get nonce (sequence number)
			// In production, this would query the account
			nonce := uint64(time.Now().Unix())
			
			// Create message
			msg := &sultantypes.MsgSend{
				From:   fromAddr,
				To:     toAddr,
				Amount: amount,
				Nonce:  nonce,
			}
			
			if err := msg.ValidateBasic(); err != nil {
				return err
			}
			
			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}
	
	flags.AddTxFlagsToCmd(cmd)
	
	return cmd
}

// CreateValidatorTxCmd returns a CLI command for creating a validator
func CreateValidatorTxCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "create-validator [validator-address] [stake]",
		Short: "Create a new validator",
		Long: `Register as a validator on the Sultan blockchain.

Validators participate in block production and consensus.
Stake determines voting power in the consensus algorithm.

Example:
  sultand tx create-validator cosmos1validator... 100000 --from validator
  
This will register a validator with 100,000 tokens staked.`,
		Args: cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}
			
			validatorAddr := args[0]
			
			// Parse stake
			stake, err := strconv.ParseUint(args[1], 10, 64)
			if err != nil {
				return fmt.Errorf("invalid stake amount: %w", err)
			}
			
			// Create message
			msg := &sultantypes.MsgCreateValidator{
				ValidatorAddress: validatorAddr,
				Stake:            stake,
			}
			
			if err := msg.ValidateBasic(); err != nil {
				return err
			}
			
			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}
	
	flags.AddTxFlagsToCmd(cmd)
	
	return cmd
}
