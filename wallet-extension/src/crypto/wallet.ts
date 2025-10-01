// Sultan Enterprise Crypto Module - Military Grade
import { generateMnemonic, mnemonicToSeedSync, validateMnemonic } from '@scure/bip39';
import { wordlist } from '@scure/bip39/wordlists/english';
import { sha256 } from '@noble/hashes/sha256';
import { randomBytes } from '@noble/hashes/utils';
import * as ed25519 from '@noble/ed25519';
import { HDKey } from 'viem/accounts';

export class SultanCrypto {
  private static readonly SULTAN_PATH = "m/44'/1984'/0'/0"; // 1984 = Sultan coin type
  
  static generateMnemonic(strength: 128 | 256 = 256): string {
    // 256 bits = 24 words for maximum security
    return generateMnemonic(wordlist, strength);
  }
  
  static validateMnemonic(mnemonic: string): boolean {
    return validateMnemonic(mnemonic, wordlist);
  }
  
  static async deriveAccount(mnemonic: string, index: number): Promise<{
    address: string;
    publicKey: string;
    privateKey: string;
    path: string;
  }> {
    const seed = mnemonicToSeedSync(mnemonic);
    const hdKey = HDKey.fromMasterSeed(seed);
    const path = `${this.SULTAN_PATH}/${index}`;
    const account = hdKey.derive(path);
    
    if (!account.privateKey) {
      throw new Error('Failed to derive private key');
    }
    
    // Use Ed25519 for Sultan (like Solana)
    const privateKey = account.privateKey;
    const publicKey = await ed25519.getPublicKeyAsync(privateKey);
    const address = this.publicKeyToAddress(publicKey);
    
    return {
      address,
      publicKey: Buffer.from(publicKey).toString('hex'),
      privateKey: Buffer.from(privateKey).toString('hex'),
      path
    };
  }
  
  static publicKeyToAddress(publicKey: Uint8Array): string {
    // Sultan address = "S" + base58(sha256(publicKey)[:20])
    const hash = sha256(publicKey);
    const addressBytes = hash.slice(0, 20);
    return 'S' + this.base58Encode(addressBytes);
  }
  
  static async signTransaction(privateKey: string, transaction: any): Promise<string> {
    const msgHash = this.hashTransaction(transaction);
    const privKey = Buffer.from(privateKey, 'hex');
    const signature = await ed25519.signAsync(msgHash, privKey);
    return Buffer.from(signature).toString('hex');
  }
  
  static async signMessage(privateKey: string, message: string): Promise<string> {
    const msgBytes = new TextEncoder().encode(message);
    const privKey = Buffer.from(privateKey, 'hex');
    const signature = await ed25519.signAsync(msgBytes, privKey);
    return Buffer.from(signature).toString('hex');
  }
  
  private static hashTransaction(tx: any): Uint8Array {
    const txString = JSON.stringify({
      from: tx.from,
      to: tx.to,
      value: tx.value.toString(),
      data: tx.data || '',
      nonce: tx.nonce
    });
    return sha256(new TextEncoder().encode(txString));
  }
  
  private static base58Encode(bytes: Uint8Array): string {
    const ALPHABET = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
    let encoded = '';
    let num = BigInt('0x' + Buffer.from(bytes).toString('hex'));
    
    while (num > 0n) {
      const remainder = num % 58n;
      num = num / 58n;
      encoded = ALPHABET[Number(remainder)] + encoded;
    }
    
    // Handle leading zeros
    for (const byte of bytes) {
      if (byte === 0) encoded = '1' + encoded;
      else break;
    }
    
    return encoded;
  }
  
  static async encryptSeed(seed: string, password: string): Promise<string> {
    // Use Web Crypto API for encryption
    const encoder = new TextEncoder();
    const salt = randomBytes(16);
    
    // Derive key from password using PBKDF2
    const keyMaterial = await crypto.subtle.importKey(
      'raw',
      encoder.encode(password),
      'PBKDF2',
      false,
      ['deriveBits', 'deriveKey']
    );
    
    const key = await crypto.subtle.deriveKey(
      {
        name: 'PBKDF2',
        salt,
        iterations: 100000,
        hash: 'SHA-256'
      },
      keyMaterial,
      { name: 'AES-GCM', length: 256 },
      false,
      ['encrypt', 'decrypt']
    );
    
    const iv = randomBytes(12);
    const encrypted = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      key,
      encoder.encode(seed)
    );
    
    // Combine salt + iv + encrypted
    const combined = new Uint8Array(salt.length + iv.length + encrypted.byteLength);
    combined.set(salt, 0);
    combined.set(iv, salt.length);
    combined.set(new Uint8Array(encrypted), salt.length + iv.length);
    
    return Buffer.from(combined).toString('base64');
  }
  
  static async decryptSeed(encryptedSeed: string, password: string): Promise<string> {
    const combined = Buffer.from(encryptedSeed, 'base64');
    const salt = combined.slice(0, 16);
    const iv = combined.slice(16, 28);
    const encrypted = combined.slice(28);
    
    const encoder = new TextEncoder();
    const decoder = new TextDecoder();
    
    const keyMaterial = await crypto.subtle.importKey(
      'raw',
      encoder.encode(password),
      'PBKDF2',
      false,
      ['deriveBits', 'deriveKey']
    );
    
    const key = await crypto.subtle.deriveKey(
      {
        name: 'PBKDF2',
        salt,
        iterations: 100000,
        hash: 'SHA-256'
      },
      keyMaterial,
      { name: 'AES-GCM', length: 256 },
      false,
      ['encrypt', 'decrypt']
    );
    
    const decrypted = await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv },
      key,
      encrypted
    );
    
    return decoder.decode(decrypted);
  }
}
