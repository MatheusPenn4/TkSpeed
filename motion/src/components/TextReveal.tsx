import React from 'react';

interface TextRevealProps {
  text: string;
  progress: number;  // 0–1
  style?: React.CSSProperties;
}

export const TextReveal: React.FC<TextRevealProps> = ({ text, progress, style }) => {
  const chars = text.split('');
  return (
    <span style={{ display: 'inline-block', ...style }}>
      {chars.map((ch, i) => {
        const t = (progress - (i / chars.length) * 0.5) / 0.5;
        const p = Math.max(0, Math.min(1, t));
        return (
          <span key={i} style={{
            opacity: p,
            display: 'inline-block',
            transform: `translateY(${(1 - p) * 8}px)`,
          }}>
            {ch === ' ' ? ' ' : ch}
          </span>
        );
      })}
    </span>
  );
};
