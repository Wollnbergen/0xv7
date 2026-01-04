import { Token } from '../api/defiApi';

interface TokenIconProps {
  token: Token | null;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

const sizeClasses = {
  sm: 'w-6 h-6 text-sm',
  md: 'w-10 h-10 text-base',
  lg: 'w-12 h-12 text-xl',
};

export default function TokenIcon({ token, size = 'md', className = '' }: TokenIconProps) {
  if (!token) {
    return (
      <div
        className={`${sizeClasses[size]} rounded-full bg-white/10 flex items-center justify-center text-white/30 ${className}`}
      >
        ?
      </div>
    );
  }

  // If token has a logo, display it
  if (token.logo) {
    return (
      <img
        src={token.logo}
        alt={token.symbol}
        className={`${sizeClasses[size]} rounded-full object-cover ${className}`}
        onError={(e) => {
          // Fallback to letter if image fails
          e.currentTarget.style.display = 'none';
          e.currentTarget.nextElementSibling?.classList.remove('hidden');
        }}
      />
    );
  }

  // Generate color from symbol for consistency
  const colors = [
    'bg-blue-500/20 text-blue-400',
    'bg-green-500/20 text-green-400',
    'bg-purple-500/20 text-purple-400',
    'bg-pink-500/20 text-pink-400',
    'bg-yellow-500/20 text-yellow-400',
    'bg-red-500/20 text-red-400',
    'bg-cyan-500/20 text-cyan-400',
    'bg-orange-500/20 text-orange-400',
  ];

  // Special case for known tokens
  const specialColors: Record<string, string> = {
    SLTN: 'bg-sultan-gold/20 text-sultan-gold',
    BTC: 'bg-orange-500/20 text-orange-400',
    WBTC: 'bg-orange-500/20 text-orange-400',
    ETH: 'bg-blue-400/20 text-blue-300',
    WETH: 'bg-blue-400/20 text-blue-300',
    USDC: 'bg-blue-500/20 text-blue-400',
    USDT: 'bg-green-500/20 text-green-400',
  };

  const colorClass =
    specialColors[token.symbol] || colors[token.symbol.charCodeAt(0) % colors.length];

  return (
    <div
      className={`${sizeClasses[size]} rounded-full ${colorClass} flex items-center justify-center font-bold ${className}`}
    >
      {token.symbol.charAt(0)}
    </div>
  );
}

// Dual token icon for pools
interface TokenPairIconProps {
  token0: Token;
  token1: Token;
  size?: 'sm' | 'md' | 'lg';
}

export function TokenPairIcon({ token0, token1, size = 'md' }: TokenPairIconProps) {
  return (
    <div className="flex -space-x-2">
      <TokenIcon token={token0} size={size} className="border-2 border-sultan-dark" />
      <TokenIcon token={token1} size={size} className="border-2 border-sultan-dark" />
    </div>
  );
}
