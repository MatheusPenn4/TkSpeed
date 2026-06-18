import React from 'react';
import {
  AbsoluteFill, useCurrentFrame, useVideoConfig,
  interpolate, Easing,
} from 'remotion';
import { C, SETTLE } from '../../tokens';
import { font, fontMono } from '../../fonts';
import { Background } from '../../components/Background';
import { ScoreArc } from '../../components/ScoreArc';

const BOTTLENECKS = [
  { label: 'RAM',  pct: 84, color: C.risk, note: 'Pressão crítica'  },
  { label: 'CPU',  pct: 72, color: C.warn, note: 'Sobrecarregada'   },
  { label: 'GPU',  pct: 31, color: C.ok,   note: 'Normal'           },
];

// Scene 03 — 195 frames = 6.5s
// DASHBOARD: Score arc 380→760. Bottleneck bars. Auto Pilot CTA.
export const ShortDashboard: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const fadeIn = interpolate(frame, [0, 10], [0, 1], { extrapolateRight: 'clamp' });
  const fadeOut = interpolate(frame, [180, 195], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  // Header
  const headerP = interpolate(frame, [5, 24], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // Arc draws in (frames 12-60), then stays at 1
  const arcDrawP = interpolate(frame, [12, 60], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // Score 380 → 760 transition (frames 80-140)
  const scoreVal = interpolate(frame, [80, 140], [380, 760], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0, 0.4, 1),
  });

  // Arc container fade-in
  const arcContainerP = interpolate(frame, [10, 24], [0, 1], { extrapolateRight: 'clamp' });

  // Bottleneck bars
  const barsContainerP = interpolate(frame, [60, 75], [0, 1], { extrapolateRight: 'clamp' });

  // Score-jump label (appears at frame 140)
  const jumpLabelP = interpolate(frame, [140, 158], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // Auto Pilot CTA (frame 110)
  const ctaP = interpolate(frame, [110, 130], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });
  const ctaGlow = interpolate(frame % (fps * 2), [0, fps, fps * 2], [0.65, 1, 0.65]);

  return (
    <AbsoluteFill style={{ opacity: fadeIn * fadeOut }}>
      <Background />

      <AbsoluteFill style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        padding: '192px 88px 288px',
        gap: 28,
      }}>

        {/* Header */}
        <div style={{
          opacity: headerP,
          transform: `translateY(${(1 - headerP) * 14}px)`,
          textAlign: 'center',
          width: '100%',
        }}>
          <div style={{
            fontFamily: font,
            fontSize: 20,
            fontWeight: 600,
            color: C.signal,
            letterSpacing: '0.14em',
            textTransform: 'uppercase',
            marginBottom: 6,
          }}>
            Central de Comando
          </div>
          <div style={{
            fontFamily: font,
            fontSize: 46,
            fontWeight: 700,
            color: C.inkHi,
            letterSpacing: '-0.025em',
            lineHeight: 1.1,
          }}>
            Estado da Máquina
          </div>
        </div>

        {/* Score Arc — HERO ELEMENT */}
        <div style={{ opacity: arcContainerP }}>
          <ScoreArc score={scoreVal} progress={arcDrawP} size={400} />
        </div>

        {/* Score jump badge */}
        <div style={{
          opacity: jumpLabelP,
          transform: `translateY(${(1 - jumpLabelP) * 14}px) scale(${0.9 + jumpLabelP * 0.1})`,
          display: 'flex',
          gap: 14,
          alignItems: 'center',
          background: `${C.signal}14`,
          border: `1.5px solid ${C.signal}44`,
          borderRadius: 14,
          padding: '16px 24px',
          width: '100%',
        }}>
          <div style={{ fontSize: 28 }}>🚀</div>
          <div style={{ flex: 1 }}>
            <div style={{
              fontFamily: font,
              fontSize: 22,
              fontWeight: 700,
              color: C.signal,
            }}>
              Score otimizado
            </div>
            <div style={{
              fontFamily: fontMono,
              fontSize: 20,
              color: C.inkMid,
              marginTop: 2,
            }}>
              380 → 760 &nbsp;
              <span style={{ color: C.ok }}>+380 ▲</span>
            </div>
          </div>
        </div>

        {/* Bottleneck bars */}
        <div style={{
          opacity: barsContainerP,
          width: '100%',
          display: 'flex',
          flexDirection: 'column',
          gap: 18,
        }}>
          <div style={{
            fontFamily: font,
            fontSize: 18,
            fontWeight: 600,
            color: C.inkLow,
            letterSpacing: '0.09em',
            textTransform: 'uppercase',
          }}>
            Gargalos detectados
          </div>
          {BOTTLENECKS.map((b, i) => {
            const barP = interpolate(
              frame,
              [62 + i * 12, 80 + i * 12],
              [0, 1],
              { extrapolateRight: 'clamp', easing: Easing.bezier(...SETTLE) }
            );
            return (
              <div key={i} style={{
                opacity: barP,
                transform: `translateX(${(1 - barP) * -22}px)`,
              }}>
                <div style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  marginBottom: 7,
                }}>
                  <span style={{
                    fontFamily: font,
                    fontSize: 24,
                    fontWeight: 600,
                    color: C.inkHi,
                  }}>
                    {b.label}
                  </span>
                  <span style={{
                    fontFamily: fontMono,
                    fontSize: 24,
                    fontWeight: 700,
                    color: b.color,
                    textShadow: `0 0 16px ${b.color}60`,
                  }}>
                    {b.pct}%
                  </span>
                </div>
                <div style={{
                  height: 9,
                  background: C.raised,
                  borderRadius: 5,
                  overflow: 'hidden',
                }}>
                  <div style={{
                    height: '100%',
                    width: `${b.pct}%`,
                    background: b.color,
                    borderRadius: 5,
                    boxShadow: `0 0 14px ${b.color}80`,
                  }} />
                </div>
                <div style={{
                  fontFamily: font,
                  fontSize: 18,
                  color: b.color,
                  opacity: 0.65,
                  marginTop: 5,
                }}>
                  {b.note}
                </div>
              </div>
            );
          })}
        </div>

        {/* Auto Pilot CTA */}
        <div style={{
          opacity: ctaP,
          transform: `translateY(${(1 - ctaP) * 18}px) scale(${0.92 + ctaP * 0.08})`,
          background: `linear-gradient(135deg, ${C.signal}, ${C.ion})`,
          borderRadius: 18,
          padding: '22px 0',
          width: '100%',
          textAlign: 'center',
          fontFamily: font,
          fontSize: 28,
          fontWeight: 700,
          color: C.void,
          boxShadow: `0 0 ${36 * ctaGlow}px ${C.signal}55, 0 8px 40px rgba(0,0,0,0.45)`,
        }}>
          ⚡ Auto Pilot — Otimizar
        </div>

      </AbsoluteFill>
    </AbsoluteFill>
  );
};
