/**
 * Sultan Wallet - Approval Screen
 * 
 * Displays pending dApp approval requests and allows user to approve/reject.
 */

import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Shield, X, Check, AlertTriangle, Globe, FileText, ArrowRightLeft, Coins } from 'lucide-react';
import { useWallet } from '../hooks/useWallet';
import {
  ApprovalRequest,
  getPendingApprovals,
  approveRequest,
  rejectRequest,
  formatOrigin,
  getFaviconUrl,
  isExtensionContext
} from '../core/extension-bridge';
import '../styles/approval.css';

export function ApprovalScreen() {
  const navigate = useNavigate();
  const { wallet, currentAccount } = useWallet();
  const [approvals, setApprovals] = useState<ApprovalRequest[]>([]);
  const [currentIndex, setCurrentIndex] = useState(0);
  const [loading, setLoading] = useState(true);
  const [processing, setProcessing] = useState(false);
  const [error, setError] = useState('');

  // Load pending approvals
  useEffect(() => {
    async function load() {
      if (!isExtensionContext()) {
        setLoading(false);
        return;
      }

      try {
        const pending = await getPendingApprovals();
        setApprovals(pending);
        if (pending.length === 0) {
          // No pending approvals, go to dashboard
          navigate('/dashboard');
        }
      } catch (e) {
        console.error('Failed to load approvals:', e);
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [navigate]);

  const current = approvals[currentIndex];

  const handleApprove = async () => {
    if (!current || !wallet || !currentAccount) return;
    
    setProcessing(true);
    setError('');

    try {
      let result: unknown;

      switch (current.type) {
        case 'connect':
          result = {
            address: currentAccount.address,
            publicKey: currentAccount.publicKey
          };
          break;

        case 'signMessage': {
          const messageHex = current.data.message as string;
          // Convert hex back to bytes for signing
          const messageBytes = new Uint8Array(
            messageHex.match(/.{1,2}/g)!.map(byte => parseInt(byte, 16))
          );
          const signature = await wallet.signMessage(currentAccount.index, new TextDecoder().decode(messageBytes));
          result = {
            signature,
            publicKey: currentAccount.publicKey
          };
          break;
        }

        case 'signTransaction': {
          const tx = current.data.transaction as Record<string, unknown>;
          const signature = await wallet.signTransaction(tx, currentAccount.index);
          result = {
            signature,
            publicKey: currentAccount.publicKey,
            transaction: tx
          };
          
          // TODO: If broadcast requested, send to RPC
          if (current.data.broadcast) {
            // Would call RPC here
          }
          break;
        }

        case 'addToken':
          // Store token in wallet's token list
          result = true;
          break;
      }

      await approveRequest(current.id, result);
      
      // Move to next or close
      if (currentIndex < approvals.length - 1) {
        setCurrentIndex(currentIndex + 1);
      } else {
        navigate('/dashboard');
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to approve');
    } finally {
      setProcessing(false);
    }
  };

  const handleReject = async () => {
    if (!current) return;
    
    setProcessing(true);
    try {
      await rejectRequest(current.id);
      
      if (currentIndex < approvals.length - 1) {
        setCurrentIndex(currentIndex + 1);
      } else {
        navigate('/dashboard');
      }
    } catch (e) {
      console.error('Failed to reject:', e);
    } finally {
      setProcessing(false);
    }
  };

  const handleRejectAll = async () => {
    setProcessing(true);
    try {
      for (const approval of approvals) {
        await rejectRequest(approval.id);
      }
      navigate('/dashboard');
    } catch (e) {
      console.error('Failed to reject all:', e);
    } finally {
      setProcessing(false);
    }
  };

  if (loading) {
    return (
      <div className="approval-screen">
        <div className="approval-loading">
          <div className="spinner" />
          <p>Loading...</p>
        </div>
      </div>
    );
  }

  if (!current) {
    return null;
  }

  const getTypeIcon = () => {
    switch (current.type) {
      case 'connect': return <Globe className="type-icon connect" />;
      case 'signMessage': return <FileText className="type-icon sign" />;
      case 'signTransaction': return <ArrowRightLeft className="type-icon transaction" />;
      case 'addToken': return <Coins className="type-icon token" />;
    }
  };

  const getTypeTitle = () => {
    switch (current.type) {
      case 'connect': return 'Connection Request';
      case 'signMessage': return 'Sign Message';
      case 'signTransaction': return 'Sign Transaction';
      case 'addToken': return 'Add Token';
    }
  };

  const getTypeDescription = () => {
    switch (current.type) {
      case 'connect':
        return 'This site wants to connect to your Sultan Wallet';
      case 'signMessage':
        return 'This site wants you to sign a message';
      case 'signTransaction':
        return 'This site wants you to sign a transaction';
      case 'addToken':
        return 'This site wants to add a token to your wallet';
    }
  };

  return (
    <div className="approval-screen">
      {/* Header */}
      <header className="approval-header">
        <Shield className="shield-icon" />
        <span className="header-title">Sultan Wallet</span>
        {approvals.length > 1 && (
          <span className="approval-count">
            {currentIndex + 1} of {approvals.length}
          </span>
        )}
      </header>

      {/* Origin */}
      <div className="origin-card">
        <img 
          src={getFaviconUrl(current.origin)} 
          alt="" 
          className="origin-favicon"
          onError={(e) => {
            (e.target as HTMLImageElement).style.display = 'none';
          }}
        />
        <div className="origin-info">
          <span className="origin-name">{formatOrigin(current.origin)}</span>
          <span className="origin-url">{current.origin}</span>
        </div>
      </div>

      {/* Request Type */}
      <div className="request-card">
        {getTypeIcon()}
        <h2 className="request-title">{getTypeTitle()}</h2>
        <p className="request-description">{getTypeDescription()}</p>
      </div>

      {/* Request Details */}
      <div className="details-card">
        {current.type === 'connect' && currentAccount && (
          <div className="detail-row">
            <span className="detail-label">Account</span>
            <span className="detail-value">{currentAccount.name}</span>
          </div>
        )}

        {current.type === 'signMessage' && (
          <div className="message-preview">
            <span className="detail-label">Message</span>
            <pre className="message-content">
              {(() => {
                const hex = current.data.message as string;
                const bytes = new Uint8Array(hex.match(/.{1,2}/g)!.map(b => parseInt(b, 16)));
                return new TextDecoder().decode(bytes);
              })()}
            </pre>
          </div>
        )}

        {current.type === 'signTransaction' && (
          <div className="transaction-preview">
            <span className="detail-label">Transaction</span>
            <pre className="transaction-content">
              {JSON.stringify(current.data.transaction, null, 2)}
            </pre>
          </div>
        )}

        {current.type === 'addToken' && (
          <>
            <div className="detail-row">
              <span className="detail-label">Symbol</span>
              <span className="detail-value">{(current.data.token as any)?.symbol}</span>
            </div>
            <div className="detail-row">
              <span className="detail-label">Denom</span>
              <span className="detail-value">{(current.data.token as any)?.denom}</span>
            </div>
          </>
        )}
      </div>

      {/* Warning */}
      <div className="warning-banner">
        <AlertTriangle className="warning-icon" />
        <span>Only approve requests from sites you trust</span>
      </div>

      {/* Error */}
      {error && (
        <div className="error-banner">
          <span>{error}</span>
        </div>
      )}

      {/* Actions */}
      <div className="approval-actions">
        <button 
          className="btn btn-reject"
          onClick={handleReject}
          disabled={processing}
        >
          <X className="btn-icon" />
          Reject
        </button>
        <button 
          className="btn btn-approve"
          onClick={handleApprove}
          disabled={processing}
        >
          <Check className="btn-icon" />
          Approve
        </button>
      </div>

      {/* Reject All */}
      {approvals.length > 1 && (
        <button 
          className="btn-link reject-all"
          onClick={handleRejectAll}
          disabled={processing}
        >
          Reject all {approvals.length} requests
        </button>
      )}
    </div>
  );
}

export default ApprovalScreen;
