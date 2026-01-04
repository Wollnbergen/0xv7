import { useState, useEffect } from 'react';
import {
  WalletState,
  LaunchpadProject,
  getLaunchpadProjects,
  createToken,
  formatAmount,
} from '../api/defiApi';
import { useToast } from '../components';

interface LaunchpadPageProps {
  wallet: WalletState;
}

type TabType = 'create' | 'projects' | 'my';

export default function LaunchpadPage({ wallet }: LaunchpadPageProps) {
  const { showToast } = useToast();
  const [activeTab, setActiveTab] = useState<TabType>('create');
  const [projects, setProjects] = useState<LaunchpadProject[]>([]);
  const [loading, setLoading] = useState(false);

  // Token creation form
  const [tokenName, setTokenName] = useState('');
  const [tokenSymbol, setTokenSymbol] = useState('');
  const [tokenDecimals, setTokenDecimals] = useState(9);
  const [tokenSupply, setTokenSupply] = useState('');
  const [tokenDescription, setTokenDescription] = useState('');
  const [tokenLogo, setTokenLogo] = useState('');
  const [creating, setCreating] = useState(false);
  const [createdToken, setCreatedToken] = useState<{ denom: string; txHash: string } | null>(null);

  // Load projects
  useEffect(() => {
    async function load() {
      setLoading(true);
      try {
        const projectList = await getLaunchpadProjects();
        setProjects(projectList);
      } catch (error) {
        console.error('Failed to load projects:', error);
      } finally {
        setLoading(false);
      }
    }
    if (activeTab === 'projects' || activeTab === 'my') {
      load();
    }
  }, [activeTab]);

  // Handle token creation
  const handleCreateToken = async () => {
    if (!wallet.connected || !tokenName || !tokenSymbol || !tokenSupply) return;

    // Validation
    if (tokenName.length < 2) {
      showToast('Token name must be at least 2 characters', 'error');
      return;
    }
    if (tokenSymbol.length < 2 || tokenSymbol.length > 10) {
      showToast('Symbol must be 2-10 characters', 'error');
      return;
    }
    if (parseFloat(tokenSupply) <= 0) {
      showToast('Supply must be greater than 0', 'error');
      return;
    }

    setCreating(true);
    try {
      const result = await createToken(
        {
          name: tokenName,
          symbol: tokenSymbol.toUpperCase(),
          decimals: tokenDecimals,
          totalSupply: (parseFloat(tokenSupply) * Math.pow(10, tokenDecimals)).toString(),
          creator: wallet.address!,
          logo: tokenLogo || undefined,
          description: tokenDescription || undefined,
        },
        'signature_placeholder'
      );
      setCreatedToken(result);
      showToast(`Token ${tokenSymbol.toUpperCase()} created successfully!`, 'success');
      // Reset form
      setTokenName('');
      setTokenSymbol('');
      setTokenSupply('');
      setTokenDescription('');
      setTokenLogo('');
    } catch (error) {
      console.error('Failed to create token:', error);
      showToast('Failed to create token. Please try again.', 'error');
    } finally {
      setCreating(false);
    }
  };

  return (
    <div className="max-w-4xl mx-auto">
      {/* Page Header */}
      <div className="text-center mb-8">
        <h2 className="text-3xl font-bold text-white mb-2">🪙 Create Token</h2>
        <p className="text-white/60">Create and manage tokens on Sultan L1 with zero fees</p>
      </div>

      {/* Tabs */}
      <div className="flex gap-2 mb-8 justify-center">
        <button
          onClick={() => setActiveTab('create')}
          className={`px-6 py-2 rounded-lg font-medium transition-colors ${
            activeTab === 'create'
              ? 'bg-sultan-gold text-black'
              : 'bg-white/10 text-white/70 hover:bg-white/20'
          }`}
        >
          Create Token
        </button>
        <button
          onClick={() => setActiveTab('projects')}
          className={`px-6 py-2 rounded-lg font-medium transition-colors ${
            activeTab === 'projects'
              ? 'bg-sultan-gold text-black'
              : 'bg-white/10 text-white/70 hover:bg-white/20'
          }`}
        >
          All Tokens
        </button>
        <button
          onClick={() => setActiveTab('my')}
          className={`px-6 py-2 rounded-lg font-medium transition-colors ${
            activeTab === 'my'
              ? 'bg-sultan-gold text-black'
              : 'bg-white/10 text-white/70 hover:bg-white/20'
          }`}
        >
          My Tokens
        </button>
      </div>

      {/* Create Token Tab */}
      {activeTab === 'create' && (
        <div className="max-w-lg mx-auto">
          {/* Token Creation Form */}
          <div className="glass-card">
            <h3 className="text-xl font-semibold text-white mb-6 flex items-center gap-2">
              <span>🪙</span> Create New Token
            </h3>

            {createdToken ? (
              <div className="text-center py-8">
                <div className="text-5xl mb-4">🎉</div>
                <h4 className="text-xl font-bold text-white mb-2">Token Created!</h4>
                <p className="text-white/60 mb-4">Your token has been minted on Sultan L1</p>
                <div className="bg-white/5 rounded-lg p-4 text-left space-y-2 mb-6">
                  <div>
                    <span className="text-white/50 text-sm">Denom:</span>
                    <p className="text-white font-mono text-sm break-all">{createdToken.denom}</p>
                  </div>
                  <div>
                    <span className="text-white/50 text-sm">Transaction:</span>
                    <p className="text-white font-mono text-sm break-all">{createdToken.txHash}</p>
                  </div>
                </div>
                <button onClick={() => setCreatedToken(null)} className="btn-gold">
                  Create Another Token
                </button>
              </div>
            ) : (
              <div className="space-y-4">
                {/* Token Name */}
                <div>
                  <label className="text-white/70 text-sm block mb-1">Token Name *</label>
                  <input
                    type="text"
                    placeholder="e.g., Sultan Gold"
                    value={tokenName}
                    onChange={(e) => setTokenName(e.target.value)}
                    className="token-input w-full"
                    maxLength={50}
                  />
                </div>

                {/* Token Symbol */}
                <div>
                  <label className="text-white/70 text-sm block mb-1">Symbol *</label>
                  <input
                    type="text"
                    placeholder="e.g., SGOLD"
                    value={tokenSymbol}
                    onChange={(e) => setTokenSymbol(e.target.value.toUpperCase())}
                    className="token-input w-full"
                    maxLength={10}
                  />
                </div>

                {/* Decimals */}
                <div>
                  <label className="text-white/70 text-sm block mb-1">Decimals</label>
                  <select
                    value={tokenDecimals}
                    onChange={(e) => setTokenDecimals(parseInt(e.target.value))}
                    className="token-input w-full bg-white/5"
                  >
                    <option value={6}>6 (like USDC)</option>
                    <option value={8}>8 (like BTC)</option>
                    <option value={9}>9 (Sultan default)</option>
                    <option value={18}>18 (like ETH)</option>
                  </select>
                </div>

                {/* Total Supply */}
                <div>
                  <label className="text-white/70 text-sm block mb-1">Total Supply *</label>
                  <input
                    type="number"
                    placeholder="e.g., 1000000000"
                    value={tokenSupply}
                    onChange={(e) => setTokenSupply(e.target.value)}
                    className="token-input w-full"
                    min="1"
                  />
                </div>

                {/* Description */}
                <div>
                  <label className="text-white/70 text-sm block mb-1">Description (optional)</label>
                  <textarea
                    placeholder="Describe your token..."
                    value={tokenDescription}
                    onChange={(e) => setTokenDescription(e.target.value)}
                    className="token-input w-full h-20 resize-none"
                    maxLength={500}
                  />
                </div>

                {/* Logo URL */}
                <div>
                  <label className="text-white/70 text-sm block mb-1">Logo URL (optional)</label>
                  <input
                    type="url"
                    placeholder="https://..."
                    value={tokenLogo}
                    onChange={(e) => setTokenLogo(e.target.value)}
                    className="token-input w-full"
                  />
                </div>

                {/* Fee Info */}
                <div className="bg-green-500/10 rounded-lg p-3 text-sm">
                  <p className="text-green-400 flex items-center gap-2">
                    ✓ Zero gas fees on Sultan L1
                  </p>
                </div>

                {/* Create Button */}
                <button
                  onClick={handleCreateToken}
                  disabled={!wallet.connected || !tokenName || !tokenSymbol || !tokenSupply || creating}
                  className="btn-gold w-full disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {creating ? (
                    <span className="flex items-center justify-center gap-2">
                      <span className="animate-spin">⟳</span> Creating...
                    </span>
                  ) : !wallet.connected ? (
                    'Connect Wallet to Create'
                  ) : (
                    'Create Token'
                  )}
                </button>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Projects Tab */}
      {activeTab === 'projects' && (
        <div>
          {loading ? (
            <div className="glass-card text-center py-12">
              <div className="animate-spin text-4xl mb-4">⟳</div>
              <p className="text-white/50">Loading tokens...</p>
            </div>
          ) : projects.length === 0 ? (
            <div className="glass-card text-center py-12">
              <div className="text-5xl mb-4">🌟</div>
              <h3 className="text-xl font-bold text-white mb-2">No Tokens Yet</h3>
              <p className="text-white/60 mb-6">Be the first to create a token on Sultan L1!</p>
              <button onClick={() => setActiveTab('create')} className="btn-gold">
                Create Token
              </button>
            </div>
          ) : (
            <div className="grid md:grid-cols-2 gap-4">
              {projects.map((project) => (
                <div key={project.id} className="glass-card">
                  <div className="flex items-center gap-4 mb-4">
                    <div className="w-12 h-12 rounded-full bg-sultan-gold/20 flex items-center justify-center text-sultan-gold text-xl font-bold">
                      {project.token.symbol.charAt(0)}
                    </div>
                    <div>
                      <h4 className="text-white font-semibold">{project.token.name}</h4>
                      <p className="text-white/50 text-sm">{project.token.symbol}</p>
                    </div>
                    <div className="ml-auto">
                      <span
                        className={`px-2 py-1 rounded text-xs font-medium ${
                          project.status === 'completed'
                            ? 'bg-green-500/20 text-green-400'
                            : project.status === 'launched'
                            ? 'bg-blue-500/20 text-blue-400'
                            : 'bg-white/10 text-white/60'
                        }`}
                      >
                        {project.status}
                      </span>
                    </div>
                  </div>
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <p className="text-white/50">Total Supply</p>
                      <p className="text-white font-medium">
                        {formatAmount(project.token.totalSupply, project.token.decimals)}
                      </p>
                    </div>
                    <div>
                      <p className="text-white/50">Created</p>
                      <p className="text-white font-medium">
                        {new Date(project.createdAt * 1000).toLocaleDateString()}
                      </p>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* My Tokens Tab */}
      {activeTab === 'my' && (
        <div>
          {!wallet.connected ? (
            <div className="glass-card text-center py-12">
              <div className="text-5xl mb-4">🔗</div>
              <p className="text-white/70 mb-4">Connect your wallet to view your tokens</p>
            </div>
          ) : (
            <div className="glass-card text-center py-12">
              <div className="text-5xl mb-4">📭</div>
              <h3 className="text-xl font-bold text-white mb-2">No Tokens Yet</h3>
              <p className="text-white/60 mb-6">You haven't created any tokens yet</p>
              <button onClick={() => setActiveTab('create')} className="btn-gold">
                Create Your First Token
              </button>
            </div>
          )}
        </div>
      )}

      {/* Bottom Info */}
      <div className="mt-12 grid md:grid-cols-3 gap-6 text-center">
        <div className="glass-card">
          <div className="text-3xl mb-3">⚡</div>
          <h4 className="text-white font-semibold mb-1">Instant Creation</h4>
          <p className="text-white/50 text-sm">Tokens minted in ~1 second with instant finality</p>
        </div>
        <div className="glass-card">
          <div className="text-3xl mb-3">💰</div>
          <h4 className="text-white font-semibold mb-1">Zero Fees</h4>
          <p className="text-white/50 text-sm">No gas costs for token creation or transfers</p>
        </div>
        <div className="glass-card">
          <div className="text-3xl mb-3">🌐</div>
          <h4 className="text-white font-semibold mb-1">Native DEX</h4>
          <p className="text-white/50 text-sm">Instantly tradeable on Sultan's built-in DEX</p>
        </div>
      </div>
    </div>
  );
}
