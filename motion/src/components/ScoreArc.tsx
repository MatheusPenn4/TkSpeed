import React from 'react';
import { C, scoreColor, scoreLabel } from '../tokens';
import { fontMono, font } from '../fonts';

interface ScoreArcProps {
  score: number;      // 0–1000
  progress: number;   // 0–1 animation progress
  size?: number;
}

export const ScoreArc: React.FC<ScoreArcProps> = ({ score, progress, size = 280 }) => {
  const cx = size / 2;
  const cy = size / 2;
  const r = size * 0.4;
  const strokeW = size * 0.038;
  const GAP = 60; // degrees cut at bottom
  const ARC = 360 - GAP;

  const toRad = (deg: number) => (deg * Math.PI) / 180;
  const startAngle = 90 + GAP / 2;
  const arcProgress = progress * (score / 1000);

  const polarX = (angle: number, rad: number) => cx + rad * Math.cos(toRad(angle));
  const polarY = (angle: number, rad: number) => cy + rad * Math.sin(toRad(angle));

  const arcPath = (start: number, sweep: number) => {
    const end = start + sweep;
    const x1 = polarX(start, r);
    const y1 = polarY(start, r);
    const x2 = polarX(end, r);
    const y2 = polarY(end, r);
    const large = sweep > 180 ? 1 : 0;
    return `M ${x1} ${y1} A ${r} ${r} 0 ${large} 1 ${x2} ${y2}`;
  };

  const color = scoreColor(score * progress);
  const label = scoreLabel(score);
  const displayScore = Math.round(score * progress);

  return (
    <div style={{ position: 'relative', width: size, height: size }}>
      <svg width={size} height={size} style={{ overflow: 'visible' }}>
        {/* Track */}
        <path
          d={arcPath(startAngle, ARC)}
          fill="none"
          stroke={C.raised}
          strokeWidth={strokeW}
          strokeLinecap="round"
        />
        {/* Fill */}
        <path
          d={arcPath(startAngle, ARC * arcProgress)}
          fill="none"
          stroke={color}
          strokeWidth={strokeW}
          strokeLinecap="round"
          style={{ filter: `drop-shadow(0 0 ${strokeW * 0.8}px ${color})` }}
        />
        {/* Tick marks */}
        {[0, 0.2, 0.4, 0.6, 0.8, 1.0].map((t, i) => {
          const angle = startAngle + t * ARC;
          const inner = r - strokeW * 0.9;
          const outer = r - strokeW * 0.3;
          return (
            <line
              key={i}
              x1={polarX(angle, inner)}
              y1={polarY(angle, inner)}
              x2={polarX(angle, outer)}
              y2={polarY(angle, outer)}
              stroke={C.hairline}
              strokeWidth={1.5}
            />
          );
        })}
      </svg>

      {/* Center text */}
      <div style={{
        position: 'absolute', inset: 0,
        display: 'flex', flexDirection: 'column',
        alignItems: 'center', justifyContent: 'center',
        gap: 2,
      }}>
        <div style={{
          fontFamily: fontMono, fontSize: size * 0.155,
          fontWeight: 700, color: color,
          letterSpacing: '-0.03em',
          textShadow: `0 0 ${size * 0.1}px ${color}`,
          lineHeight: 1,
        }}>
          {displayScore}
        </div>
        <div style={{
          fontFamily: fontMono, fontSize: size * 0.05,
          color: C.inkLow, letterSpacing: '0.05em',
          lineHeight: 1,
        }}>
          /1000
        </div>
        <div style={{
          fontFamily: font, fontSize: size * 0.058,
          fontWeight: 600, color: color,
          letterSpacing: '0.06em', textTransform: 'uppercase',
          marginTop: 4,
          opacity: progress > 0.3 ? 1 : 0,
        }}>
          {label}
        </div>
      </div>
    </div>
  );
};
