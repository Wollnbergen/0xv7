// Sultan Chain - Zero Gas Quick Start

const { SultanSDK } = require('@sltn/sdk');

async function main() {
  // Connect - no API key needed!
  const sltn = new SultanSDK({
    network: 'mainnet',
    rpc: 'https://rpc.sltn.io'
  });

  // Check balance
  const balance = await sltn.getBalance('sultan1...');
  console.log(`Balance: ${balance} SLTN`);

  // Send transaction - ZERO gas fees!
  const tx = await sltn.sendTransaction({
    to: 'sultan1xyz...',
    amount: 100
    // Notice: no gas parameter needed!
  });
  console.log(`Transaction sent: ${tx.hash}`);
  console.log(`Gas fee: $0.00`);

  // Deploy a contract - also free!
  const contract = await sltn.deployContract({
    bytecode: '0x...',
    // No gas fees for deployment
  });
  console.log(`Contract deployed at: ${contract.address}`);
}

main().catch(console.error);
