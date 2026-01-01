#!/usr/bin/env node
/**
 * End-to-end wallet test for Sultan L1
 * Tests Ed25519 signature verification on the live network
 */

import { ed25519 } from '@noble/ed25519';
import { sha256 } from '@noble/hashes/sha256';
import { bytesToHex, hexToBytes } from '@noble/hashes/utils';

// Test configuration
const NODE_URL = process.env.NODE_URL || 'http://206.189.224.142:8545';

// Generate a test keypair
const privateKey = ed25519.utils.randomPrivateKey();
const publicKey = await ed25519.getPublicKeyAsync(privateKey);
const publicKeyHex = bytesToHex(publicKey);

// Create a simple address from the public key (sultan1 prefix + first 40 hex chars)
const address = 'sultan1' + publicKeyHex.slice(0, 40);

console.log('=== Sultan L1 End-to-End Wallet Test ===');
console.log('Node URL:', NODE_URL);
console.log('Test Address:', address);
console.log('Public Key:', publicKeyHex);

// First, check the node status
console.log('\n1. Checking node status...');
const statusRes = await fetch(`${NODE_URL}/status`);
const status = await statusRes.json();
console.log('   Height:', status.height);
console.log('   Validators:', status.validator_count);

// Check balance (should be 0 for new address)
console.log('\n2. Checking balance...');
const balanceRes = await fetch(`${NODE_URL}/balance/${address}`);
const balance = await balanceRes.json();
console.log('   Balance:', balance.balance || 0, 'SLTN');
console.log('   Nonce:', balance.nonce || 0);

// Create a signed transaction
console.log('\n3. Creating signed transaction...');
const timestamp = Date.now();
const nonce = balance.nonce || 0;

const tx = {
  from: address,
  to: 'sultan1recipient123456789012345678901234567890',
  amount: 100,
  memo: '',
  nonce: nonce,
  timestamp: timestamp,
};

// The message format that the node expects (amount as quoted string)
const messageStr = JSON.stringify({
  from: tx.from,
  to: tx.to,
  amount: tx.amount,  // JS JSON.stringify will serialize as number
  memo: tx.memo,
  nonce: tx.nonce,
  timestamp: tx.timestamp,
});

console.log('   Message to sign:', messageStr);

// Hash and sign
const messageHash = sha256(new TextEncoder().encode(messageStr));
const signature = await ed25519.signAsync(messageHash, privateKey);
const signatureHex = bytesToHex(signature);

console.log('   Signature length:', signatureHex.length, 'hex chars');

// Build the request body (wallet format)
const txRequest = {
  tx: {
    from: tx.from,
    to: tx.to,
    amount: tx.amount,
    memo: tx.memo,
    nonce: tx.nonce,
    timestamp: tx.timestamp,
  },
  signature: signatureHex,
  public_key: publicKeyHex,
};

console.log('\n4. Submitting transaction...');
console.log('   Request:', JSON.stringify(txRequest, null, 2));

const txRes = await fetch(`${NODE_URL}/tx`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify(txRequest),
});

const txResult = await txRes.json();
console.log('\n5. Transaction result:', JSON.stringify(txResult, null, 2));

if (txResult.error) {
  console.log('\n❌ Transaction FAILED:', txResult.error);
  process.exit(1);
} else {
  console.log('\n✅ Transaction SUBMITTED! Hash:', txResult.hash);
  
  // Wait for it to be included in a block
  console.log('\n6. Waiting for confirmation (5 seconds)...');
  await new Promise(resolve => setTimeout(resolve, 5000));
  
  // Check balance again
  const balanceRes2 = await fetch(`${NODE_URL}/balance/${address}`);
  const balance2 = await balanceRes2.json();
  console.log('   Balance after tx:', balance2.balance || 0, 'SLTN');
  console.log('   Nonce after tx:', balance2.nonce || 0);
  
  console.log('\n=== Test Complete ===');
}
