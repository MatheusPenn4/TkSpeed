import { useEffect, useState } from "react";

type Props = { value: number; size?: number };

// Faixas de classificação do TkSpeed Score (ver docs/13).
function classify(v: number): { label: string; color: string } {
  if (v < 200) return { label: "Crítico", color: "var(--danger)" };
  if (v < 450) return { label: "Regular", color: "var(--warning)" };
  if (v < 700) return { label: "Bom", color: "var(--primary)" };
  if (v < 900) return { label: "Excelente", color: "var(--success)" };
  return { label: "Elite", color: "var(--success)" };
}

export function ScoreGauge({ value, size = 240 }: Props) {
  const [shown, setShown] = useState(0);
  const { label, color } = classify(value);
  const r = size / 2 - 18;
  const cx = size / 2;
  const circ = 2 * Math.PI * r;
  const arc = 0.75; // 270° gauge
  const pct = Math.min(1, value / 1000);

  // count-up
  useEffect(() => {
    let raf = 0;
    const start = performance.now();
    const dur = 900;
    const tick = (t: number) => {
      const k = Math.min(1, (t - start) / dur);
      const eased = 1 - Math.pow(1 - k, 3);
      setShown(Math.round(value * eased));
      if (k < 1) raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [value]);

  return (
    <div style={{ position: "relative", width: size, height: size }}>
      <svg width={size} height={size} style={{ transform: "rotate(135deg)" }}>
        <defs>
          <linearGradient id="gauge-grad" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#00e5ff" />
            <stop offset="100%" stopColor="#7b61ff" />
          </linearGradient>
        </defs>
        <circle cx={cx} cy={cy(cx)} r={r} fill="none" stroke="rgba(255,255,255,0.06)"
          strokeWidth={12} strokeLinecap="round"
          strokeDasharray={`${circ * arc} ${circ}`} />
        <circle cx={cx} cy={cy(cx)} r={r} fill="none" stroke="url(#gauge-grad)"
          strokeWidth={12} strokeLinecap="round"
          strokeDasharray={`${circ * arc * pct} ${circ}`}
          style={{ filter: "drop-shadow(0 0 8px rgba(0,229,255,0.5))", transition: "stroke-dasharray 0.9s var(--ease)" }} />
      </svg>
      <div style={{ position: "absolute", inset: 0, display: "grid", placeItems: "center", textAlign: "center" }}>
        <div>
          <div className="num" style={{ fontSize: 56, fontWeight: 700, lineHeight: 1 }}>{shown}</div>
          <div style={{ color: "var(--text-low)", fontSize: 13, marginTop: 4 }}>/ 1000</div>
          <div style={{ marginTop: 10, color, fontWeight: 600, letterSpacing: 0.5 }}>{label}</div>
        </div>
      </div>
    </div>
  );
}

const cy = (c: number) => c;
