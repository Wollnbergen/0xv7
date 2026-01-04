import { useState, useEffect } from 'react';
import {
  WalletState,
  Pool,
  Position,
  Token,
  getPools,
  getUserPositions,
  getTokens,
  addLiquidity,
  removeLiquidity,
  formatAmount,
  parseAmount,
} from '../api/defiApi';
import { useToast, Modal, TokenPairIcon } from '../components';

interface PoolsPageProps {
  wallet: WalletState;
}

// Tab type
type TabType = 'all' | 'my';

export default function PoolsPage({ wallet }: PoolsPageProps) {
  const { showToast } = useToast();
  const [activeTab, setActiveTab] = useState<TabType>('all');
  const [pools, setPools] = useState<Pool[]>([]);
  const [positions, setPositions] = useState<Position[]>([]);
  const [tokens, setTokens] = useState<Token[]>([]);
  const [loading, setLoading] = useState(true);

  // Add liquidity modal
  const [showAddModal, setShowAddModal] = useState(false);
  const [selectedPool, setSelectedPool] = useState<Pool | null>(null);
  const [amount0, setAmount0] = useState('');
  const [amount1, setAmount1] = useState('');
  const [addingLiquidity, setAddingLiquidity] = useState(false);

  // Remove liquidity modal
  const [showRemoveModal, setShowRemoveModal] = useState(false);
  const [selectedPosition, setSelectedPosition] = useState<Position | null>(null);
  const [removePercent, setRemovePercent] = useState(100);
  const [removingLiquidity, setRemovingLiquidity] = useState(false);

  // Create pool modal
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newPoolToken0, setNewPoolToken0] = useState<Token | null>(null);
  const [newPoolToken1, setNewPoolToken1] = useState<Token | null>(null);
  const [newPoolAmount0, setNewPoolAmount0] = useState('');
  const [newPoolAmount1, setNewPoolAmount1] = useState('');

  // Load data
  useEffect(() => {
    async function load() {
      setLoading(true);
      try {
        const [poolList, tokenList] = await Promise.all([getPools(), getTokens()]);
        setPools(poolList);
        setTokens(tokenList);

        if (wallet.connected && wallet.address) {
          const userPositions = await getUserPositions(wallet.address);
          setPositions(userPositions);
        }
      } catch (error) {
        console.error('Failed to load pools:', error);
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [wallet.connected, wallet.address]);

  // Handle add liquidity
  const handleAddLiquidity = async () => {
    if (!wallet.connected || !selectedPool) return;

    setAddingLiquidity(true);
    try {
      const result = await addLiquidity(
        {
          user: wallet.address!,
          token0: selectedPool.token0.denom,
          token1: selectedPool.token1.denom,
          amount0: parseAmount(amount0, selectedPool.token0.decimals),
          amount1: parseAmount(amount1, selectedPool.token1.decimals),
          slippage: 0.5,
        },
        'signature_placeholder'
      );
      showToast(`Added liquidity: ${result.lpTokens} LP tokens`, 'success');
      setShowAddModal(false);
      setAmount0('');
      setAmount1('');
    } catch (error) {
      console.error('Failed to add liquidity:', error);
      showToast('Failed to add liquidity', 'error');
    } finally {
      setAddingLiquidity(false);
    }
  };

  // Handle remove liquidity
  const handleRemoveLiquidity = async () => {
    if (!wallet.connected || !selectedPosition) return;

    setRemovingLiquidity(true);
    try {
      const lpToRemove = (
        (BigInt(selectedPosition.lpTokens) * BigInt(removePercent)) /
        BigInt(100)
      ).toString();

      const result = await removeLiquidity(
        {
          user: wallet.address!,
          poolId: selectedPosition.poolId,
          lpTokens: lpToRemove,
          minAmount0: '0',
          minAmount1: '0',
        },
        'signature_placeholder'
      );
      showToast(`Removed liquidity: received ${formatAmount(result.amount0)} + ${formatAmount(result.amount1)}`, 'success');
      setShowRemoveModal(false);
    } catch (error) {
      console.error('Failed to remove liquidity:', error);
      showToast('Failed to remove liquidity', 'error');
    } finally {
      setRemovingLiquidity(false);
    }
  };

  return (
    <div className="max-w-4xl mx-auto">
      {/* Page Header */}
      <div className="flex flex-col md:flex-row justify-between items-start md:items-center mb-8 gap-4">
        <div>
          <h2 className="text-3xl font-bold text-white mb-2">Liquidity Pools</h2>
          <p className="text-white/60">Provide liquidity and earn trading fees</p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="btn-gold flex items-center gap-2"
        >
          <span>➕</span> Create Pool
        </button>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
        <div className="glass-card">
          <p className="text-white/50 text-sm">Total TVL</p>
          <p className="text-2xl font-bold text-white">$0.00</p>
        </div>
        <div className="glass-card">
          <p className="text-white/50 text-sm">24h Volume</p>
          <p className="text-2xl font-bold text-white">$0.00</p>
        </div>
        <div className="glass-card">
          <p className="text-white/50 text-sm">Total Pools</p>
          <p className="text-2xl font-bold text-white">{pools.length}</p>
        </div>
        <div className="glass-card">
          <p className="text-white/50 text-sm">Your Positions</p>
          <p className="text-2xl font-bold text-white">{positions.length}</p>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-2 mb-6">
        <button
          onClick={() => setActiveTab('all')}
          className={`px-4 py-2 rounded-lg font-medium transition-colors ${
            activeTab === 'all'
              ? 'bg-sultan-gold text-black'
              : 'bg-white/10 text-white/70 hover:bg-white/20'
          }`}
        >
          All Pools
        </button>
        <button
          onClick={() => setActiveTab('my')}
          className={`px-4 py-2 rounded-lg font-medium transition-colors ${
            activeTab === 'my'
              ? 'bg-sultan-gold text-black'
              : 'bg-white/10 text-white/70 hover:bg-white/20'
          }`}
        >
          My Positions
        </button>
      </div>

      {/* Content */}
      {loading ? (
        <div className="glass-card text-center py-12">
          <div className="animate-spin text-4xl mb-4">⟳</div>
          <p className="text-white/50">Loading pools...</p>
        </div>
      ) : activeTab === 'all' ? (
        /* All Pools */
        <div className="space-y-4">
          {pools.length === 0 ? (
            <div className="glass-card text-center py-12">
              <div className="text-4xl mb-4">💧</div>
              <p className="text-white mb-4">No pools yet. Be the first to create one!</p>
              <button onClick={() => setShowCreateModal(true)} className="btn-gold">
                Create Pool
              </button>
            </div>
          ) : (
            pools.map((pool) => (
              <div
                key={pool.id}
                className="glass-card flex flex-col md:flex-row items-start md:items-center justify-between gap-4"
              >
                <div className="flex items-center gap-4">
                  <TokenPairIcon token0={pool.token0} token1={pool.token1} size="md" />
                  <div>
                    <p className="text-white font-semibold">
                      {pool.token0.symbol} / {pool.token1.symbol}
                    </p>
                    <p className="text-white/50 text-sm">Fee: {pool.feeRate / 100}%</p>
                  </div>
                </div>

                <div className="flex gap-8 text-sm">
                  <div>
                    <p className="text-white/50">TVL</p>
                    <p className="text-white font-medium">${pool.tvl || '0.00'}</p>
                  </div>
                  <div>
                    <p className="text-white/50">24h Vol</p>
                    <p className="text-white font-medium">${pool.volume24h || '0.00'}</p>
                  </div>
                </div>

                <button
                  onClick={() => {
                    setSelectedPool(pool);
                    setShowAddModal(true);
                  }}
                  className="btn-gold text-sm"
                >
                  Add Liquidity
                </button>
              </div>
            ))
          )}
        </div>
      ) : (
        /* My Positions */
        <div className="space-y-4">
          {!wallet.connected ? (
            <div className="glass-card text-center py-12">
              <div className="text-4xl mb-4">🔗</div>
              <p className="text-white/70">Connect your wallet to view your positions</p>
            </div>
          ) : positions.length === 0 ? (
            <div className="glass-card text-center py-12">
              <div className="text-4xl mb-4">📭</div>
              <p className="text-white mb-4">You don't have any liquidity positions yet</p>
              <button onClick={() => setActiveTab('all')} className="btn-gold">
                Browse Pools
              </button>
            </div>
          ) : (
            positions.map((position) => (
              <div
                key={position.poolId}
                className="glass-card flex flex-col md:flex-row items-start md:items-center justify-between gap-4"
              >
                <div className="flex items-center gap-4">
                  <TokenPairIcon token0={position.token0} token1={position.token1} size="md" />
                  <div>
                    <p className="text-white font-semibold">
                      {position.token0.symbol} / {position.token1.symbol}
                    </p>
                    <p className="text-white/50 text-sm">Pool Share: {position.share.toFixed(2)}%</p>
                  </div>
                </div>

                <div className="flex gap-6 text-sm">
                  <div>
                    <p className="text-white/50">{position.token0.symbol}</p>
                    <p className="text-white font-medium">
                      {formatAmount(position.token0Amount, position.token0.decimals)}
                    </p>
                  </div>
                  <div>
                    <p className="text-white/50">{position.token1.symbol}</p>
                    <p className="text-white font-medium">
                      {formatAmount(position.token1Amount, position.token1.decimals)}
                    </p>
                  </div>
                </div>

                <div className="flex gap-2">
                  <button
                    onClick={() => {
                      const pool = pools.find((p) => p.id === position.poolId);
                      if (pool) {
                        setSelectedPool(pool);
                        setShowAddModal(true);
                      }
                    }}
                    className="px-4 py-2 rounded-lg bg-white/10 text-white hover:bg-white/20 transition-colors text-sm"
                  >
                    Add
                  </button>
                  <button
                    onClick={() => {
                      setSelectedPosition(position);
                      setShowRemoveModal(true);
                    }}
                    className="px-4 py-2 rounded-lg bg-red-500/20 text-red-400 hover:bg-red-500/30 transition-colors text-sm"
                  >
                    Remove
                  </button>
                </div>
              </div>
            ))
          )}
        </div>
      )}

      {/* Add Liquidity Modal */}
      <Modal
        isOpen={showAddModal && !!selectedPool}
        onClose={() => setShowAddModal(false)}
        title="Add Liquidity"
      >
        {selectedPool && (
          <div className="space-y-4">
            {/* Token 0 */}
            <div className="bg-white/5 rounded-xl p-4">
              <div className="flex justify-between mb-2">
                <span className="text-white/50 text-sm">{selectedPool.token0.symbol}</span>
                <span className="text-white/50 text-sm">Balance: 0.00</span>
              </div>
              <input
                type="number"
                placeholder="0.00"
                value={amount0}
                onChange={(e) => setAmount0(e.target.value)}
                className="token-input w-full text-xl"
                min="0"
                step="any"
              />
            </div>

            <div className="flex justify-center">
              <span className="text-white/30 text-2xl">+</span>
            </div>

            {/* Token 1 */}
            <div className="bg-white/5 rounded-xl p-4">
              <div className="flex justify-between mb-2">
                <span className="text-white/50 text-sm">{selectedPool.token1.symbol}</span>
                <span className="text-white/50 text-sm">Balance: 0.00</span>
              </div>
              <input
                type="number"
                placeholder="0.00"
                value={amount1}
                onChange={(e) => setAmount1(e.target.value)}
                className="token-input w-full text-xl"
                min="0"
                step="any"
              />
            </div>

            <button
              onClick={handleAddLiquidity}
              disabled={!wallet.connected || !amount0 || !amount1 || addingLiquidity}
              className="btn-gold w-full disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {addingLiquidity ? (
                <span className="flex items-center justify-center gap-2">
                  <span className="animate-spin">⟳</span> Adding...
                </span>
              ) : !wallet.connected ? (
                'Connect Wallet'
              ) : (
                'Add Liquidity'
              )}
            </button>
          </div>
        )}
      </Modal>

      {/* Remove Liquidity Modal */}
      <Modal
        isOpen={showRemoveModal && !!selectedPosition}
        onClose={() => setShowRemoveModal(false)}
        title="Remove Liquidity"
      >
        {selectedPosition && (
          <div className="space-y-6">
            {/* Amount slider */}
            <div>
              <div className="flex justify-between mb-2">
                <span className="text-white/50">Amount</span>
                <span className="text-white font-bold text-2xl">{removePercent}%</span>
              </div>
              <input
                type="range"
                min="1"
                max="100"
                value={removePercent}
                onChange={(e) => setRemovePercent(parseInt(e.target.value))}
                className="w-full h-2 bg-white/10 rounded-lg appearance-none cursor-pointer accent-sultan-gold"
              />
              <div className="flex justify-between mt-2 gap-2">
                {[25, 50, 75, 100].map((p) => (
                  <button
                    key={p}
                    onClick={() => setRemovePercent(p)}
                    className={`flex-1 px-2 py-1 rounded-lg text-sm transition-colors ${
                      removePercent === p
                        ? 'bg-sultan-gold text-black'
                        : 'bg-white/10 text-white/70 hover:bg-white/20'
                    }`}
                  >
                    {p}%
                  </button>
                ))}
              </div>
            </div>

            {/* Preview */}
            <div className="bg-white/5 rounded-xl p-4 space-y-2">
              <p className="text-white/50 text-sm mb-2">You will receive</p>
              <div className="flex justify-between">
                <span className="text-white">{selectedPosition.token0.symbol}</span>
                <span className="text-white font-medium">
                  ~{formatAmount(
                    ((BigInt(selectedPosition.token0Amount) * BigInt(removePercent)) / BigInt(100)).toString(),
                    selectedPosition.token0.decimals
                  )}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-white">{selectedPosition.token1.symbol}</span>
                <span className="text-white font-medium">
                  ~{formatAmount(
                    ((BigInt(selectedPosition.token1Amount) * BigInt(removePercent)) / BigInt(100)).toString(),
                    selectedPosition.token1.decimals
                  )}
                </span>
              </div>
            </div>

            <button
              onClick={handleRemoveLiquidity}
              disabled={removingLiquidity}
              className="btn-danger w-full disabled:opacity-50"
            >
              {removingLiquidity ? (
                <span className="flex items-center justify-center gap-2">
                  <span className="animate-spin">⟳</span> Removing...
                </span>
              ) : (
                'Remove Liquidity'
              )}
            </button>
          </div>
        )}
      </Modal>

      {/* Create Pool Modal */}
      <Modal
        isOpen={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        title="Create New Pool"
      >
        <div className="space-y-4">
          <p className="text-white/60 text-sm">
            Create a new liquidity pool by providing the initial liquidity for a token pair.
          </p>

          {/* Token 0 selector */}
          <div className="bg-white/5 rounded-xl p-4">
            <p className="text-white/50 text-sm mb-2">First Token</p>
            <select
              className="w-full bg-white/10 rounded-lg p-3 text-white border border-white/10 focus:border-sultan-gold outline-none"
              onChange={(e) => {
                const token = tokens.find((t) => t.denom === e.target.value);
                setNewPoolToken0(token || null);
              }}
            >
              <option value="">Select token</option>
              {tokens.map((t) => (
                <option key={t.denom} value={t.denom}>
                  {t.symbol} - {t.name}
                </option>
              ))}
            </select>
            {newPoolToken0 && (
              <input
                type="number"
                placeholder="Amount"
                value={newPoolAmount0}
                onChange={(e) => setNewPoolAmount0(e.target.value)}
                className="token-input sm w-full mt-2"
                min="0"
                step="any"
              />
            )}
          </div>

          {/* Token 1 selector */}
          <div className="bg-white/5 rounded-xl p-4">
            <p className="text-white/50 text-sm mb-2">Second Token</p>
            <select
              className="w-full bg-white/10 rounded-lg p-3 text-white border border-white/10 focus:border-sultan-gold outline-none"
              onChange={(e) => {
                const token = tokens.find((t) => t.denom === e.target.value);
                setNewPoolToken1(token || null);
              }}
            >
              <option value="">Select token</option>
              {tokens
                .filter((t) => t.denom !== newPoolToken0?.denom)
                .map((t) => (
                  <option key={t.denom} value={t.denom}>
                    {t.symbol} - {t.name}
                  </option>
                ))}
            </select>
            {newPoolToken1 && (
              <input
                type="number"
                placeholder="Amount"
                value={newPoolAmount1}
                onChange={(e) => setNewPoolAmount1(e.target.value)}
                className="token-input sm w-full mt-2"
                min="0"
                step="any"
              />
            )}
          </div>

          <div className="bg-yellow-500/10 rounded-lg p-3 text-sm">
            <p className="text-yellow-400 flex items-center gap-2">
              ⚠️ The ratio of tokens you provide determines the initial price.
            </p>
          </div>

          <button
            disabled={!wallet.connected || !newPoolToken0 || !newPoolToken1}
            className="btn-gold w-full disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {!wallet.connected ? 'Connect Wallet' : 'Create Pool'}
          </button>
        </div>
      </Modal>
    </div>
  );
}
