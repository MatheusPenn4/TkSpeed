import React from 'react';
import {
  AbsoluteFill, useCurrentFrame, useVideoConfig,
  interpolate, Easing,
} from 'remotion';
import { C, SETTLE } from '../../tokens';
import { font, fontMono } from '../../fonts';
import { Background } from '../../components/Background';

// Scene 01 — 90 frames = 3s
// HOOK: Aggressive problem statement. Stops scroll in first 2 seconds.
export const ShortHook: React.FC = () => {
  const frame = useCurrentFrame();
  useVideoConfig();

  // Lifecycle
  const fadeIn = interpolate(frame, [0, 4], [0, 1], { extrapolateRight: 'clamp' });
  const fadeOut = interpolate(frame, [75, 90], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  // Red flash on first hit
  const flashOpacity = interpolate(frame, [0, 2, 7], [0.4, 0.4, 0], { extrapolateRight: 'clamp' });

  // MAIN HEADLINE — slams in hard
  const headP = interpolate(frame, [3, 18], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.22, 1, 0.36, 1),
  });
  const headScale = interpolate(frame, [3, 18], [0.75, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.22, 1, 0.36, 1),
  });

  // Metric cards stagger in
  const m1 = interpolate(frame, [22, 36], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });
  const m2 = interpolate(frame, [28, 42], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });
  const m3 = interpolate(frame, [34, 48], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // Punch line at bottom
  const punchP = interpolate(frame, [50, 64], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  const METRICS = [
    { label: 'CPU',  value: '92%',  color: C.risk, p: m1 },
    { label: 'RAM',  value: '84%',  color: C.warn, p: m2 },
    { label: 'TEMP', value: '91°C', color: C.risk, p: m3 },
  ];

  return (
    <AbsoluteFill style={{ opacity: fadeIn * fadeOut }}>
      <Background />

      {/* Impact flash */}
      <AbsoluteFill style={{
        background: C.risk,
        opacity: flashOpacity * 0.12,
      }} />

      <AbsoluteFill style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '192px 88px 288px',
      }}>

        {/* Hero text */}
        <div style={{
          opacity: headP,
          transform: `scale(${headScale})`,
          textAlign: 'center',
          marginBottom: 60,
        }}>
          <div style={{
            fontFamily: font,
            fontSize: 144,
            fontWeight: 800,
            color: C.risk,
            lineHeight: 0.88,
            letterSpacing: '-0.04em',
            textShadow: `0 0 80px ${C.risk}70, 0 0 160px ${C.risk}30`,
          }}>
            BAIXO
          </div>
          <div style={{
            fontFamily: font,
            fontSize: 144,
            fontWeight: 800,
            color: C.inkHi,
            lineHeight: 0.88,
            letterSpacing: '-0.04em',
          }}>
            FPS?
          </div>
        </div>

        {/* Metric cards */}
        <div style={{ display: 'flex', gap: 16, width: '100%', marginBottom: 48 }}>
          {METRICS.map((m, i) => (
            <div key={i} style={{
              flex: 1,
              background: `${m.color}14`,
              border: `1.5px solid ${m.color}50`,
              borderRadius: 18,
              padding: '26px 12px',
              textAlign: 'center',
              opacity: m.p,
              transform: `translateY(${(1 - m.p) * 28}px)`,
              position: 'relative',
              overflow: 'hidden',
            }}>
              {/* Top accent bar */}
              <div style={{
                position: 'absolute',
                top: 0, left: 0, right: 0,
                height: 2,
                background: m.color,
                boxShadow: `0 0 8px ${m.color}`,
              }} />
              <div style={{
                fontFamily: fontMono,
                fontSize: 52,
                fontWeight: 700,
                color: m.color,
                lineHeight: 1,
                textShadow: `0 0 28px ${m.color}80`,
              }}>
                {m.value}
              </div>
              <div style={{
                fontFamily: font,
                fontSize: 22,
                fontWeight: 600,
                color: m.color,
                opacity: 0.65,
                letterSpacing: '0.1em',
                marginTop: 8,
                textTransform: 'uppercase',
              }}>
                {m.label}
              </div>
            </div>
          ))}
        </div>

        {/* Punch line */}
        <div style={{
          opacity: punchP,
          transform: `translateY(${(1 - punchP) * 16}px)`,
          fontFamily: font,
          fontSize: 36,
          fontWeight: 500,
          color: C.inkMid,
          textAlign: 'center',
          lineHeight: 1.4,
        }}>
          Seu PC pode muito mais.
        </div>

      </AbsoluteFill>
    </AbsoluteFill>
  );
};
