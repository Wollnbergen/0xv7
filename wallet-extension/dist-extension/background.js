/**
 * Sultan Wallet - Background Service Worker
 * 
 * Handles:
 * - Message routing between content scripts and popup
 * - Connection state management
 * - Pending approval queue
 * - RPC communication with Sultan node
 * - Dynamic icon switching based on system color scheme
 */

const SULTAN_RPC_URL = 'http://206.189.224.142:8545';

// =====================================================
// Dynamic Icon Switching (Light/Dark Mode)
// =====================================================

/**
 * Update extension icon based on color scheme
 * @param {boolean} isDark - Whether system is in dark mode
 */
function updateIcon(isDark) {
  const suffix = isDark ? '-dark' : '-light';
  chrome.action.setIcon({
    path: {
      16: `icons/icon-16${suffix}.png`,
      32: `icons/icon-32${suffix}.png`,
      48: `icons/icon-48${suffix}.png`,
      128: `icons/icon-128${suffix}.png`
    }
  }).catch(err => {
    // Fallback to default icons if themed ones don't exist
    console.log('[Sultan BG] Themed icons not found, using defaults');
  });
}

/**
 * Initialize icon based on system color scheme
 * Note: Service workers don't have access to window.matchMedia,
 * so we use a content script or offscreen document to detect theme
 */
function initializeIconSwitching() {
  // Check stored preference first
  chrome.storage.local.get(['systemColorScheme'], (result) => {
    const isDark = result.systemColorScheme === 'dark';
    updateIcon(isDark);
  });
}

// Listen for color scheme changes from content scripts
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === 'SYSTEM_COLOR_SCHEME_CHANGED') {
    const isDark = message.colorScheme === 'dark';
    chrome.storage.local.set({ systemColorScheme: message.colorScheme });
    updateIcon(isDark);
    return;
  }
});

// Initialize icon on service worker start
initializeIconSwitching();

// Connection state per origin
const connections = new Map();

// Pending approval requests (waiting for user action in popup)
const pendingApprovals = new Map();
let approvalId = 0;

/**
 * Get connection for origin
 */
function getConnection(origin) {
  return connections.get(origin) || null;
}

/**
 * Set connection for origin
 */
function setConnection(origin, account) {
  connections.set(origin, {
    address: account.address,
    publicKey: account.publicKey,
    connectedAt: Date.now()
  });
  
  // Persist connections
  chrome.storage.local.set({ 
    connections: Object.fromEntries(connections) 
  });
}

/**
 * Remove connection for origin
 */
function removeConnection(origin) {
  connections.delete(origin);
  chrome.storage.local.set({ 
    connections: Object.fromEntries(connections) 
  });
}

/**
 * Load persisted connections on startup
 */
async function loadConnections() {
  try {
    const data = await chrome.storage.local.get(['connections']);
    if (data.connections) {
      Object.entries(data.connections).forEach(([origin, conn]) => {
        connections.set(origin, conn);
      });
    }
  } catch (e) {
    console.error('[Sultan BG] Failed to load connections:', e);
  }
}

/**
 * Create approval request and wait for user action
 */
function createApprovalRequest(type, origin, data) {
  return new Promise((resolve, reject) => {
    const id = ++approvalId;
    
    pendingApprovals.set(id, {
      id,
      type,
      origin,
      data,
      resolve,
      reject,
      createdAt: Date.now()
    });

    // Open popup for approval
    chrome.action.openPopup().catch(() => {
      // Popup may already be open or blocked
      // User can click the extension icon
    });

    // Timeout after 5 minutes
    setTimeout(() => {
      if (pendingApprovals.has(id)) {
        pendingApprovals.delete(id);
        reject(new Error('Request expired'));
      }
    }, 300000);
  });
}

/**
 * Get pending approvals for popup
 */
function getPendingApprovals() {
  return Array.from(pendingApprovals.values()).map(req => ({
    id: req.id,
    type: req.type,
    origin: req.origin,
    data: req.data,
    createdAt: req.createdAt
  }));
}

/**
 * Resolve a pending approval
 */
function resolveApproval(id, approved, result = null) {
  const request = pendingApprovals.get(id);
  if (!request) {
    return { error: 'Approval request not found' };
  }

  pendingApprovals.delete(id);

  if (approved) {
    request.resolve(result);
  } else {
    request.reject(new Error('User rejected the request'));
  }

  return { success: true };
}

/**
 * Handle RPC request from content script
 */
async function handleRpcRequest(method, params, origin) {
  switch (method) {
    case 'connect': {
      // Check if already connected
      const existing = getConnection(origin);
      if (existing) {
        return { result: existing };
      }

      // Request user approval
      try {
        const account = await createApprovalRequest('connect', origin, {});
        setConnection(origin, account);
        return { result: { address: account.address, publicKey: account.publicKey } };
      } catch (error) {
        return { error: { message: error.message } };
      }
    }

    case 'disconnect': {
      removeConnection(origin);
      // Notify content script
      notifyOrigin(origin, 'disconnect', {});
      return { result: true };
    }

    case 'getBalance': {
      const conn = getConnection(origin);
      if (!conn) {
        return { error: { message: 'Not connected' } };
      }

      try {
        const response = await fetch(`${SULTAN_RPC_URL}/balance/${conn.address}`);
        const data = await response.json();
        return { result: data };
      } catch (error) {
        return { error: { message: 'Failed to fetch balance' } };
      }
    }

    case 'signMessage': {
      const conn = getConnection(origin);
      if (!conn) {
        return { error: { message: 'Not connected' } };
      }

      try {
        const result = await createApprovalRequest('signMessage', origin, {
          message: params.message,
          address: conn.address
        });
        return { result };
      } catch (error) {
        return { error: { message: error.message } };
      }
    }

    case 'signTransaction': {
      const conn = getConnection(origin);
      if (!conn) {
        return { error: { message: 'Not connected' } };
      }

      try {
        const result = await createApprovalRequest('signTransaction', origin, {
          transaction: params.transaction,
          broadcast: params.broadcast || false,
          address: conn.address
        });
        return { result };
      } catch (error) {
        return { error: { message: error.message } };
      }
    }

    case 'addToken': {
      const conn = getConnection(origin);
      if (!conn) {
        return { error: { message: 'Not connected' } };
      }

      try {
        const result = await createApprovalRequest('addToken', origin, {
          token: params.token,
          address: conn.address
        });
        return { result };
      } catch (error) {
        return { error: { message: error.message } };
      }
    }

    case 'getNetwork': {
      return {
        result: {
          chainId: 'sultan-1',
          name: 'Sultan Mainnet',
          rpcUrl: SULTAN_RPC_URL
        }
      };
    }

    default:
      return { error: { message: `Unknown method: ${method}` } };
  }
}

/**
 * Send event to specific origin via content script
 */
async function notifyOrigin(origin, eventName, payload) {
  const tabs = await chrome.tabs.query({ url: `${origin}/*` });
  for (const tab of tabs) {
    try {
      await chrome.tabs.sendMessage(tab.id, {
        type: 'SULTAN_PROVIDER_EVENT',
        eventName,
        payload
      });
    } catch (e) {
      // Tab may not have content script
    }
  }
}

/**
 * Broadcast event to all connected origins
 */
async function broadcastEvent(eventName, payload) {
  for (const origin of connections.keys()) {
    await notifyOrigin(origin, eventName, payload);
  }
}

/**
 * Message handler
 */
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  // Handle content script ready notification
  if (message.type === 'SULTAN_CONTENT_READY') {
    // Could check if origin has existing connection
    return;
  }

  // Handle RPC request from content script
  if (message.type === 'SULTAN_RPC_REQUEST') {
    handleRpcRequest(message.method, message.params, message.origin)
      .then(sendResponse)
      .catch(error => sendResponse({ error: { message: error.message } }));
    return true; // Will respond asynchronously
  }

  // Handle popup requests
  if (message.type === 'SULTAN_POPUP_REQUEST') {
    switch (message.action) {
      case 'getPendingApprovals':
        sendResponse({ approvals: getPendingApprovals() });
        break;

      case 'resolveApproval':
        sendResponse(resolveApproval(message.id, message.approved, message.result));
        break;

      case 'getConnections':
        sendResponse({ connections: Object.fromEntries(connections) });
        break;

      case 'disconnectOrigin':
        removeConnection(message.origin);
        notifyOrigin(message.origin, 'disconnect', {});
        sendResponse({ success: true });
        break;

      case 'disconnectAll':
        for (const origin of connections.keys()) {
          notifyOrigin(origin, 'disconnect', {});
        }
        connections.clear();
        chrome.storage.local.set({ connections: {} });
        sendResponse({ success: true });
        break;

      case 'notifyAccountChange':
        broadcastEvent('accountChange', message.account);
        sendResponse({ success: true });
        break;

      default:
        sendResponse({ error: 'Unknown action' });
    }
    return true;
  }
});

// Load connections on startup
loadConnections();

// Log service worker start
console.log('[Sultan BG] Service worker started');
