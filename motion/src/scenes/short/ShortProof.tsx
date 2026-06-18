import React from 'react';
import {
  AbsoluteFill, useCurrentFrame, useVideoConfig,
  interpolate, Easing,
} from 'remotion';
import { C, SETTLE } from '../../tokens';
import { font, fontMono } from '../../fonts';
import { Background } from '../../components/Background';

// Scene 05 — 165 frames = 5.5s
// PROOF: Hard numbers. FPS 94→127. Score 380→760. +35% badge slams in.
export const ShortProof: React.FC = () => {
  const frame = useCurrentFrame();
  useVideoConfig();

  const fadeIn = interpolate(frame, [0, 10], [0, 1], { extrapolateRight: 'clamp' });
  const fadeOut = interpolate(frame, [150, 165], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  // Header
  const headerP = interpolate(frame, [5, 24], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // FPS card entrance
  const fpsCardP = interpolate(frame, [18, 38], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // FPS counter: 94 → 127
  const fpsVal = Math.round(interpolate(frame, [25, 88], [94, 127], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0, 0.4, 1),
  }));

  // +35% badge — overshoot pop
  const gainP = interpolate(frame, [88, 108], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.34, 1.56, 0.64, 1),
  });

  // Latency row
  const latP = interpolate(frame, [100, 118], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // Score row
  const scoreP = interpolate(frame, [110, 128], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });
  const scoreVal = Math.round(interpolate(frame, [110, 150], [380, 760], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0, 0.4, 1),
  }));

  // Caption
  const capP = interpolate(frame, [128, 145], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // FPS color: transitions from warn toward ok as FPS climbs
  const fpsColor = fpsVal > 115 ? C.ok : fpsVal > 100 ? C.signal : C.warn;

  return (
    <AbsoluteFill style={{ opacity: fadeIn * fadeOut }}>
      <Background />

      <AbsoluteFill style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        padding: '192px 88px 288px',
        gap: 24,
      }}>

        {/* Header */}
        <div style={{
          opacity: headerP,
          transform: `translateY(${(1 - headerP) * 14}px)`,
          width: '100%',
          textAlign: 'center',
        }}>
          <div style={{
            fontFamily: font,
            fontSize: 20,
            fontWeight: 600,
            color: C.ok,
            letterSpacing: '0.14em',
            textTransform: 'uppercase',
            marginBottom: 6,
          }}>
            Resultado verificado
          </div>
          <div style={{
            fontFamily: font,
            fontSize: 52,
            fontWeight: 700,
            color: C.inkHi,
            letterSpacing: '-0.025em',
            lineHeight: 1.1,
          }}>
            Prova por medição.
            <span style={{ color: C.signal }}> Não por promessa.</span>
          </div>
        </div>

        {/* FPS hero card */}
        <div style={{
          opacity: fpsCardP,
          transform: `scale(${0.88 + fpsCardP * 0.12})`,
          width: '100%',
          background: `${C.ok}0e`,
          border: `1.5px solid ${C.ok}30`,
          borderRadius: 20,
          padding: '28px 32px',
        }}>
          <div style={{
            fontFamily: font,
            fontSize: 18,
            fontWeight: 600,
            color: C.inkLow,
            letterSpacing: '0.1em',
            textTransform: 'uppercase',
            marginBottom: 16,
          }}>
            FPS em jogo — Antes vs Depois
          </div>

          <div style={{
            display: 'flex',
            alignItems: 'center',
            gap: 24,
          }}>
            {/* Before */}
            <div style={{ textAlign: 'center' }}>
              <div style={{
                fontFamily: fontMono,
                fontSize: 36,
                fontWeight: 700,
                color: C.inkLow,
                textDecoration: 'line-through',
                lineHeight: 1,
              }}>
                94
              </div>
              <div style={{
                fontFamily: font,
                fontSize: 18,
                color: C.inkLow,
                marginTop: 4,
              }}>
                antes
              </div>
            </div>

            <div style={{ color: C.inkFaint, fontSize: 36, flex: 1, textAlign: 'center' }}>→</div>

            {/* After — big hero number */}
            <div style={{ textAlign: 'center' }}>
              <div style={{
                fontFamily: fontMono,
                fontSize: 104,
                fontWeight: 800,
                color: fpsColor,
                lineHeight: 0.9,
                textShadow: `0 0 48px ${fpsColor}80`,
              }}>
                {fpsVal}
              </div>
              <div style={{
                fontFamily: font,
                fontSize: 20,
                color: fpsColor,
                marginTop: 6,
              }}>
                fps agora
              </div>
            </div>
          </div>
        </div>

        {/* +35% gain badge — SLAMS IN */}
        <div style={{
          opacity: gainP,
          transform: `scale(${gainP < 0.01 ? 0 : gainP})`,
          display: 'flex',
          gap: 18,
          alignItems: 'center',
          background: `${C.ok}16`,
          border: `2px solid ${C.ok}50`,
          borderRadius: 18,
          padding: '20px 28px',
          width: '100%',
        }}>
          <div style={{
            fontFamily: fontMono,
            fontSize: 64,
            fontWeight: 800,
            color: C.ok,
            lineHeight: 1,
            textShadow: `0 0 40px ${C.ok}80`,
          }}>
            +35%
          </div>
          <div style={{
            fontFamily: font,
            fontSize: 28,
            fontWeight: 700,
            color: C.inkHi,
            lineHeight: 1.25,
          }}>
            de performance<br />comprovada
          </div>
        </div>

        {/* Latency row */}
        <div style={{
          opacity: latP,
          transform: `translateY(${(1 - latP) * 14}px)`,
          display: 'flex',
          gap: 16,
          width: '100%',
        }}>
          {[
            { label: 'Latência 1%',  before: '16ms', after: '11ms', color: C.ion   },
            { label: 'Input lag',    before: '8ms',  after: '5ms',  color: C.signal },
          ].map((m, i) => (
            <div key={i} style={{
              flex: 1,
              background: C.panel,
              border: `1px solid ${C.hairline}`,
              borderRadius: 14,
              padding: '16px 18px',
            }}>
              <div style={{
                fontFamily: font,
                fontSize: 18,
                color: C.inkLow,
                marginBottom: 6,
              }}>
                {m.label}
              </div>
              <div style={{ display: 'flex', alignItems: 'baseline', gap: 8 }}>
                <span style={{
                  fontFamily: fontMono,
                  fontSize: 22,
                  color: C.inkLow,
                  textDecoration: 'line-through',
                }}>
                  {m.before}
                </span>
                <span style={{ color: C.inkFaint }}>→</span>
                <span style={{
                  fontFamily: fontMono,
                  fontSize: 28,
                  fontWeight: 700,
                  color: m.color,
                  textShadow: `0 0 14px ${m.color}60`,
                }}>
                  {m.after}
                </span>
              </div>
            </div>
          ))}
        </div>

        {/* Score row */}
        <div style={{
          opacity: scoreP,
          transform: `translateY(${(1 - scoreP) * 14}px)`,
          display: 'flex',
          gap: 16,
          alignItems: 'center',
          width: '100%',
          background: C.panel,
          border: `1px solid ${C.hairline}`,
          borderRadius: 14,
          padding: '18px 22px',
        }}>
          <div style={{
            fontFamily: font,
            fontSize: 22,
            fontWeight: 600,
            color: C.inkMid,
            flex: 1,
          }}>
            TkSpeed Score
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <span style={{ fontFamily: fontMono, fontSize: 24, color: C.inkLow }}>380</span>
            <span style={{ color: C.inkFaint, fontSize: 18 }}>→</span>
            <span style={{
              fontFamily: fontMono,
              fontSize: 34,
              fontWeight: 700,
              color: C.signal,
              textShadow: `0 0 20px ${C.signal}60`,
            }}>
              {scoreVal}
            </span>
            <span style={{
              fontFamily: font,
              fontSize: 18,
              fontWeight: 600,
              color: C.ok,
              background: `${C.ok}1a`,
              border: `1px solid ${C.ok}44`,
              borderRadius: 7,
              padding: '3px 10px',
            }}>
              +380 ▲
            </span>
          </div>
        </div>

        {/* Caption */}
        <div style={{
          opacity: capP,
          fontFamily: fontMono,
          fontSize: 20,
          color: C.inkLow,
          textAlign: 'center',
        }}>
          Confiança 94% · 5 sessões analisadas
        </div>

      </AbsoluteFill>
    </AbsoluteFill>
  );
};
