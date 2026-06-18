import React from 'react';
import {
  AbsoluteFill, useCurrentFrame, useVideoConfig,
  interpolate, Easing,
} from 'remotion';
import { C, SETTLE } from '../tokens';
import { font, fontMono } from '../fonts';
import { Background } from '../components/Background';

const ISSUES = [
  { icon: '⚡', label: 'CPU', value: '92%', note: 'sobrecarregada', color: C.risk },
  { icon: '🧠', label: 'RAM', value: '87%', note: 'pressão elevada', color: C.warn },
  { icon: '💿', label: 'Disco', value: '94%', note: 'lotado', color: C.warn },
  { icon: '🌡', label: 'Temp', value: '91°C', note: 'throttling ativo', color: C.risk },
];

// Scene: 240 frames = 8s
export const Scene02Problem: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const fadeIn = interpolate(frame, [0, fps * 0.5], [0, 1], { extrapolateRight: 'clamp' });

  const headlineY = interpolate(frame, [fps * 0.1, fps * 0.9], [24, 0], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  const subY = interpolate(frame, [fps * 0.5, fps * 1.3], [16, 0], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  const subOpacity = interpolate(frame, [fps * 0.5, fps * 1.1], [0, 1], { extrapolateRight: 'clamp' });

  const fadeOut = interpolate(frame, [fps * 7, fps * 8], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  return (
    <AbsoluteFill style={{ opacity: fadeIn * fadeOut }}>
      <Background />

      <AbsoluteFill style={{
        display: 'flex', flexDirection: 'column',
        alignItems: 'center', justifyContent: 'center',
        padding: '0 120px', gap: 0,
      }}>
        {/* Headline */}
        <div style={{
          transform: `translateY(${headlineY}px)`,
          textAlign: 'center', marginBottom: 16,
        }}>
          <div style={{
            fontFamily: font, fontSize: 52, fontWeight: 700,
            color: C.inkHi, lineHeight: 1.1, letterSpacing: '-0.025em',
          }}>
            Seu PC está mais lento
          </div>
          <div style={{
            fontFamily: font, fontSize: 52, fontWeight: 700,
            color: C.risk, lineHeight: 1.1, letterSpacing: '-0.025em',
          }}>
            do que deveria ser.
          </div>
        </div>

        {/* Sub */}
        <div style={{
          transform: `translateY(${subY}px)`,
          opacity: subOpacity,
          fontFamily: font, fontSize: 20, color: C.inkMid,
          textAlign: 'center', lineHeight: 1.5, marginBottom: 52,
          maxWidth: 640,
        }}>
          Gargalos silenciosos consomem a performance que você pagou.
          A maioria dos usuários nem sabe onde estão os problemas.
        </div>

        {/* Issue cards */}
        <div style={{ display: 'flex', gap: 20 }}>
          {ISSUES.map((issue, i) => {
            const cardProgress = interpolate(
              frame,
              [fps * (0.8 + i * 0.18), fps * (1.4 + i * 0.18)],
              [0, 1],
              { extrapolateRight: 'clamp', easing: Easing.bezier(...SETTLE) }
            );
            const barPulse = interpolate(
              frame % (fps * 1.2),
              [0, fps * 0.6, fps * 1.2],
              [0.7, 1.0, 0.7],
              { extrapolateRight: 'clamp' }
            );

            return (
              <div key={i} style={{
                background: C.panel,
                border: `1px solid ${issue.color}33`,
                borderRadius: 12,
                padding: '20px 24px',
                width: 180,
                opacity: cardProgress,
                transform: `translateY(${(1 - cardProgress) * 20}px)`,
                display: 'flex', flexDirection: 'column', gap: 8,
                position: 'relative', overflow: 'hidden',
              }}>
                {/* Top bar */}
                <div style={{
                  position: 'absolute', top: 0, left: 0, right: 0,
                  height: 2, background: issue.color,
                  boxShadow: `0 0 8px ${issue.color}`,
                  opacity: barPulse,
                }} />
                <div style={{ fontSize: 24 }}>{issue.icon}</div>
                <div style={{
                  fontFamily: font, fontSize: 12, fontWeight: 600,
                  color: C.inkLow, letterSpacing: '0.08em',
                  textTransform: 'uppercase',
                }}>{issue.label}</div>
                <div style={{
                  fontFamily: fontMono, fontSize: 28, fontWeight: 700,
                  color: issue.color,
                  textShadow: `0 0 20px ${issue.color}`,
                  lineHeight: 1,
                }}>{issue.value}</div>
                <div style={{
                  fontFamily: font, fontSize: 12, color: issue.color,
                  opacity: 0.8,
                }}>{issue.note}</div>
              </div>
            );
          })}
        </div>

        {/* Bottom text */}
        {(() => {
          const p = interpolate(frame, [fps * 2.5, fps * 3.2], [0, 1], {
            extrapolateRight: 'clamp',
            easing: Easing.bezier(...SETTLE),
          });
          return (
            <div style={{
              marginTop: 40, opacity: p,
              transform: `translateY(${(1 - p) * 12}px)`,
              fontFamily: font, fontSize: 16, color: C.signal,
              fontWeight: 600, letterSpacing: '0.04em',
            }}>
              TkSpeed encontra, mede e resolve esses problemas.
            </div>
          );
        })()}
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
