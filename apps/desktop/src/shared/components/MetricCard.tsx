import { Sparkline } from "./Sparkline";

type Props = {
  label: string;
  value: string;
  unit?: string;
  delta?: number;
  color?: string;
  series?: number[];
};

export function MetricCard({ label, value, unit, delta, color = "var(--primary)", series = [] }: Props) {
  return (
    <div className="glass glass-hover" style={{ padding: "var(--s-4)", display: "flex", flexDirection: "column", gap: "var(--s-2)" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <span style={{ color: "var(--text-mid)", fontSize: 13 }}>{label}</span>
        {delta !== undefined && (
          <span style={{ fontSize: 12, color: delta >= 0 ? "var(--success)" : "var(--danger)" }}>
            {delta >= 0 ? "▲" : "▼"} {Math.abs(delta)}%
          </span>
        )}
      </div>
      <div style={{ display: "flex", alignItems: "baseline", gap: 4 }}>
        <span className="num" style={{ fontSize: 30, fontWeight: 700 }}>{value}</span>
        {unit && <span style={{ color: "var(--text-low)", fontSize: 14 }}>{unit}</span>}
      </div>
      <Sparkline data={series} color={color} />
    </div>
  );
}
