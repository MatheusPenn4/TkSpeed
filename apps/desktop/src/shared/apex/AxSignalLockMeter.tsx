/**
 * Signal-Lock Meter (assinatura) — visualiza o Confidence Engine como qualidade
 * de sinal de um instrumento. Estável → barras mint ("lock"); instável → âmbar.
 * Reforça o princípio Medir→Provar: só há "lock" quando a medição é confiável.
 */
export function AxSignalLockMeter({
  confidence,
  stable = true,
  label,
}: {
  confidence: number; // 0–100
  stable?: boolean;
  label?: string;
}) {
  const total = 5;
  const lit = Math.max(0, Math.min(total, Math.round((confidence / 100) * total)));
  const heights = [8, 11, 14, 16, 18];
  const state = stable ? "locked" : "unstable";
  return (
    <div className={`ax-lock ${state}`} title={`Confiança da medição: ${confidence}%`}>
      <span className="ax-lock-bars">
        {heights.map((h, i) => (
          <span key={i} className={`ax-lock-bar${i < lit ? " on" : ""}`} style={{ height: h }} />
        ))}
      </span>
      <span className="ax-lock-label">{label ?? (stable ? `${confidence}% estável` : `${confidence}% instável`)}</span>
    </div>
  );
}
