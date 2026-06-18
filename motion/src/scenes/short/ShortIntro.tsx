import React from 'react';
import {
  AbsoluteFill, useCurrentFrame,
  interpolate, Easing, Img, staticFile,
} from 'remotion';
import { C, SETTLE } from '../../tokens';
import { font } from '../../fonts';
import { Background } from '../../components/Background';

// Scene 02 — 105 frames = 3.5s
// INTRO: Premium logo reveal with rings. Sets the brand tone.
export const ShortIntro: React.FC = () => {
  const frame = useCurrentFrame();

  const fadeIn = interpolate(frame, [0, 8], [0, 1], { extrapolateRight: 'clamp' });
  const fadeOut = interpolate(frame, [90, 105], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  // Symbol
  const symP = interpolate(frame, [5, 32], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });
  const symScale = interpolate(frame, [5, 32], [0.35, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // Ring 1 (inner)
  const r1Scale = interpolate(frame, [10, 45], [0.5, 1.25], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0.8, 0.4, 1),
  });
  const r1Opacity = interpolate(frame, [10, 26, 75, 95], [0, 0.42, 0.42, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  // Ring 2 (outer)
  const r2Scale = interpolate(frame, [18, 60], [0.45, 1.6], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0.8, 0.4, 1),
  });
  const r2Opacity = interpolate(frame, [18, 36, 75, 95], [0, 0.22, 0.22, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  // Wordmark
  const wordP = interpolate(frame, [28, 52], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });
  const wordY = interpolate(frame, [28, 52], [24, 0], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // Tagline
  const tagP = interpolate(frame, [42, 62], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // Three keywords
  const kwP = interpolate(frame, [55, 75], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // Inner glow pulse
  const glowPulse = interpolate(Math.sin(frame / 38), [-1, 1], [0.06, 0.14]);

  return (
    <AbsoluteFill style={{ opacity: fadeIn * fadeOut }}>
      <Background pulse />

      {/* Inner glow halo */}
      <div style={{
        position: 'absolute',
        top: '42%', left: '50%',
        width: 420, height: 420,
        marginLeft: -210, marginTop: -210,
        borderRadius: '50%',
        background: `radial-gradient(ellipse, rgba(88,242,210,${glowPulse}) 0%, transparent 65%)`,
        pointerEvents: 'none',
      }} />

      {/* Rings */}
      <div style={{
        position: 'absolute',
        top: '42%', left: '50%',
        width: 560, height: 560,
        marginLeft: -280, marginTop: -280,
        borderRadius: '50%',
        border: `1.5px solid ${C.signal}`,
        transform: `scale(${r2Scale})`,
        opacity: r2Opacity,
      }} />
      <div style={{
        position: 'absolute',
        top: '42%', left: '50%',
        width: 360, height: 360,
        marginLeft: -180, marginTop: -180,
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
          marginBottom: 36,
          filter: `drop-shadow(0 0 48px ${C.signal}) drop-shadow(0 0 16px ${C.signal}80)`,
        }}>
          <Img
            src={staticFile('symbol.png')}
            style={{ width: 108, height: 'auto', objectFit: 'contain' }}
          />
        </div>

        {/* Wordmark */}
        <div style={{
          opacity: wordP,
          transform: `translateY(${wordY}px)`,
          fontFamily: font,
          fontSize: 76,
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
          marginBottom: 48,
        }}>
          Engenharia de Performance
        </div>

        {/* Three keywords */}
        <div style={{
          opacity: kwP,
          transform: `translateY(${(1 - kwP) * 12}px)`,
          display: 'flex',
          gap: 0,
          background: C.panel,
          border: `1px solid ${C.hairline}`,
          borderRadius: 14,
          overflow: 'hidden',
        }}>
          {[
            { word: 'Medida',     color: C.ion    },
            { word: 'Comprovada', color: C.ok     },
            { word: 'Reversível', color: C.signal },
          ].map((kw, i) => (
            <div key={i} style={{
              padding: '16px 24px',
              borderRight: i < 2 ? `1px solid ${C.hairline}` : 'none',
              textAlign: 'center',
            }}>
              <div style={{
                fontFamily: font,
                fontSize: 24,
                fontWeight: 700,
                color: kw.color,
                textShadow: `0 0 20px ${kw.color}50`,
              }}>
                {kw.word}
              </div>
            </div>
          ))}
        </div>

      </AbsoluteFill>
    </AbsoluteFill>
  );
};
