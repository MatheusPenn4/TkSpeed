import React from 'react';
import { C } from '../tokens';
import { font } from '../fonts';

interface SignalLockMeterProps {
  confidence: number; // 0–1
  progress: number;   // animation 0–1
  label?: string;
}

const BARS = [0.4, 0.65, 0.85, 1.0, 0.75, 0.55, 0.35];

export const SignalLockMeter: React.FC<SignalLockMeterProps> = ({
  confidence,
  progress,
  label = 'Qualidade da medição',
}) => {
  const locked = confidence >= 0.7;
  const color = locked ? C.signal : C.warn;
  const statusText = locked ? `${Math.round(confidence * 100)}% estável` : 'calibrando…';

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
      <div style={{
        fontFamily: font, fontSize: 11, fontWeight: 600,
        color: C.inkLow, letterSpacing: '0.08em', textTransform: 'uppercase',
      }}>
        {label}
      </div>
      <div style={{ display: 'flex', alignItems: 'flex-end', gap: 4, height: 32 }}>
        {BARS.map((h, i) => {
          const barProgress = Math.max(0, Math.min(1, (progress - i * 0.06) / 0.4));
          const isActive = (i / BARS.length) < confidence;
          const barColor = isActive ? color : C.raised;
          return (
            <div key={i} style={{
              width: 6, borderRadius: 2,
              height: 32 * h * barProgress,
              background: barColor,
              boxShadow: isActive && barProgress > 0.8 ? `0 0 6px ${color}` : 'none',
            }} />
          );
        })}
        <div style={{
          fontFamily: font, fontSize: 12, fontWeight: 600,
          color: color, marginLeft: 8, opacity: progress,
          alignSelf: 'center',
        }}>
          {statusText}
        </div>
      </div>
    </div>
  );
};
