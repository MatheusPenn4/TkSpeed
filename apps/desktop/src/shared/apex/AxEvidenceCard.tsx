import { AxBadge, type AxBadgeVariant } from "./AxBadge";
import { AxSignalLockMeter } from "./AxSignalLockMeter";

export type AxVerdict = "Gain" | "Loss" | "NoChange" | "Unstable";

const VERDICT: Record<AxVerdict, { label: string; variant: AxBadgeVariant }> = {
  Gain: { label: "Ganho", variant: "ok" },
  Loss: { label: "Perda", variant: "risk" },
  NoChange: { label: "Sem mudança", variant: "neutral" },
  Unstable: { label: "Instável", variant: "warn" },
};

/** Constrói um path SVG (sparkline) a partir de uma série, normalizada no viewport. */
function pathFrom(series: number[], w = 120, h = 44): string {
  if (series.length < 2) return "";
  const min = Math.min(...series);
  const max = Math.max(...series);
  const range = max - min || 1;
  return series
    .map((v, i) => {
      const x = (i / (series.length - 1)) * w;
      const y = h - ((v - min) / range) * h;
      return `${i === 0 ? "M" : "L"}${x.toFixed(1)} ${y.toFixed(1)}`;
    })
    .join(" ");
}

/**
 * Evidence Card (assinatura) — o componente que materializa "Provar": traços
 * antes→depois sobrepostos + veredito honesto + confiança. Recebe SÉRIES REAIS
 * (nunca dado fake).
 */
export function AxEvidenceCard({
  before,
  after,
  verdict,
  confidence,
  deltaPct,
  reliable = true,
}: {
  before: number[];
  after: number[];
  verdict: AxVerdict;
  confidence: number;
  deltaPct?: number;
  reliable?: boolean;
}) {
  const v = VERDICT[verdict];
  const delta =
    deltaPct === undefined ? null : `${deltaPct >= 0 ? "+" : ""}${deltaPct.toFixed(2)}%`;
  return (
    <div className="ax-evidence">
      <div className="ax-evidence-head">
        <span className="ax-label">Evidência · antes → depois</span>
        <AxSignalLockMeter confidence={confidence} stable={reliable} />
      </div>
      <div className="ax-evidence-traces">
        <div className="ax-trace ax-trace-before">
          <span className="ax-trace-k">Antes</span>
          <svg className="ax-trace-svg" viewBox="0 0 120 44" preserveAspectRatio="none">
            <path d={pathFrom(before)} fill="none" strokeWidth={1.5} strokeLinecap="round" strokeLinejoin="round" />
          </svg>
        </div>
        <div className="ax-trace ax-trace-after">
          <span className="ax-trace-k">Depois</span>
          <svg className="ax-trace-svg" viewBox="0 0 120 44" preserveAspectRatio="none">
            <path d={pathFrom(after)} fill="none" strokeWidth={1.5} strokeLinecap="round" strokeLinejoin="round" />
          </svg>
        </div>
      </div>
      <div className="ax-evidence-verdict">
        <AxBadge variant={v.variant}>{v.label}</AxBadge>
        {delta && <span className="ax-data" style={{ color: "var(--ink-mid)" }}>{delta}</span>}
      </div>
    </div>
  );
}
