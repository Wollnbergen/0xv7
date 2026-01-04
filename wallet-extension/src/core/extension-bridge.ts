/**
 * Sultan Wallet - Extension Bridge
 * 
 * Provides hooks and utilities for the popup UI to communicate
 * with the background service worker and handle dApp approval requests.
 */

/**
 * Check if running as extension (vs standalone web app)
 */
export function isExtensionContext(): boolean {
  return typeof chrome !== 'undefined' && 
         typeof chrome.runtime !== 'undefined' && 
         typeof chrome.runtime.id !== 'undefined';
}

/**
 * Send message to background service worker
 */
export async function sendToBackground<T>(action: string, data: Record<string, unknown> = {}): Promise<T> {
  if (!isExtensionContext()) {
    throw new Error('Not running as extension');
  }

  return new Promise((resolve, reject) => {
    chrome.runtime.sendMessage(
      { type: 'SULTAN_POPUP_REQUEST', action, ...data },
      (response) => {
        if (chrome.runtime.lastError) {
          reject(new Error(chrome.runtime.lastError.message));
        } else if (response?.error) {
          reject(new Error(response.error));
        } else {
          resolve(response as T);
        }
      }
    );
  });
}

/**
 * Approval request from a dApp
 */
export interface ApprovalRequest {
  id: number;
  type: 'connect' | 'signMessage' | 'signTransaction' | 'addToken';
  origin: string;
  data: Record<string, unknown>;
  createdAt: number;
}

/**
 * Get pending approval requests
 */
export async function getPendingApprovals(): Promise<ApprovalRequest[]> {
  const response = await sendToBackground<{ approvals: ApprovalRequest[] }>('getPendingApprovals');
  return response.approvals || [];
}

/**
 * Approve a pending request
 */
export async function approveRequest(id: number, result: unknown): Promise<void> {
  await sendToBackground('resolveApproval', { id, approved: true, result });
}

/**
 * Reject a pending request
 */
export async function rejectRequest(id: number): Promise<void> {
  await sendToBackground('resolveApproval', { id, approved: false });
}

/**
 * Connected dApp info
 */
export interface ConnectedApp {
  origin: string;
  address: string;
  publicKey: string;
  connectedAt: number;
}

/**
 * Get all connected dApps
 */
export async function getConnectedApps(): Promise<ConnectedApp[]> {
  const response = await sendToBackground<{ connections: Record<string, Omit<ConnectedApp, 'origin'>> }>('getConnections');
  return Object.entries(response.connections || {}).map(([origin, conn]) => ({
    origin,
    ...conn
  }));
}

/**
 * Disconnect a specific dApp
 */
export async function disconnectApp(origin: string): Promise<void> {
  await sendToBackground('disconnectOrigin', { origin });
}

/**
 * Disconnect all dApps
 */
export async function disconnectAllApps(): Promise<void> {
  await sendToBackground('disconnectAll');
}

/**
 * Notify dApps of account change
 */
export async function notifyAccountChange(account: { address: string; publicKey: string }): Promise<void> {
  await sendToBackground('notifyAccountChange', { account });
}

/**
 * Format origin for display
 */
export function formatOrigin(origin: string): string {
  try {
    const url = new URL(origin);
    return url.hostname;
  } catch {
    return origin;
  }
}

/**
 * Get favicon URL for origin
 */
export function getFaviconUrl(origin: string): string {
  return `https://www.google.com/s2/favicons?domain=${encodeURIComponent(origin)}&sz=64`;
}
