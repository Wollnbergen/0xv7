import { Routes, Route, NavLink } from 'react-router-dom';
import { useState } from 'react';
import SwapPage from './pages/SwapPage';
import PoolsPage from './pages/PoolsPage';
import LaunchpadPage from './pages/LaunchpadPage';
import { WalletState, connectWallet } from './api/defiApi';

// Sultan Logo SVG
const SultanLogo = () => (
  <svg viewBox="0 0 40 40" className="w-10 h-10">
    <defs>
      <linearGradient id="goldGradient" x1="0%" y1="0%" x2="100%" y2="100%">
        <stop offset="0%" stopColor="#FFD700" />
        <stop offset="50%" stopColor="#F7931A" />
        <stop offset="100%" stopColor="#FFA500" />
      </linearGradient>
    </defs>
    <circle cx="20" cy="20" r="18" fill="url(#goldGradient)" />
    <text x="20" y="26" textAnchor="middle" fontSize="16" fontWeight="bold" fill="#0D1117">S</text>
  </svg>
);

function App() {
  const [wallet, setWallet] = useState<WalletState>({
    connected: false,
    address: null,
    balance: '0',
  });
  const [connecting, setConnecting] = useState(false);

  const handleConnect = async () => {
    setConnecting(true);
    try {
      const state = await connectWallet();
      setWallet(state);
    } catch (error) {
      console.error('Failed to connect wallet:', error);
    } finally {
      setConnecting(false);
    }
  };

  // Navigation items
  const navItems = [
    { path: '/', label: 'Swap', icon: '🔄' },
    { path: '/pools', label: 'Pools', icon: '💧' },
    { path: '/create', label: 'Create Token', icon: '🪙' },
  ];

  return (
    <div className="min-h-screen bg-sultan-dark">
      {/* Header */}
      <header className="border-b border-white/10 backdrop-blur-md bg-sultan-dark/80 sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            {/* Logo & Brand */}
            <div className="flex items-center gap-3">
              <SultanLogo />
              <div>
                <h1 className="text-xl font-bold text-white">Sultan DeFi</h1>
                <p className="text-xs text-white/50">Zero Gas · Instant Settlement</p>
              </div>
            </div>

            {/* Navigation - Desktop */}
            <nav className="hidden md:flex items-center gap-1 bg-white/5 rounded-xl p-1">
              {navItems.map((item) => (
                <NavLink
                  key={item.path}
                  to={item.path}
                  className={({ isActive }) =>
                    `px-4 py-2 rounded-lg font-medium transition-all ${
                      isActive
                        ? 'bg-sultan-gold text-black'
                        : 'text-white/70 hover:text-white hover:bg-white/10'
                    }`
                  }
                >
                  <span className="mr-2">{item.icon}</span>
                  {item.label}
                </NavLink>
              ))}
            </nav>

            {/* Wallet Button */}
            <button
              onClick={handleConnect}
              disabled={connecting}
              className="btn-gold flex items-center gap-2"
            >
              {connecting ? (
                <>
                  <span className="animate-spin">⟳</span>
                  Connecting...
                </>
              ) : wallet.connected ? (
                <>
                  <span className="w-2 h-2 bg-green-400 rounded-full"></span>
                  {wallet.address?.slice(0, 6)}...{wallet.address?.slice(-4)}
                </>
              ) : (
                <>
                  <span>🔗</span>
                  Connect Wallet
                </>
              )}
            </button>
          </div>
        </div>
      </header>

      {/* Mobile Navigation */}
      <nav className="md:hidden fixed bottom-0 left-0 right-0 bg-sultan-dark/95 backdrop-blur-md border-t border-white/10 z-50">
        <div className="flex justify-around py-3">
          {navItems.map((item) => (
            <NavLink
              key={item.path}
              to={item.path}
              className={({ isActive }) =>
                `flex flex-col items-center gap-1 px-6 py-2 rounded-lg transition-all ${
                  isActive
                    ? 'text-sultan-gold'
                    : 'text-white/50 hover:text-white/70'
                }`
              }
            >
              <span className="text-xl">{item.icon}</span>
              <span className="text-xs font-medium">{item.label}</span>
            </NavLink>
          ))}
        </div>
      </nav>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 py-8 pb-24 md:pb-8">
        <Routes>
          <Route path="/" element={<SwapPage wallet={wallet} />} />
          <Route path="/pools" element={<PoolsPage wallet={wallet} />} />
          <Route path="/create" element={<LaunchpadPage wallet={wallet} />} />
        </Routes>
      </main>

      {/* Network Status Footer */}
      <footer className="fixed bottom-16 md:bottom-0 left-0 right-0 pointer-events-none">
        <div className="max-w-7xl mx-auto px-4 py-2 flex justify-end">
          <div className="glass-card px-3 py-1.5 text-xs flex items-center gap-2 pointer-events-auto">
            <span className="w-2 h-2 bg-green-400 rounded-full animate-pulse"></span>
            <span className="text-white/70">Sultan L1</span>
            <span className="text-white/40">|</span>
            <span className="text-white/50">16 Shards Active</span>
          </div>
        </div>
      </footer>
    </div>
  );
}

export default App;
