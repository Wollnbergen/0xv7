import CryptoJS from 'crypto-js';
import LRU from 'lru-cache';

export function hashPIN(pin: string): string {
  return CryptoJS.SHA256(pin).toString();
}

export function encryptData(data: string, pin: string): string {
  return CryptoJS.AES.encrypt(data, pin).toString();
}

export function decryptData(encrypted: string, pin: string): string {
  const bytes = CryptoJS.AES.decrypt(encrypted, pin);
  return bytes.toString(CryptoJS.enc.Utf8);
}

export function generateMnemonic(): string {
  return 'word1 word2 word3 ... word24';
}

// MEV-resistant rate limiter using LRU cache
const rateLimiter = new LRU<string, number[]>({ max: 1000, ttl: 60000 }); // 1min TTL, max 1k users

export function checkRateLimit(userId: string, action: string): boolean {
  const key = `${userId}:${action}`;
  const requests = rateLimiter.get(key) || [];
  const now = Date.now();
  const recentRequests = requests.filter(time => now - time < 60000);
  if (recentRequests.length >= 10) return false;
  recentRequests.push(now);
  rateLimiter.set(key, recentRequests);
  return true;
}