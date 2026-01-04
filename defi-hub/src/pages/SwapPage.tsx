import { useState, useEffect, useMemo } from 'react';
import {
  WalletState,
  Token,
  SwapQuote,
  getTokens,
  getSwapQuote,
  executeSwap,
  formatAmount,
  parseAmount,
} from '../api/defiApi';
import { useToast, Modal, TokenIcon } from '../components';

interface SwapPageProps {
  wallet: WalletState;
}

// Token selector component using Modal
function TokenSelector({
  tokens,
  onSelect,
  onClose,
  excludeDenom,
}: {
  tokens: Token[];
  onSelect: (token: Token) => void;
  onClose: () => void;
  excludeDenom?: string;
}) {
  const [search, setSearch] = useState('');

  const filtered = useMemo(() => {
    return tokens.filter(
      (t) =>
        t.denom !== excludeDenom &&
        (t.name.toLowerCase().includes(search.toLowerCase()) ||
          t.symbol.toLowerCase().includes(search.toLowerCase()))
    );
  }, [tokens, search, excludeDenom]);

  return (
    <Modal isOpen={true} onClose={onClose} title="Select Token">
      <div className="space-y-4">
        <input
          type="text"
          placeholder="Search by name or symbol..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="token-input sm w-full"
          autoFocus
        />
        <div className="max-h-[50vh] overflow-y-auto -mx-5 px-5">
          {filtered.length === 0 ? (
            <p className="text-white/50 text-center py-8">No tokens found</p>
          ) : (
            <div className="space-y-1">
              {filtered.map((token) => (
                <button
                  key={token.denom}
                  onClick={() => {
                    onSelect(token);
                    onClose();
                  }}
                  className="w-full flex items-center gap-3 p-3 rounded-lg hover:bg-white/5 transition-colors text-left"
                >
                  <TokenIcon token={token} size="md" />
                  <div className="flex-1 min-w-0">
                    <p className="text-white font-medium">{token.symbol}</p>
                    <p className="text-white/50 text-sm truncate">{token.name}</p>
                  </div>
                  {token.description && (
                    <span className="text-white/30 text-xs hidden md:block">Native</span>
                  )}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>
    </Modal>
  );
}

export default function SwapPage({ wallet }: SwapPageProps) {
  const { showToast } = useToast();
  const [tokens, setTokens] = useState<Token[]>([]);
  const [, setLoading] = useState(true);

  // Swap state
  const [inputToken, setInputToken] = useState<Token | null>(null);
  const [outputToken, setOutputToken] = useState<Token | null>(null);
  const [inputAmount, setInputAmount] = useState('');
  const [quote, setQuote] = useState<SwapQuote | null>(null);
  const [slippage, setSlippage] = useState(0.5);
  const [showSlippageSettings, setShowSlippageSettings] = useState(false);

  // Modal state
  const [selectingFor, setSelectingFor] = useState<'input' | 'output' | null>(null);

  // Swap execution
  const [swapping, setSwapping] = useState(false);
  const [quoteLoading, setQuoteLoading] = useState(false);

  // Load tokens
  useEffect(() => {
    async function load() {
      setLoading(true);
      try {
        const tokenList = await getTokens();
        setTokens(tokenList);
        // Default to SLTN
        const sltn = tokenList.find((t) => t.symbol === 'SLTN');
        if (sltn) setInputToken(sltn);
      } catch (error) {
        console.error('Failed to load tokens:', error);
      } finally {
        setLoading(false);
      }
    }
    load();
  }, []);

  // Get quote when amounts change
  useEffect(() => {
    if (!inputToken || !outputToken || !inputAmount || parseFloat(inputAmount) <= 0) {
      setQuote(null);
      return;
    }

    const timeout = setTimeout(async () => {
      setQuoteLoading(true);
      try {
        const q = await getSwapQuote(
          inputToken.denom,
          outputToken.denom,
          parseAmount(inputAmount, inputToken.decimals),
          slippage
        );
        setQuote(q);
      } catch (error) {
        console.error('Failed to get quote:', error);
        setQuote(null);
      } finally {
        setQuoteLoading(false);
      }
    }, 300);

    return () => clearTimeout(timeout);
  }, [inputToken, outputToken, inputAmount, slippage]);

  // Flip tokens
  const flipTokens = () => {
    const temp = inputToken;
    setInputToken(outputToken);
    setOutputToken(temp);
    setInputAmount('');
    setQuote(null);
  };

  // Execute swap
  const handleSwap = async () => {
    if (!wallet.connected || !inputToken || !outputToken || !quote) return;

    setSwapping(true);
    try {
      const result = await executeSwap(
        {
          user: wallet.address!,
          inputDenom: inputToken.denom,
          outputDenom: outputToken.denom,
          inputAmount: parseAmount(inputAmount, inputToken.decimals),
          minOutput: quote.minOutput,
        },
        'signature_placeholder'
      );
      showToast(`Swapped ${inputAmount} ${inputToken.symbol} for ${formatAmount(result.outputAmount, outputToken.decimals)} ${outputToken.symbol}`, 'success');
      setInputAmount('');
      setQuote(null);
    } catch (error) {
      console.error('Swap failed:', error);
      showToast('Swap failed. Please try again.', 'error');
    } finally {
      setSwapping(false);
    }
  };

  return (
    <div className="max-w-lg mx-auto">
      {/* Page Header */}
      <div className="text-center mb-8">
        <h2 className="text-3xl font-bold text-white mb-2">Swap</h2>
        <p className="text-white/60">Trade tokens instantly with zero gas fees</p>
      </div>

      {/* Swap Card */}
      <div className="glass-card">
        {/* Settings */}
        <div className="flex justify-between items-center mb-4">
          <span className="text-white/50 text-sm">Swap</span>
          <button
            onClick={() => setShowSlippageSettings(!showSlippageSettings)}
            className="text-white/50 hover:text-white transition-colors"
          >
            ⚙️ {slippage}%
          </button>
        </div>

        {/* Slippage Settings */}
        {showSlippageSettings && (
          <div className="mb-4 p-3 rounded-lg bg-white/5">
            <p className="text-white/70 text-sm mb-2">Slippage Tolerance</p>
            <div className="flex gap-2">
              {[0.1, 0.5, 1.0, 3.0].map((s) => (
                <button
                  key={s}
                  onClick={() => setSlippage(s)}
                  className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
                    slippage === s
                      ? 'bg-sultan-gold text-black'
                      : 'bg-white/10 text-white hover:bg-white/20'
                  }`}
                >
                  {s}%
                </button>
              ))}
              <input
                type="number"
                value={slippage}
                onChange={(e) => setSlippage(parseFloat(e.target.value) || 0.5)}
                className="token-input w-20 text-center text-sm"
                min="0.01"
                max="50"
                step="0.1"
              />
            </div>
          </div>
        )}

        {/* Input Token */}
        <div className="bg-white/5 rounded-xl p-4 mb-2">
          <div className="flex justify-between mb-2">
            <span className="text-white/50 text-sm">You Pay</span>
            <span className="text-white/50 text-sm">Balance: 0.00</span>
          </div>
          <div className="flex gap-3">
            <input
              type="number"
              placeholder="0.00"
              value={inputAmount}
              onChange={(e) => setInputAmount(e.target.value)}
              className="token-input flex-1 text-2xl"
              min="0"
              step="any"
            />
            <button
              onClick={() => setSelectingFor('input')}
              className="flex items-center gap-2 bg-white/10 hover:bg-white/20 rounded-xl px-4 py-2 transition-colors min-w-[120px]"
            >
              {inputToken ? (
                <>
                  <TokenIcon token={inputToken} size="sm" />
                  <span className="text-white font-medium">{inputToken.symbol}</span>
                </>
              ) : (
                <span className="text-white">Select</span>
              )}
              <span className="text-white/50 ml-auto">▼</span>
            </button>
          </div>
        </div>

        {/* Swap Arrow */}
        <div className="flex justify-center -my-2 relative z-10">
          <button
            onClick={flipTokens}
            className="w-10 h-10 rounded-full bg-sultan-dark border-4 border-[#1a1f2e] flex items-center justify-center text-sultan-gold hover:rotate-180 transition-transform duration-300"
            aria-label="Swap tokens"
          >
            ↓
          </button>
        </div>

        {/* Output Token */}
        <div className="bg-white/5 rounded-xl p-4 mt-2">
          <div className="flex justify-between mb-2">
            <span className="text-white/50 text-sm">You Receive</span>
            <span className="text-white/50 text-sm">Balance: 0.00</span>
          </div>
          <div className="flex gap-3">
            <input
              type="text"
              placeholder="0.00"
              value={
                quoteLoading
                  ? 'Loading...'
                  : quote
                  ? formatAmount(quote.outputAmount, outputToken?.decimals)
                  : ''
              }
              readOnly
              className="token-input flex-1 text-2xl bg-transparent"
            />
            <button
              onClick={() => setSelectingFor('output')}
              className="flex items-center gap-2 bg-white/10 hover:bg-white/20 rounded-xl px-4 py-2 transition-colors min-w-[120px]"
            >
              {outputToken ? (
                <>
                  <TokenIcon token={outputToken} size="sm" />
                  <span className="text-white font-medium">{outputToken.symbol}</span>
                </>
              ) : (
                <span className="text-white">Select</span>
              )}
              <span className="text-white/50 ml-auto">▼</span>
            </button>
          </div>
        </div>

        {/* Quote Details */}
        {quote && (
          <div className="mt-4 p-3 rounded-lg bg-white/5 space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-white/50">Price Impact</span>
              <span className={quote.priceImpact > 3 ? 'text-red-400' : 'text-white'}>
                {quote.priceImpact.toFixed(2)}%
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-white/50">Minimum Received</span>
              <span className="text-white">
                {formatAmount(quote.minOutput, outputToken?.decimals)} {outputToken?.symbol}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-white/50">Fee</span>
              <span className="text-white">0 SLTN (Zero Gas)</span>
            </div>
          </div>
        )}

        {/* Swap Button */}
        <button
          onClick={handleSwap}
          disabled={!wallet.connected || !inputToken || !outputToken || !inputAmount || swapping}
          className="btn-gold w-full mt-4 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {swapping ? (
            <span className="flex items-center justify-center gap-2">
              <span className="animate-spin">⟳</span> Swapping...
            </span>
          ) : !wallet.connected ? (
            'Connect Wallet to Swap'
          ) : !inputToken || !outputToken ? (
            'Select Tokens'
          ) : !inputAmount ? (
            'Enter Amount'
          ) : (
            'Swap'
          )}
        </button>
      </div>

      {/* Features */}
      <div className="grid grid-cols-3 gap-4 mt-8">
        <div className="glass-card text-center">
          <div className="text-2xl mb-2">⚡</div>
          <p className="text-white font-medium text-sm">Instant</p>
          <p className="text-white/50 text-xs">~1s Finality</p>
        </div>
        <div className="glass-card text-center">
          <div className="text-2xl mb-2">💰</div>
          <p className="text-white font-medium text-sm">Zero Gas</p>
          <p className="text-white/50 text-xs">No Tx Fees</p>
        </div>
        <div className="glass-card text-center">
          <div className="text-2xl mb-2">🔒</div>
          <p className="text-white font-medium text-sm">Secure</p>
          <p className="text-white/50 text-xs">Native DEX</p>
        </div>
      </div>

      {/* Token Selector Modal */}
      {selectingFor && (
        <TokenSelector
          tokens={tokens}
          onSelect={(token) => {
            if (selectingFor === 'input') {
              setInputToken(token);
            } else {
              setOutputToken(token);
            }
          }}
          onClose={() => setSelectingFor(null)}
          excludeDenom={selectingFor === 'input' ? outputToken?.denom : inputToken?.denom}
        />
      )}
    </div>
  );
}
