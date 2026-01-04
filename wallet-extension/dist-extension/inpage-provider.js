/**
 * Sultan Wallet - Inpage Provider
 * 
 * This script is injected into web pages and exposes window.sultan
 * for dApps to interact with the Sultan Wallet extension.
 * 
 * Provider Interface (EIP-1193 inspired):
 * - connect(): Request wallet connection
 * - disconnect(): Disconnect wallet
 * - isConnected(): Check connection status
 * - getAddress(): Get connected address
 * - getPublicKey(): Get connected public key
 * - signMessage(message): Sign arbitrary message
 * - signTransaction(tx): Sign and optionally broadcast transaction
 * - on(event, handler): Subscribe to events
 * - off(event, handler): Unsubscribe from events
 */

(function() {
  'use strict';

  // Prevent double injection
  if (window.sultan) {
    console.warn('[Sultan] Provider already injected');
    return;
  }

  const EXTENSION_ID = 'sultan-wallet';
  
  // Event emitter
  class EventEmitter {
    constructor() {
      this._events = {};
    }

    on(event, handler) {
      if (!this._events[event]) {
        this._events[event] = [];
      }
      this._events[event].push(handler);
      return this;
    }

    off(event, handler) {
      if (!this._events[event]) return this;
      if (!handler) {
        delete this._events[event];
      } else {
        this._events[event] = this._events[event].filter(h => h !== handler);
      }
      return this;
    }

    emit(event, ...args) {
      if (!this._events[event]) return false;
      this._events[event].forEach(handler => {
        try {
          handler(...args);
        } catch (e) {
          console.error('[Sultan] Event handler error:', e);
        }
      });
      return true;
    }

    removeAllListeners() {
      this._events = {};
    }
  }

  // Request ID counter
  let requestId = 0;
  const pendingRequests = new Map();

  // Connection state
  let connectedAccount = null;
  let isConnectedState = false;

  /**
   * Send message to content script (which forwards to background)
   */
  function sendMessage(method, params = {}) {
    return new Promise((resolve, reject) => {
      const id = ++requestId;
      
      pendingRequests.set(id, { resolve, reject });

      window.postMessage({
        type: 'SULTAN_PROVIDER_REQUEST',
        id,
        method,
        params,
        source: EXTENSION_ID
      }, '*');

      // Timeout after 5 minutes (for user approval)
      setTimeout(() => {
        if (pendingRequests.has(id)) {
          pendingRequests.delete(id);
          reject(new Error('Request timeout'));
        }
      }, 300000);
    });
  }

  /**
   * Handle responses from content script
   */
  window.addEventListener('message', (event) => {
    if (event.source !== window) return;
    if (!event.data || event.data.type !== 'SULTAN_PROVIDER_RESPONSE') return;
    if (event.data.source !== EXTENSION_ID) return;

    const { id, result, error } = event.data;
    const pending = pendingRequests.get(id);
    
    if (pending) {
      pendingRequests.delete(id);
      if (error) {
        pending.reject(new Error(error.message || 'Unknown error'));
      } else {
        pending.resolve(result);
      }
    }
  });

  /**
   * Handle events from content script
   */
  window.addEventListener('message', (event) => {
    if (event.source !== window) return;
    if (!event.data || event.data.type !== 'SULTAN_PROVIDER_EVENT') return;
    if (event.data.source !== EXTENSION_ID) return;

    const { eventName, payload } = event.data;
    
    switch (eventName) {
      case 'connect':
        isConnectedState = true;
        connectedAccount = payload;
        provider.emit('connect', payload);
        break;
      case 'disconnect':
        isConnectedState = false;
        connectedAccount = null;
        provider.emit('disconnect');
        break;
      case 'accountChange':
        connectedAccount = payload;
        provider.emit('accountChange', payload);
        break;
      case 'networkChange':
        provider.emit('networkChange', payload);
        break;
    }
  });

  /**
   * Sultan Provider
   */
  class SultanProvider extends EventEmitter {
    constructor() {
      super();
      this.isSultan = true;
      this.version = '1.0.0';
    }

    /**
     * Request wallet connection
     * Opens extension popup for user approval
     * @returns {Promise<{address: string, publicKey: string}>}
     */
    async connect() {
      const result = await sendMessage('connect');
      isConnectedState = true;
      connectedAccount = result;
      this.emit('connect', result);
      return result;
    }

    /**
     * Disconnect wallet
     * @returns {Promise<void>}
     */
    async disconnect() {
      await sendMessage('disconnect');
      isConnectedState = false;
      connectedAccount = null;
      this.emit('disconnect');
    }

    /**
     * Check if wallet is connected
     * @returns {boolean}
     */
    isConnected() {
      return isConnectedState;
    }

    /**
     * Get connected address
     * @returns {Promise<string|null>}
     */
    async getAddress() {
      if (!isConnectedState) return null;
      return connectedAccount?.address || null;
    }

    /**
     * Get connected public key
     * @returns {Promise<string|null>}
     */
    async getPublicKey() {
      if (!isConnectedState) return null;
      return connectedAccount?.publicKey || null;
    }

    /**
     * Get account balance
     * @returns {Promise<{available: string, staked: string, rewards: string}>}
     */
    async getBalance() {
      if (!isConnectedState) {
        throw new Error('Wallet not connected');
      }
      return sendMessage('getBalance');
    }

    /**
     * Sign arbitrary message
     * Opens extension popup for user approval
     * @param {string|Uint8Array} message - Message to sign
     * @returns {Promise<{signature: string, publicKey: string}>}
     */
    async signMessage(message) {
      if (!isConnectedState) {
        throw new Error('Wallet not connected');
      }
      
      // Convert Uint8Array to hex if needed
      let messageHex;
      if (message instanceof Uint8Array) {
        messageHex = Array.from(message).map(b => b.toString(16).padStart(2, '0')).join('');
      } else if (typeof message === 'string') {
        // Encode string as UTF-8 then to hex
        const encoder = new TextEncoder();
        const bytes = encoder.encode(message);
        messageHex = Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
      } else {
        throw new Error('Message must be string or Uint8Array');
      }

      return sendMessage('signMessage', { message: messageHex });
    }

    /**
     * Sign a transaction
     * Opens extension popup for user approval
     * @param {object} transaction - Transaction object
     * @param {boolean} [broadcast=false] - Whether to broadcast after signing
     * @returns {Promise<{signature: string, publicKey: string, txHash?: string}>}
     */
    async signTransaction(transaction, broadcast = false) {
      if (!isConnectedState) {
        throw new Error('Wallet not connected');
      }

      // Validate transaction has required fields based on type
      if (!transaction.type) {
        throw new Error('Transaction must have a type');
      }

      return sendMessage('signTransaction', { transaction, broadcast });
    }

    /**
     * Sign and broadcast a transaction
     * Convenience wrapper for signTransaction with broadcast=true
     * @param {object} transaction - Transaction object
     * @returns {Promise<{signature: string, publicKey: string, txHash: string}>}
     */
    async sendTransaction(transaction) {
      return this.signTransaction(transaction, true);
    }

    /**
     * Request to add a custom token to the wallet
     * @param {object} token - Token details
     * @returns {Promise<boolean>}
     */
    async addToken(token) {
      if (!token.denom || !token.symbol) {
        throw new Error('Token must have denom and symbol');
      }
      return sendMessage('addToken', { token });
    }

    /**
     * Get network information
     * @returns {Promise<{chainId: string, name: string, rpcUrl: string}>}
     */
    async getNetwork() {
      return sendMessage('getNetwork');
    }

    /**
     * Check if extension is installed and available
     * @returns {boolean}
     */
    static isAvailable() {
      return typeof window.sultan !== 'undefined' && window.sultan.isSultan === true;
    }
  }

  // Create provider instance
  const provider = new SultanProvider();

  // Freeze provider to prevent tampering
  Object.freeze(provider);

  // Expose on window
  Object.defineProperty(window, 'sultan', {
    value: provider,
    writable: false,
    configurable: false,
    enumerable: true
  });

  // Announce availability
  window.dispatchEvent(new CustomEvent('sultan#initialized'));
  
  console.log('[Sultan] Wallet provider injected v' + provider.version);
})();
