import React from 'react';
import { C } from '../tokens';
import { font, fontMono } from '../fonts';

interface EvidenceCardProps {
  metric: string;
  before: number[];
  after: number[];
  unit: string;
  delta: number;     // e.g. +12.5
  progress: number;  // 0–1
  width?: number;
  height?: number;
}

const Sparkline: React.FC<{
  data: number[];
  color: string;
  w: number;
  h: number;
  progress: number;
}> = ({ data, color, w, h, progress }) => {
  const min = Math.min(...data);
  const max = Math.max(...data);
  const range = max - min || 1;
  const pts = data.map((v, i) => {
    const x = (i / (data.length - 1)) * w;
    const y = h - ((v - min) / range) * (h * 0.8) - h * 0.1;
    return `${x},${y}`;
  });
  // Trim path by progress
  const totalPts = Math.max(2, Math.ceil(progress * data.length));
  const visiblePts = pts.slice(0, totalPts).join(' ');

  return (
    <svg width={w} height={h} style={{ overflow: 'visible' }}>
      <polyline
        points={visiblePts}
        fill="none"
        stroke={color}
        strokeWidth={2}
        strokeLinecap="round"
        strokeLinejoin="round"
        style={{ filter: `drop-shadow(0 0 4px ${color})` }}
      />
    </svg>
  );
};

export const EvidenceCard: React.FC<EvidenceCardProps> = ({
  metric, before, after, unit, delta, progress,
  width = 360, height = 160,
}) => {
  const verdictColor = delta > 1 ? C.ok : delta < -1 ? C.risk : C.warn;
  const verdictText = delta > 1 ? `Ganho ▲ +${delta.toFixed(1)}%` : delta < -1 ? `Perda ▼ ${delta.toFixed(1)}%` : `Sem mudança`;

  const sparkW = (width - 48) / 2 - 8;
  const sparkH = 52;
  const cardProgress = Math.min(1, progress * 1.2);

  return (
    <div style={{
      background: C.panel,
      border: `1px solid ${C.hairline}`,
      borderRadius: 10,
      padding: '14px 16px',
      width, minHeight: height,
      opacity: cardProgress,
    }}>
      {/* Header */}
      <div style={{
        fontFamily: font, fontSize: 11, fontWeight: 600,
        color: C.inkLow, letterSpacing: '0.08em',
        textTransform: 'uppercase', marginBottom: 10,
      }}>
        {metric}
      </div>

      {/* Sparklines side by side */}
      <div style={{ display: 'flex', gap: 8, marginBottom: 10 }}>
        <div style={{ flex: 1 }}>
          <div style={{
            fontFamily: fontMono, fontSize: 10, color: C.inkLow, marginBottom: 4,
          }}>Antes</div>
          <Sparkline data={before} color={C.inkLow} w={sparkW} h={sparkH} progress={Math.min(1, progress * 2)} />
        </div>
        <div style={{
          width: 1, background: C.hairline, alignSelf: 'stretch',
        }} />
        <div style={{ flex: 1 }}>
          <div style={{
            fontFamily: fontMono, fontSize: 10, color: C.ion, marginBottom: 4,
          }}>Depois</div>
          <Sparkline data={after} color={C.ion} w={sparkW} h={sparkH} progress={Math.max(0, progress * 2 - 0.8)} />
        </div>
      </div>

      {/* Verdict */}
      <div style={{
        display: 'flex', justifyContent: 'space-between', alignItems: 'center',
        opacity: progress > 0.7 ? (progress - 0.7) / 0.3 : 0,
      }}>
        <div style={{
          fontFamily: fontMono, fontSize: 13, fontWeight: 700, color: verdictColor,
        }}>
          {verdictText}
        </div>
        <div style={{
          fontFamily: fontMono, fontSize: 11, color: C.inkLow,
        }}>
          confiança 94%
        </div>
      </div>
    </div>
  );
};
