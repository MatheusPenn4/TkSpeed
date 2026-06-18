import React from 'react';
import { C } from '../tokens';
import { font, fontMono } from '../fonts';

interface MetricBarProps {
  label: string;
  value: number;   // 0–100
  unit?: string;
  color?: string;
  progress: number; // animation 0–1
  critical?: boolean;
}

export const MetricBar: React.FC<MetricBarProps> = ({
  label, value, unit = '%', color, progress, critical = false,
}) => {
  const barColor = color ?? (critical ? C.risk : value > 80 ? C.warn : C.ion);
  const fillWidth = value * progress;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <span style={{
          fontFamily: font, fontSize: 12, fontWeight: 500, color: C.inkMid,
        }}>{label}</span>
        <span style={{
          fontFamily: fontMono, fontSize: 13, fontWeight: 700,
          color: barColor,
        }}>
          {Math.round(value * progress)}{unit}
        </span>
      </div>
      <div style={{
        height: 4, background: C.raised, borderRadius: 2, overflow: 'hidden',
      }}>
        <div style={{
          height: '100%', width: `${fillWidth}%`,
          background: barColor,
          borderRadius: 2,
          boxShadow: `0 0 8px ${barColor}`,
        }} />
      </div>
    </div>
  );
};
