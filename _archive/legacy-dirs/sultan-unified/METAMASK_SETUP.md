# MetaMask Setup for Sultan Chain

Sultan Chain is fully compatible with MetaMask and all Ethereum wallets. **No custom wallet needed!**

## 🦊 Quick Setup

### 1. Add Sultan Network to MetaMask

Click the network dropdown in MetaMask and select "Add Network" (or "Add a network manually"):

```
Network Name:       Sultan Chain
RPC URL:            http://localhost:8545
Chain ID:           1397969742
Currency Symbol:    SLTN
Block Explorer:     (leave empty or add your explorer URL)
```

### 2. Network Details

- **Chain ID (Decimal):** `1397969742`
- **Chain ID (Hex):** `0x534c544e` (ASCII for "SLTN")
- **Native Token:** SLTN
- **Block Time:** 5 seconds
- **Gas Price:** Always 0 (ZERO FEES!)

### 3. Import Existing Wallet

If you have a Sultan wallet address (starting with `sultan1...`), you can:

**Option A:** Use your existing Ethereum private key in MetaMask - it will work on Sultan
**Option B:** Create a new account in MetaMask specifically for Sultan

## ✨ Benefits vs Ethereum

| Feature | Ethereum | Sultan |
|---------|----------|--------|
| Gas Fees | $5-$50+ | **FREE (0 SLTN)** |
| Block Time | ~12 seconds | **5 seconds** |
| Failed TX Cost | Still pay gas | **FREE** |
| Wallet Support | MetaMask, etc. | **Same wallets!** |

## 🔗 Cross-Chain Features

### IBC (Inter-Blockchain Communication)
Sultan is part of the **Cosmos ecosystem** with native IBC support:
- **100+ Compatible Chains** - Transfer SLTN to/from any IBC-enabled chain
- **Major Chains**: Osmosis (DEX), Celestia (Data Availability), dYdX (Derivatives), Injective (DeFi), Cosmos Hub, Akash (Cloud), Juno, Secret Network, Kujira, Stride (Liquid Staking)
- **Zero Fees on Both Sides** - Sultan charges 0 fees, receive side depends on destination chain
- **Trustless** - No centralized bridge operators, secured by light clients

### Custom Bridges (Non-Cosmos)
For blockchains outside the Cosmos ecosystem:
- **Ethereum** - Transfer ETH/ERC20 ↔ SLTN (via JSON-RPC compatibility)
- **Solana** - SPL token bridges (gRPC service)
- **TON** - Jetton bridges (gRPC service)
- **Bitcoin** - BTC atomic swaps (HTLC-based)

## 📱 Mobile Wallet Support

Works with:
- ✅ MetaMask Mobile
- ✅ Rainbow Wallet
- ✅ Trust Wallet
- ✅ Coinbase Wallet
- ✅ Any WalletConnect-compatible wallet

## 🛠 Developer Integration

### Web3.js Example

```javascript
// Detect MetaMask
if (typeof window.ethereum !== 'undefined') {
  console.log('MetaMask is installed!');
}

// Request account access
await window.ethereum.request({ method: 'eth_requestAccounts' });

// Create Web3 instance
const web3 = new Web3(window.ethereum);

// Send transaction with ZERO fees
const tx = await web3.eth.sendTransaction({
  from: accounts[0],
  to: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
  value: web3.utils.toWei('1', 'ether'),
  gas: 0,        // Zero gas
  gasPrice: 0    // Zero fees!
});

// For IBC transfers to Cosmos chains, use Sultan RPC:
// const ibcTx = await fetch('http://localhost:8545', {
//   method: 'POST',
//   body: JSON.stringify({
//     method: 'sultan_ibcTransfer',
//     params: [accounts[0], 'osmo1...', '100', 'transfer/channel-0']
//   })
// });
```

### ethers.js Example

```javascript
// Connect to MetaMask
const provider = new ethers.providers.Web3Provider(window.ethereum);
await provider.send("eth_requestAccounts", []);
const signer = provider.getSigner();

// Send transaction
const tx = await signer.sendTransaction({
  to: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
  value: ethers.utils.parseEther("1.0"),
  gasPrice: 0,  // Zero fees!
  gasLimit: 0
});

await tx.wait(); // 5 second confirmation!
```

### Hardhat Configuration

```javascript
module.exports = {
  networks: {
    sultan: {
      url: "http://localhost:8545",
      chainId: 1397969742,
      accounts: ["0xYOUR_PRIVATE_KEY"],
      gasPrice: 0,  // Zero fees!
    }
  }
};
```

## 🎨 DApp Example

```html
<!DOCTYPE html>
<html>
<head>
  <title>Sultan DApp</title>
  <script src="https://cdn.jsdelivr.net/npm/web3@latest/dist/web3.min.js"></script>
</head>
<body>
  <button id="connect">Connect MetaMask</button>
  <button id="send">Send 1 SLTN (FREE!)</button>
  
  <script>
    let web3;
    let account;
    
    document.getElementById('connect').onclick = async () => {
      const accounts = await window.ethereum.request({ 
        method: 'eth_requestAccounts' 
      });
      account = accounts[0];
      web3 = new Web3(window.ethereum);
      alert('Connected: ' + account);
    };
    
    document.getElementById('send').onclick = async () => {
      const tx = await web3.eth.sendTransaction({
        from: account,
        to: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
        value: web3.utils.toWei('1', 'ether'),
        gasPrice: 0  // ZERO FEES!
      });
      alert('Transaction sent! Hash: ' + tx.transactionHash);
    };
  </script>
</body>
</html>
```

## 🔐 Security Notes

- Sultan uses standard Ethereum cryptography (secp256k1)
- Private keys work across both chains
- **Never share your private key or seed phrase**
- Always verify the RPC URL before adding to MetaMask
- Test with small amounts first

## 📊 Network Status

Check your connection:

```javascript
const web3 = new Web3('http://localhost:8545');
const chainId = await web3.eth.getChainId();
console.log(chainId); // Should be 1397969742

const gasPrice = await web3.eth.getGasPrice();
console.log(gasPrice); // Should be "0" (zero fees!)
```

## 🆘 Troubleshooting

**"Wrong network" error?**
- Verify Chain ID is `1397969742`
- Check RPC URL is correct

**Transaction failing?**
- Ensure you have SLTN balance
- Set `gasPrice: 0` explicitly
- Wait 5 seconds for block confirmation

**MetaMask not detecting Sultan?**
- Manually add network using settings above
- Refresh the page
- Try disconnecting and reconnecting

## 🌟 Why This Works

Sultan implements the **Ethereum JSON-RPC specification**, making it compatible with the entire Ethereum ecosystem. The only differences:
1. ✅ Gas price is always 0
2. ✅ Faster block times (5s vs 12s)
3. ✅ Native bridges to other chains

Everything else works exactly like Ethereum!

## 📚 Additional Resources

- [SDK Documentation](./SDK_RPC_DOCS.md)
- [RPC API Reference](./SDK_RPC_DOCS.md#-json-rpc-api)
- [Example Projects](./examples/)

---

**No custom wallet development needed - launch your DApp with MetaMask today!**
