import { BrandSymbol } from "./BrandSymbol";

/**
 * Lockup oficial do produto: [símbolo] TkSpeed / Performance Engineering.
 * Usado no rail e em cabeçalhos — substitui qualquer "logo de dashboard comum".
 */
export function BrandHeader({ symbolSize = 26, compact = false }: { symbolSize?: number; compact?: boolean }) {
  return (
    <div className="ax-brand-header">
      <BrandSymbol size={symbolSize} />
      {!compact && (
        <div className="ax-brand-text">
          <b>TkSpeed</b>
          <span>Performance Engineering</span>
        </div>
      )}
    </div>
  );
}
