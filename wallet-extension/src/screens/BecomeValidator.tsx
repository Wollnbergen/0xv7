/**
 * Become Validator Screen
 * 
 * Register as a Sultan network validator with 10,000 SLTN minimum stake.
 * Earn 13.33% APY + commission on delegator rewards.
 */

import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useWallet } from '../hooks/useWallet';
import { useTheme } from '../hooks/useTheme';
import { useBalance } from '../hooks/useBalance';
import { SultanWallet } from '../core/wallet';
import { sultanAPI } from '../api/sultanAPI';
import { validateAmount } from '../core/security';
import './BecomeValidator.css';

// Premium SVG Icons
const BackIcon = () => (
  <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <polyline points="15 18 9 12 15 6" />
  </svg>
);

const SunIcon = () => (
  <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <circle cx="12" cy="12" r="5" />
    <line x1="12" y1="1" x2="12" y2="3" />
    <line x1="12" y1="21" x2="12" y2="23" />
    <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
    <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
    <line x1="1" y1="12" x2="3" y2="12" />
    <line x1="21" y1="12" x2="23" y2="12" />
    <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
    <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
  </svg>
);

const MoonIcon = () => (
  <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
  </svg>
);

const ValidatorIcon = () => (
  <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
    <path d="M12 2L2 7l10 5 10-5-10-5z" />
    <path d="M2 17l10 5 10-5" />
    <path d="M2 12l10 5 10-5" />
  </svg>
);

const CheckIcon = () => (
  <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <polyline points="20 6 9 17 4 12" />
  </svg>
);

// Minimum 10,000 SLTN required to become a validator

export default function BecomeValidator() {
  const navigate = useNavigate();
  const { wallet, currentAccount } = useWallet();
  const { theme, setTheme } = useTheme();
  const { data: balanceData, refetch: refetchBalance } = useBalance(currentAccount?.address);
  
  const [moniker, setMoniker] = useState('');
  const [stake, setStake] = useState('10000');
  const [commission, setCommission] = useState('10');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');

  const toggleTheme = () => {
    setTheme(theme === 'dark' ? 'light' : 'dark');
  };

  const availableBalance = SultanWallet.formatSLTN(balanceData?.available || '0');
  const hasMinimumStake = parseFloat(availableBalance) >= 10000;

  const handleCreateValidator = async () => {
    if (!wallet || !currentAccount) return;

    // Validate moniker
    if (!moniker.trim()) {
      setError('Validator name is required');
      return;
    }
    if (moniker.length < 3 || moniker.length > 32) {
      setError('Validator name must be 3-32 characters');
      return;
    }

    // Validate stake amount
    const stakeNum = parseFloat(stake);
    if (isNaN(stakeNum) || stakeNum < 10000) {
      setError('Minimum stake is 10,000 SLTN');
      return;
    }

    // Validate against balance
    const amountValidation = validateAmount(stake, availableBalance);
    if (!amountValidation.valid) {
      setError(amountValidation.error || 'Insufficient balance');
      return;
    }

    // Validate commission
    const commissionNum = parseFloat(commission);
    if (isNaN(commissionNum) || commissionNum < 0 || commissionNum > 100) {
      setError('Commission must be between 0% and 100%');
      return;
    }

    setIsLoading(true);
    setError('');
    setSuccess('');

    try {
      const atomicAmount = SultanWallet.parseSLTN(stake);
      
      const txData = {
        type: 'create_validator' as const,
        from: currentAccount.address,
        moniker: moniker.trim(),
        amount: atomicAmount,
        commission: commissionNum / 100,
        timestamp: Date.now(),
      };

      const signature = await wallet.signTransaction(txData, currentAccount.index);
      
      await sultanAPI.createValidator({
        validatorAddress: currentAccount.address,
        moniker: moniker.trim(),
        initialStake: atomicAmount,
        commissionRate: commissionNum / 100,
        signature,
        publicKey: currentAccount.publicKey,
      });

      setSuccess(`🎉 Congratulations! You are now a Sultan validator!`);
      refetchBalance();
      
      // Navigate to stake screen after 3 seconds
      setTimeout(() => {
        navigate('/stake');
      }, 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create validator');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="container">
      <header className="header">
        <button className="back-btn" onClick={() => navigate('/stake')}>
          <BackIcon />
        </button>
        <h1 className="title">Become Validator</h1>
        <button className="theme-toggle" onClick={toggleTheme}>
          {theme === 'dark' ? <SunIcon /> : <MoonIcon />}
        </button>
      </header>

      <main className="main-content">
        {/* Hero Section */}
        <div className="validator-hero">
          <div className="validator-icon">
            <ValidatorIcon />
          </div>
          <h2>Become a Sultan Validator</h2>
          <p className="hero-subtitle">
            Secure the network and earn rewards
          </p>
        </div>

        {/* Benefits */}
        <div className="benefits-card">
          <h3>Validator Benefits</h3>
          <ul className="benefits-list">
            <li><CheckIcon /> <span>13.33% Base APY on staked tokens</span></li>
            <li><CheckIcon /> <span>Commission on delegator rewards</span></li>
            <li><CheckIcon /> <span>Zero gas fees on all transactions</span></li>
            <li><CheckIcon /> <span>Help secure the Sultan network</span></li>
            <li><CheckIcon /> <span>Participate in governance decisions</span></li>
          </ul>
        </div>

        {/* Requirements */}
        <div className="requirements-card">
          <h3>Requirements</h3>
          <div className="requirement-item">
            <span className="requirement-label">Minimum Stake</span>
            <span className="requirement-value">10,000 SLTN</span>
          </div>
          <div className="requirement-item">
            <span className="requirement-label">Your Balance</span>
            <span className={`requirement-value ${hasMinimumStake ? 'success' : 'error'}`}>
              {parseFloat(availableBalance).toLocaleString()} SLTN
            </span>
          </div>
          <div className="requirement-item">
            <span className="requirement-label">Unbonding Period</span>
            <span className="requirement-value">21 days</span>
          </div>
        </div>

        {/* Form */}
        {hasMinimumStake ? (
          <div className="validator-form">
            <div className="form-group">
              <label>Validator Name (Moniker)</label>
              <input
                type="text"
                className="input"
                placeholder="e.g., MyValidator"
                value={moniker}
                onChange={(e) => setMoniker(e.target.value)}
                maxLength={32}
              />
              <span className="input-hint">{moniker.length}/32 characters</span>
            </div>

            <div className="form-group">
              <label>Initial Stake (SLTN)</label>
              <input
                type="number"
                className="input"
                placeholder="10000"
                value={stake}
                onChange={(e) => setStake(e.target.value)}
                min="10000"
              />
              <span className="input-hint">Minimum: 10,000 SLTN</span>
            </div>

            <div className="form-group">
              <label>Commission Rate (%)</label>
              <input
                type="number"
                className="input"
                placeholder="10"
                value={commission}
                onChange={(e) => setCommission(e.target.value)}
                min="0"
                max="100"
                step="0.1"
              />
              <span className="input-hint">
                You earn {commission}% of delegator rewards as commission
              </span>
            </div>

            {/* Estimated Earnings */}
            <div className="earnings-preview">
              <h4>Estimated Annual Earnings</h4>
              <div className="earnings-row">
                <span>Staking Rewards (13.33% APY)</span>
                <span className="earnings-value">
                  +{((parseFloat(stake) || 0) * 0.1333).toLocaleString(undefined, { maximumFractionDigits: 0 })} SLTN/year
                </span>
              </div>
              <div className="earnings-row">
                <span>Commission (estimated)</span>
                <span className="earnings-value">+Variable</span>
              </div>
            </div>

            {error && <div className="error-message">{error}</div>}
            {success && <div className="success-message">{success}</div>}

            <button
              className="btn btn-primary btn-lg"
              onClick={handleCreateValidator}
              disabled={isLoading || !moniker.trim()}
            >
              {isLoading ? (
                <><span className="spinner-small" /> Creating Validator...</>
              ) : (
                '🚀 Become a Validator'
              )}
            </button>
          </div>
        ) : (
          <div className="insufficient-funds">
            <p>You need at least <strong>10,000 SLTN</strong> to become a validator.</p>
            <p className="current-balance">
              Current balance: <strong>{parseFloat(availableBalance).toLocaleString()} SLTN</strong>
            </p>
            <button className="btn btn-secondary" onClick={() => navigate('/receive')}>
              Receive SLTN
            </button>
          </div>
        )}

        {/* Warning */}
        <div className="warning-card">
          <h4>⚠️ Important</h4>
          <ul>
            <li>Staked tokens are locked for 21 days when unbonding</li>
            <li>Validators can be slashed for downtime or misbehavior</li>
            <li>Ensure you can maintain high uptime (99.9%+)</li>
          </ul>
        </div>
      </main>
    </div>
  );
}
