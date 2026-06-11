import type { ReactNode } from "react";

/** Metric Card — numeral mono tabular (assinatura instrumental). ion=true usa a cor de dados. */
export function AxMetric({
  label,
  value,
  unit,
  sub,
  ion = false,
}: {
  label: string;
  value: ReactNode;
  unit?: string;
  sub?: ReactNode;
  ion?: boolean;
}) {
  return (
    <div className={`ax-metric${ion ? " ax-metric-ion" : ""}`}>
      <span className="ax-metric-k">{label}</span>
      <span className="ax-metric-v">
        {value}
        {unit && <span className="ax-unit"> {unit}</span>}
      </span>
      {sub && <span className="ax-metric-sub">{sub}</span>}
    </div>
  );
}
