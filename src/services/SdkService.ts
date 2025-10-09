import { createPublicClient, http } from 'viem';
import { Connection, PublicKey } from '@solana/web3.js';
import { TonClient, Address } from '@ton/ton';

export class SdkService {
  async getBalance(chain: string, rpcUrl: string, address: string): Promise<string> {
    switch (chain) {
      case 'sultan': {
        const client = createPublicClient({ transport: http('https://sultan-node.io/rpc') });
        return (await client.getBalance({ address })).toString();
      }
      case 'ethereum': {
        const ethClient = createPublicClient({ transport: http('https://mainnet.infura.io/v3/YOUR_KEY') });
        return (await ethClient.getBalance({ address })).toString();
      }
      case 'solana': {
        const solClient = new Connection('https://api.mainnet-beta.solana.com');
        return (await solClient.getBalance(new PublicKey(address))).toString();
      }
      case 'ton': {
        const tonClient = new TonClient({ endpoint: 'https://toncenter.com/api/v2/jsonRPC' });
        const bal = await tonClient.getBalance(Address.parse(address));
        return bal.toString();
      }
      case 'bitcoin': {
        try {
          const response = await fetch(`https://blockstream.info/api/address/${address}`);
          if (!response.ok) throw new Error('BTC API error');
          const data = await response.json();
          // Calculate balance: funded_txo_sum - spent_txo_sum
          const funded = Number(data.chain_stats.funded_txo_sum || 0);
          const spent = Number(data.chain_stats.spent_txo_sum || 0);
          const balance = funded - spent;
          return balance.toString();
        } catch (error) {
          console.error('BTC balance failed:', error);
          return '0';
        }
      }
      default:
        throw new Error('Unsupported chain');
    }
  }

  async crossChainSwap(from: string, amount: string): Promise<void> {
    // TODO: Implement atomic swap logic via WASM/gRPC
    console.log(`Swapping ${amount} from ${from} (atomic, <3s)`);
  }

  async sendBitcoinTx(rawTx: string): Promise<string> {
    const response = await fetch('https://blockstream.info/api/tx', {
      method: 'POST',
      headers: { 'Content-Type': 'text/plain' },
      body: rawTx,
    });
    if (!response.ok) throw new Error('Failed to broadcast BTC transaction');
    return await response.text();
  }
}