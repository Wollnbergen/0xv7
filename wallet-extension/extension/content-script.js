/**
 * Sultan Wallet - Content Script
 * 
 * This script runs in the context of web pages and acts as a bridge
 * between the inpage provider (window.sultan) and the background service worker.
 * 
 * Message flow:
 * dApp → inpage-provider.js → content-script.js → background.js → popup
 */

(function() {
  'use strict';

  const EXTENSION_ID = 'sultan-wallet';

  /**
   * Inject the inpage provider script into the page
   * This makes window.sultan available to dApps
   */
  function injectProvider() {
    try {
      const script = document.createElement('script');
      script.src = chrome.runtime.getURL('inpage-provider.js');
      script.type = 'text/javascript';
      script.onload = function() {
        this.remove();
      };
      
      // Inject as early as possible
      const container = document.head || document.documentElement;
      container.insertBefore(script, container.firstChild);
    } catch (error) {
      console.error('[Sultan Content] Failed to inject provider:', error);
    }
  }

  /**
   * Listen for messages from the inpage provider
   */
  window.addEventListener('message', async (event) => {
    // Only accept messages from same window
    if (event.source !== window) return;
    
    // Check message format
    if (!event.data || event.data.type !== 'SULTAN_PROVIDER_REQUEST') return;
    if (event.data.source !== EXTENSION_ID) return;

    const { id, method, params } = event.data;

    try {
      // Forward to background service worker
      const response = await chrome.runtime.sendMessage({
        type: 'SULTAN_RPC_REQUEST',
        id,
        method,
        params,
        origin: window.location.origin,
        href: window.location.href
      });

      // Send response back to inpage provider
      window.postMessage({
        type: 'SULTAN_PROVIDER_RESPONSE',
        id,
        result: response.result,
        error: response.error,
        source: EXTENSION_ID
      }, '*');
    } catch (error) {
      // Handle extension context invalidation
      window.postMessage({
        type: 'SULTAN_PROVIDER_RESPONSE',
        id,
        error: { message: error.message || 'Extension communication failed' },
        source: EXTENSION_ID
      }, '*');
    }
  });

  /**
   * Listen for events from background service worker
   */
  chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
    if (message.type === 'SULTAN_PROVIDER_EVENT') {
      // Forward event to inpage provider
      window.postMessage({
        type: 'SULTAN_PROVIDER_EVENT',
        eventName: message.eventName,
        payload: message.payload,
        source: EXTENSION_ID
      }, '*');
    }
    
    // Return true to indicate we may respond asynchronously
    return true;
  });

  /**
   * Notify background that content script is ready
   */
  function notifyReady() {
    try {
      chrome.runtime.sendMessage({
        type: 'SULTAN_CONTENT_READY',
        origin: window.location.origin,
        href: window.location.href
      }).catch(() => {
        // Ignore errors during page load
      });
    } catch (e) {
      // Extension context may not be ready yet
    }
  }

  // Inject provider immediately
  injectProvider();
  
  // Notify background when DOM is ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', notifyReady);
  } else {
    notifyReady();
  }

})();
