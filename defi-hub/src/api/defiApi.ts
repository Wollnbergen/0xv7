/**
 * Sultan DeFi Hub - API Client
 * Connects to Sultan L1 Native DEX and Token Factory
 */

// Configuration - can be overridden via environment
export const config = {
  RPC_URL: import.meta.env.VITE_RPC_URL || 'https://rpc.sltn.io',
  API_TIMEOUT: 30000,
  MAX_RETRIES: 3,
  RETRY_DELAY: 1000,
};

// ============================================================================
// Error Handling
// ============================================================================

export class ApiError extends Error {
  constructor(
    message: string,
    public readonly status?: number,
    public readonly code?: string
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

// ============================================================================
// Types
// ============================================================================

export interface Token {
  denom: string;
  name: string;
  symbol: string;
  decimals: number;
  totalSupply: string;
  logo?: string;
  description?: string;
}

export interface Pool {
  id: string;
  token0: Token;
  token1: Token;
  reserve0: string;
  reserve1: string;
  lpToken: string;
  totalLpSupply: string;
  feeRate: number; // basis points (30 = 0.3%)
  volume24h?: string;
  tvl?: string;
}

export interface SwapQuote {
  inputAmount: string;
  outputAmount: string;
  priceImpact: number;
  fee: string;
  route: string[];
  minOutput: string; // with slippage
}

export interface Position {
  poolId: string;
  lpTokens: string;
  share: number; // percentage
  token0Amount: string;
  token1Amount: string;
  token0: Token;
  token1: Token;
}

export interface LaunchpadProject {
  id: string;
  token: Token;
  creator: string;
  status: 'created' | 'presale' | 'launched' | 'completed';
  createdAt: number;
  // Future smart contract fields (Coming Soon)
  presalePrice?: string;
  presaleStart?: number;
  presaleEnd?: number;
  hardCap?: string;
  softCap?: string;
  raised?: string;
  vestingSchedule?: VestingSchedule;
}

export interface VestingSchedule {
  cliff: number; // seconds
  duration: number; // seconds
  initialUnlock: number; // percentage
}

// ============================================================================
// Feature Flags - These unlock when smart contracts are ready
// ============================================================================

export const FEATURES = {
  SWAP: true,
  LIQUIDITY: true,
  CREATE_TOKEN: true,
  // Coming with smart contracts
  PRESALE: false,
  VESTING: false,
  WHITELIST: false,
  FAIR_LAUNCH: false,
  STAKING_POOLS: false,
};

// ============================================================================
// API Helpers
// ============================================================================

async function fetchApi<T>(
  endpoint: string,
  options?: RequestInit,
  retries = config.MAX_RETRIES
): Promise<T> {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), config.API_TIMEOUT);

  try {
    const response = await fetch(`${config.RPC_URL}${endpoint}`, {
      ...options,
      signal: controller.signal,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
    });

    if (!response.ok) {
      const errorBody = await response.text().catch(() => '');
      throw new ApiError(
        `API error: ${response.status}`,
        response.status,
        errorBody
      );
    }

    return response.json();
  } catch (error) {
    if (error instanceof ApiError) throw error;
    
    // Retry on network errors
    if (retries > 0 && error instanceof Error && error.name !== 'AbortError') {
      await new Promise((r) => setTimeout(r, config.RETRY_DELAY));
      return fetchApi<T>(endpoint, options, retries - 1);
    }
    
    throw new ApiError(
      error instanceof Error ? error.message : 'Network error',
      undefined,
      'NETWORK_ERROR'
    );
  } finally {
    clearTimeout(timeout);
  }
}

// ============================================================================
// Token Factory API
// ============================================================================

// Demo tokens for development/testing
const DEMO_TOKENS: Token[] = [
  { denom: 'usltn', name: 'Sultan', symbol: 'SLTN', decimals: 9, totalSupply: '500000000000000000', description: 'Native token of Sultan L1' },
  { denom: 'uusdc', name: 'USD Coin', symbol: 'USDC', decimals: 6, totalSupply: '1000000000000000' },
  { denom: 'uweth', name: 'Wrapped Ether', symbol: 'WETH', decimals: 18, totalSupply: '100000000000000000000000' },
  { denom: 'uwbtc', name: 'Wrapped Bitcoin', symbol: 'WBTC', decimals: 8, totalSupply: '2100000000000000' },
];

export async function getTokens(): Promise<Token[]> {
  try {
    const result = await fetchApi<{ tokens: Token[] }>('/tokens');
    return result.tokens || [];
  } catch {
    // Return demo tokens for development
    return DEMO_TOKENS;
  }
}

export async function getToken(denom: string): Promise<Token | null> {
  try {
    return await fetchApi<Token>(`/tokens/${denom}`);
  } catch {
    return null;
  }
}

export interface CreateTokenParams {
  name: string;
  symbol: string;
  decimals: number;
  totalSupply: string;
  creator: string;
  logo?: string;
  description?: string;
}

export async function createToken(params: CreateTokenParams, signature: string): Promise<{ denom: string; txHash: string }> {
  return fetchApi('/token-factory/create', {
    method: 'POST',
    body: JSON.stringify({ ...params, signature }),
  });
}

// ============================================================================
// DEX API - Swap
// ============================================================================

export async function getPools(): Promise<Pool[]> {
  try {
    const result = await fetchApi<{ pools: Pool[] }>('/dex/pools');
    return result.pools || [];
  } catch {
    return [];
  }
}

export async function getPool(poolId: string): Promise<Pool | null> {
  try {
    return await fetchApi<Pool>(`/dex/pools/${poolId}`);
  } catch {
    return null;
  }
}

export async function getSwapQuote(
  inputDenom: string,
  outputDenom: string,
  inputAmount: string,
  slippage: number = 0.5
): Promise<SwapQuote> {
  const result = await fetchApi<{
    output_amount: number;
    price_impact: number;
    fee: number;
  }>(`/dex/quote?input=${inputDenom}&output=${outputDenom}&amount=${inputAmount}`);

  const minOutput = Math.floor(result.output_amount * (1 - slippage / 100));

  return {
    inputAmount,
    outputAmount: result.output_amount.toString(),
    priceImpact: result.price_impact,
    fee: result.fee.toString(),
    route: [inputDenom, outputDenom],
    minOutput: minOutput.toString(),
  };
}

export interface SwapParams {
  user: string;
  inputDenom: string;
  outputDenom: string;
  inputAmount: string;
  minOutput: string;
}

export async function executeSwap(params: SwapParams, signature: string): Promise<{ txHash: string; outputAmount: string }> {
  return fetchApi('/dex/swap', {
    method: 'POST',
    body: JSON.stringify({ ...params, signature }),
  });
}

// ============================================================================
// DEX API - Liquidity
// ============================================================================

export async function getUserPositions(address: string): Promise<Position[]> {
  try {
    const result = await fetchApi<{ positions: Position[] }>(`/dex/positions/${address}`);
    return result.positions || [];
  } catch {
    return [];
  }
}

export interface AddLiquidityParams {
  user: string;
  token0: string;
  token1: string;
  amount0: string;
  amount1: string;
  slippage: number;
}

export async function addLiquidity(params: AddLiquidityParams, signature: string): Promise<{ txHash: string; lpTokens: string }> {
  return fetchApi('/dex/add-liquidity', {
    method: 'POST',
    body: JSON.stringify({ ...params, signature }),
  });
}

export interface RemoveLiquidityParams {
  user: string;
  poolId: string;
  lpTokens: string;
  minAmount0: string;
  minAmount1: string;
}

export async function removeLiquidity(params: RemoveLiquidityParams, signature: string): Promise<{ txHash: string; amount0: string; amount1: string }> {
  return fetchApi('/dex/remove-liquidity', {
    method: 'POST',
    body: JSON.stringify({ ...params, signature }),
  });
}

// ============================================================================
// Launchpad API (Token Factory + Future Smart Contracts)
// ============================================================================

export async function getLaunchpadProjects(): Promise<LaunchpadProject[]> {
  try {
    const result = await fetchApi<{ projects: LaunchpadProject[] }>('/launchpad/projects');
    return result.projects || [];
  } catch {
    return [];
  }
}

export async function getLaunchpadProject(id: string): Promise<LaunchpadProject | null> {
  try {
    return await fetchApi<LaunchpadProject>(`/launchpad/projects/${id}`);
  } catch {
    return null;
  }
}

// ============================================================================
// Wallet Connection (stub - integrates with Sultan Wallet)
// ============================================================================

export interface WalletState {
  connected: boolean;
  address: string | null;
  balance: string;
}

export function connectWallet(): Promise<WalletState> {
  // This will integrate with Sultan Wallet extension/PWA
  // For now, return mock for UI development
  return Promise.resolve({
    connected: false,
    address: null,
    balance: '0',
  });
}

// ============================================================================
// Utility Functions
// ============================================================================

export function formatAmount(amount: string, decimals: number = 9): string {
  const value = Number(amount) / Math.pow(10, decimals);
  if (value >= 1000000) return `${(value / 1000000).toFixed(2)}M`;
  if (value >= 1000) return `${(value / 1000).toFixed(2)}K`;
  return value.toFixed(value < 1 ? 6 : 2);
}

export function parseAmount(amount: string, decimals: number = 9): string {
  const value = parseFloat(amount);
  if (isNaN(value)) return '0';
  return Math.floor(value * Math.pow(10, decimals)).toString();
}
