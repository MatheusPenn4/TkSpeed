import React from 'react';
import {
  AbsoluteFill, useCurrentFrame, useVideoConfig,
  interpolate, Easing, Img, staticFile,
} from 'remotion';
import { C, SETTLE } from '../../tokens';
import { font } from '../../fonts';
import { Background } from '../../components/Background';

// Scene 07 — 120 frames = 4s
// OUTRO: Epic logo reveal. Strong CTA. Clean finish.
export const ShortOutro: React.FC = () => {
  const frame = useCurrentFrame();
  useVideoConfig();

  const fadeIn = interpolate(frame, [0, 10], [0, 1], { extrapolateRight: 'clamp' });

  // Ring 1 (inner)
  const r1Scale = interpolate(frame, [5, 50], [0.5, 1.3], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0.8, 0.4, 1),
  });
  const r1Opacity = interpolate(frame, [5, 24, 95, 112], [0, 0.38, 0.38, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  // Ring 2 (outer)
  const r2Scale = interpolate(frame, [12, 65], [0.45, 1.7], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0.8, 0.4, 1),
  });
  const r2Opacity = interpolate(frame, [12, 32, 95, 112], [0, 0.2, 0.2, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  // Symbol
  const symP = interpolate(frame, [6, 34], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });
  const symScale = interpolate(frame, [6, 34], [0.35, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // Wordmark
  const wordP = interpolate(frame, [26, 50], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });
  const wordY = interpolate(frame, [26, 50], [22, 0], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // Tagline
  const tagP = interpolate(frame, [40, 60], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // CTA button
  const ctaP = interpolate(frame, [52, 74], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });
  const ctaY = interpolate(frame, [52, 74], [18, 0], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // CTA glow pulse
  const glowCycle = interpolate(frame % 88, [0, 44, 88], [0.55, 1, 0.55]);

  // Sub-caption under CTA
  const capP = interpolate(frame, [68, 86], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // Horizontal divider at end
  const divP = interpolate(frame, [80, 98], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // Inner glow behind symbol
  const innerGlow = interpolate(Math.sin(frame / 42), [-1, 1], [0.08, 0.18]);

  return (
    <AbsoluteFill style={{ opacity: fadeIn }}>
      <Background pulse />

      {/* Glow halo behind symbol */}
      <div style={{
        position: 'absolute',
        top: '40%', left: '50%',
        width: 480, height: 480,
        marginLeft: -240, marginTop: -240,
        borderRadius: '50%',
        background: `radial-gradient(ellipse, rgba(88,242,210,${innerGlow}) 0%, transparent 68%)`,
        pointerEvents: 'none',
      }} />

      {/* Rings */}
      <div style={{
        position: 'absolute',
        top: '40%', left: '50%',
        width: 600, height: 600,
        marginLeft: -300, marginTop: -300,
        borderRadius: '50%',
        border: `1.5px solid ${C.signal}`,
        transform: `scale(${r2Scale})`,
        opacity: r2Opacity,
      }} />
      <div style={{
        position: 'absolute',
        top: '40%', left: '50%',
        width: 400, height: 400,
        marginLeft: -200, marginTop: -200,
        borderRadius: '50%',
        border: `1px solid ${C.signal}`,
        transform: `scale(${r1Scale})`,
        opacity: r1Opacity,
      }} />

      <AbsoluteFill style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '192px 88px 288px',
        gap: 0,
      }}>

        {/* Symbol */}
        <div style={{
          opacity: symP,
          transform: `scale(${symScale})`,
          marginBottom: 30,
          filter: `drop-shadow(0 0 56px ${C.signal}) drop-shadow(0 0 20px ${C.signal}90)`,
        }}>
          <Img
            src={staticFile('symbol.png')}
            style={{ width: 116, height: 'auto', objectFit: 'contain' }}
          />
        </div>

        {/* Wordmark */}
        <div style={{
          opacity: wordP,
          transform: `translateY(${wordY}px)`,
          fontFamily: font,
          fontSize: 86,
          fontWeight: 700,
          color: C.inkHi,
          letterSpacing: '-0.025em',
          lineHeight: 1,
          marginBottom: 14,
        }}>
          TkSpeed
        </div>

        {/* Tagline */}
        <div style={{
          opacity: tagP,
          fontFamily: font,
          fontSize: 24,
          fontWeight: 500,
          color: C.inkMid,
          letterSpacing: '0.14em',
          textTransform: 'uppercase',
          marginBottom: 56,
        }}>
          Engenharia de Performance
        </div>

        {/* CTA button */}
        <div style={{
          opacity: ctaP,
          transform: `translateY(${ctaY}px)`,
          background: `linear-gradient(135deg, ${C.signal} 0%, ${C.ion} 100%)`,
          borderRadius: 18,
          padding: '24px 0',
          width: '100%',
          textAlign: 'center',
          fontFamily: font,
          fontSize: 30,
          fontWeight: 700,
          color: C.void,
          boxShadow: `0 0 ${42 * glowCycle}px ${C.signal}65, 0 10px 48px rgba(0,0,0,0.5)`,
          marginBottom: 18,
        }}>
          Baixe agora — Gratuito
        </div>

        {/* Sub-caption */}
        <div style={{
          opacity: capP,
          fontFamily: font,
          fontSize: 22,
          color: C.inkLow,
          letterSpacing: '0.04em',
          textAlign: 'center',
          lineHeight: 1.5,
        }}>
          Windows 10 / 11 · 64-bit · Sem dados enviados
        </div>

        {/* Divider */}
        <div style={{
          marginTop: 44,
          opacity: divP * 0.35,
          width: 260,
          height: 1,
          background: `linear-gradient(90deg, transparent, ${C.signal}, transparent)`,
        }} />

      </AbsoluteFill>
    </AbsoluteFill>
  );
};
