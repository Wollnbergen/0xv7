# Sultan Chain - Zero Gas Quick Start

from sltn import SultanSDK

def main():
    # Connect - no API key needed!
    sltn = SultanSDK('https://rpc.sltn.io')
    
    # Check balance
    balance = sltn.get_balance('sultan1...')
    print(f'Balance: {balance} SLTN')
    
    # Send transaction - ZERO gas fees!
    tx = sltn.send_transaction(
        to='sultan1xyz...',
        amount=100
        # No gas parameter needed!
    )
    print(f'Transaction sent: {tx.hash}')
    print(f'Gas fee: $0.00')

if __name__ == '__main__':
    main()
