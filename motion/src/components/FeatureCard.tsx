import React from 'react';
import { C } from '../tokens';
import { font } from '../fonts';

interface FeatureCardProps {
  icon: string;       // emoji or unicode symbol
  title: string;
  desc: string;
  accentColor?: string;
  progress: number;   // 0–1 for entrance
  delay?: number;     // 0–1 stagger offset applied externally
}

export const FeatureCard: React.FC<FeatureCardProps> = ({
  icon, title, desc, accentColor = C.signal, progress,
}) => {
  const p = Math.max(0, Math.min(1, progress));
  const translateY = (1 - p) * 28;

  return (
    <div style={{
      background: C.panel,
      border: `1px solid ${C.hairline}`,
      borderRadius: 12,
      padding: '20px 22px',
      opacity: p,
      transform: `translateY(${translateY}px)`,
      position: 'relative',
      overflow: 'hidden',
    }}>
      {/* Top accent line */}
      <div style={{
        position: 'absolute', top: 0, left: 0, right: 0,
        height: 2,
        background: `linear-gradient(90deg, ${accentColor} 0%, transparent 100%)`,
        opacity: p,
      }} />

      {/* Icon */}
      <div style={{
        fontSize: 28, marginBottom: 12, lineHeight: 1,
      }}>{icon}</div>

      {/* Title */}
      <div style={{
        fontFamily: font, fontSize: 15, fontWeight: 700,
        color: C.inkHi, marginBottom: 6, lineHeight: 1.2,
      }}>{title}</div>

      {/* Desc */}
      <div style={{
        fontFamily: font, fontSize: 12.5, color: C.inkMid,
        lineHeight: 1.5,
      }}>{desc}</div>
    </div>
  );
};
