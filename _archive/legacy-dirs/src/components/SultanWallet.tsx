import React, { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { useWallet } from '../contexts/WalletContext';
import { useQuery } from '@tanstack/react-query';
import { SdkService } from '../services/SdkService';

const sdkService = new SdkService();

const SultanWallet: React.FC = () => {
  const { sendTransaction, stake, getValidatorInfo } = useWallet();
  const [chain, setChain] = useState('sultan');
  const [toAddress, setToAddress] = useState('');
  const [amount, setAmount] = useState('');
  const [stakeAmount, setStakeAmount] = useState(5000);
  const [balances, setBalances] = useState<{ [key: string]: string }>({});
  const [loading, setLoading] = useState(true);
  const [btcBalance, setBtcBalance] = useState('0'); // BTC specific for test

  const { data: validatorData } = useQuery(['validator'], getValidatorInfo);

  useEffect(() => {
    async function load() {
      setLoading(true);
      const address = localStorage.getItem('walletAddress') || '';
      const bal = await sdkService.getBalance(chain, '', address);
      setBalances((prev) => ({ ...prev, [chain]: bal }));
      setLoading(false);
    }
    load();
  }, [chain]);

  useEffect(() => {
    async function loadBtc() {
      // Satoshi's Genesis address, expected ~68 BTC
      const bal = await sdkService.getBalance('bitcoin', '', '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa');
      setBtcBalance(bal); // Satoshis; convert to BTC in UI
    }
    loadBtc();
  }, []);

  const handleSend = async () => {
    await sendTransaction(chain, toAddress, amount, 'SLTN');
  };

  const handleStake = async () => {
    if (stakeAmount < 5000) return;
    await stake(stakeAmount);
  };

  const handleSwap = async () => {
    await sdkService.crossChainSwap(chain, amount);
    if (window.Telegram && window.Telegram.WebApp) {
      window.Telegram.WebApp.showAlert('Swapped to SLTN (atomic, gas-free on Sultan)');
    } else {
      alert('Swapped to SLTN (atomic, gas-free on Sultan)');
    }
  };

  if (loading) return <div>Loading...</div>;
  return (
    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="wallet-container">
      <h1>Sultan Wallet</h1>
      <select onChange={(e) => setChain(e.target.value)}>
        <option value="sultan">Sultan (Zero Gas Internal)</option>
        <option value="ethereum">Ethereum</option>
        <option value="solana">Solana</option>
        <option value="ton">TON</option>
        <option value="bitcoin">Bitcoin</option>
      </select>
      <p>Balance: {balances[chain]} {chain.toUpperCase()}</p>
      <p>BTC Balance: {Number(btcBalance) / 1e8} BTC (real from Blockstream, fallback to 0 on error)</p>
      <p>Validator: {validatorData?.apy}% APY (Min 5,000 SLTN)</p>
      <input type="number" value={stakeAmount} onChange={(e) => setStakeAmount(Number(e.target.value))} />
      <button onClick={handleStake}>Stake to Become Validator</button>
      <input type="text" value={toAddress} onChange={(e) => setToAddress(e.target.value)} placeholder="@username or address" />
      <input type="number" value={amount} onChange={(e) => setAmount(e.target.value)} />
      <button onClick={handleSend}>Send</button>
      <button onClick={handleSwap}>Swap</button>
      <p>Zero Gas on Sultan Network!</p>
    </motion.div>
  );
};

export default SultanWallet;