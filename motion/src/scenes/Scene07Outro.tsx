import React from 'react';
import {
  AbsoluteFill, useCurrentFrame, useVideoConfig,
  interpolate, Easing, Img, staticFile,
} from 'remotion';
import { C, SETTLE } from '../tokens';
import { font } from '../fonts';
import { Background } from '../components/Background';

// Scene: 210 frames = 7s
export const Scene07Outro: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const fadeIn = interpolate(frame, [0, fps * 0.6], [0, 1], { extrapolateRight: 'clamp' });

  const ringScale = interpolate(frame, [fps * 0.3, fps * 1.8], [0.6, 1.2], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  const ringOpacity = interpolate(frame, [fps * 0.3, fps * 1.0, fps * 4.5, fps * 5.5], [0, 0.3, 0.3, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  const symbolP = interpolate(frame, [fps * 0.2, fps * 1.2], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  const symbolScale = interpolate(frame, [fps * 0.2, fps * 1.2], [0.5, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  const wordmarkP = interpolate(frame, [fps * 0.8, fps * 1.8], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  const wordmarkY = interpolate(frame, [fps * 0.8, fps * 1.8], [18, 0], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  const taglineP = interpolate(frame, [fps * 1.3, fps * 2.3], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  const ctaP = interpolate(frame, [fps * 2, fps * 3.2], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  const ctaY = interpolate(frame, [fps * 2, fps * 3.2], [14, 0], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  const glowPulse = interpolate(frame % (fps * 2.8), [0, fps * 1.4, fps * 2.8], [0.6, 1, 0.6]);

  return (
    <AbsoluteFill style={{ opacity: fadeIn }}>
      <Background pulse />

      {/* Outer ring */}
      <div style={{
        position: 'absolute',
        width: 500, height: 500,
        borderRadius: '50%',
        border: `1px solid ${C.signal}`,
        transform: `scale(${ringScale})`,
        opacity: ringOpacity,
        left: '50%', top: '50%',
        marginLeft: -250, marginTop: -250,
      }} />

      {/* Inner glow */}
      <div style={{
        position: 'absolute',
        left: '50%', top: '50%',
        width: 350, height: 350,
        marginLeft: -175, marginTop: -175,
        borderRadius: '50%',
        background: `radial-gradient(ellipse, rgba(88,242,210,${0.08 * glowPulse}) 0%, transparent 70%)`,
      }} />

      <AbsoluteFill style={{
        display: 'flex', flexDirection: 'column',
        alignItems: 'center', justifyContent: 'center',
        gap: 0,
      }}>
        {/* Symbol */}
        <div style={{
          opacity: symbolP,
          transform: `scale(${symbolScale})`,
          marginBottom: 24,
          filter: `drop-shadow(0 0 32px ${C.signal}) drop-shadow(0 0 8px ${C.signal})`,
        }}>
          <Img
            src={staticFile('symbol.png')}
            style={{ width: 88, height: 'auto', objectFit: 'contain' }}
          />
        </div>

        {/* TkSpeed wordmark */}
        <div style={{
          opacity: wordmarkP,
          transform: `translateY(${wordmarkY}px)`,
          fontFamily: font, fontSize: 56, fontWeight: 700,
          color: C.inkHi, letterSpacing: '-0.025em',
          lineHeight: 1, marginBottom: 10,
        }}>
          TkSpeed
        </div>

        {/* Tagline */}
        <div style={{
          opacity: taglineP,
          fontFamily: font, fontSize: 18, fontWeight: 500,
          color: C.inkMid, letterSpacing: '0.1em',
          textTransform: 'uppercase', marginBottom: 40,
        }}>
          Engenharia de Performance
        </div>

        {/* CTA */}
        <div style={{
          opacity: ctaP,
          transform: `translateY(${ctaY}px)`,
          display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 16,
        }}>
          <div style={{
            background: `linear-gradient(135deg, ${C.signal}, ${C.ion})`,
            borderRadius: 12, padding: '16px 40px',
            fontFamily: font, fontSize: 18, fontWeight: 700,
            color: C.void,
            boxShadow: `0 0 ${30 * glowPulse}px ${C.signal}60`,
          }}>
            Baixe agora — gratuito
          </div>
          <div style={{
            fontFamily: font, fontSize: 14, color: C.inkLow,
            letterSpacing: '0.04em',
          }}>
            Windows 10 / 11 · 64-bit · Sem dados enviados
          </div>
        </div>

        {/* Divider line */}
        {(() => {
          const p = interpolate(frame, [fps * 3, fps * 4], [0, 1], {
            extrapolateRight: 'clamp',
            easing: Easing.bezier(...SETTLE),
          });
          return (
            <div style={{
              marginTop: 48,
              opacity: p * 0.5,
              width: 200, height: 1,
              background: `linear-gradient(90deg, transparent, ${C.signal}, transparent)`,
            }} />
          );
        })()}
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
