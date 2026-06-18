import React from 'react';
import {
  AbsoluteFill, useCurrentFrame, useVideoConfig,
  interpolate, Easing,
} from 'remotion';
import { C, SETTLE } from '../tokens';
import { font, fontMono } from '../fonts';
import { Background } from '../components/Background';
import { ScoreArc } from '../components/ScoreArc';
import { SignalLockMeter } from '../components/SignalLockMeter';
import { MetricBar } from '../components/MetricBar';

const BOTTLENECKS = [
  { label: 'CPU', value: 72, color: C.warn },
  { label: 'RAM', value: 84, color: C.risk },
  { label: 'Armazenamento', value: 45, color: C.ion },
  { label: 'GPU', value: 31, color: C.ok },
  { label: 'Temperatura', value: 61, color: C.warn },
];

// Scene: 450 frames = 15s
export const Scene03MissionControl: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const fadeIn = interpolate(frame, [0, fps * 0.6], [0, 1], { extrapolateRight: 'clamp' });
  const fadeOut = interpolate(frame, [fps * 13.5, fps * 15], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  // Score goes from 380 → 760 mid-scene
  const TRANSITION_START = fps * 6;
  const TRANSITION_END = fps * 9;
  const scoreBefore = 380;
  const scoreAfter = 760;
  const scoreVal = interpolate(frame, [TRANSITION_START, TRANSITION_END], [scoreBefore, scoreAfter], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0, 0.4, 1),
  });

  const arcProgress = interpolate(frame, [fps * 0.8, fps * 2.5], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  const sectionOpacity = interpolate(frame, [fps * 1.2, fps * 2], [0, 1], {
    extrapolateRight: 'clamp',
  });

  const labelOpacity = interpolate(frame, [fps * 5.5, fps * 6.5], [0, 1], {
    extrapolateRight: 'clamp',
  });

  const signalProgress = interpolate(frame, [fps * 3, fps * 5], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  return (
    <AbsoluteFill style={{ opacity: fadeIn * fadeOut }}>
      <Background />

      <AbsoluteFill style={{
        display: 'flex', flexDirection: 'column',
        padding: '60px 100px',
      }}>
        {/* Top label */}
        <div style={{
          fontFamily: font, fontSize: 13, fontWeight: 600,
          color: C.signal, letterSpacing: '0.1em',
          textTransform: 'uppercase', marginBottom: 10,
          opacity: sectionOpacity,
        }}>
          Central de Comando
        </div>

        <div style={{ display: 'flex', gap: 60, flex: 1 }}>
          {/* LEFT: Score Arc */}
          <div style={{
            display: 'flex', flexDirection: 'column',
            alignItems: 'center', gap: 28,
            width: 320,
          }}>
            <ScoreArc score={scoreVal} progress={arcProgress} size={280} />

            <SignalLockMeter confidence={0.87} progress={signalProgress} />

            {/* "Análise completa" badge */}
            {(() => {
              const p = interpolate(frame, [fps * 2.2, fps * 3], [0, 1], {
                extrapolateRight: 'clamp',
                easing: Easing.bezier(...SETTLE),
              });
              return (
                <div style={{
                  opacity: p, transform: `translateY(${(1 - p) * 10}px)`,
                  background: `${C.signal}18`,
                  border: `1px solid ${C.signal}44`,
                  borderRadius: 8, padding: '8px 14px',
                  fontFamily: font, fontSize: 12, fontWeight: 600,
                  color: C.signal,
                }}>
                  ✓ Diagnóstico completo
                </div>
              );
            })()}
          </div>

          {/* RIGHT: details */}
          <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 28 }}>
            {/* Headline */}
            {(() => {
              const p = interpolate(frame, [fps * 0.6, fps * 1.4], [0, 1], {
                extrapolateRight: 'clamp',
                easing: Easing.bezier(...SETTLE),
              });
              return (
                <div style={{ opacity: p, transform: `translateY(${(1 - p) * 14}px)` }}>
                  <div style={{
                    fontFamily: font, fontSize: 36, fontWeight: 700,
                    color: C.inkHi, letterSpacing: '-0.02em', lineHeight: 1.1,
                  }}>
                    Estado da máquina
                  </div>
                  <div style={{
                    fontFamily: font, fontSize: 16, color: C.inkMid,
                    marginTop: 6, lineHeight: 1.5,
                  }}>
                    Diagnóstico em tempo real de todos os subsistemas
                  </div>
                </div>
              );
            })()}

            {/* Gargalos */}
            <div style={{ opacity: sectionOpacity }}>
              <div style={{
                fontFamily: font, fontSize: 11, fontWeight: 600,
                color: C.inkLow, letterSpacing: '0.08em',
                textTransform: 'uppercase', marginBottom: 14,
              }}>
                Gargalos detectados
              </div>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
                {BOTTLENECKS.map((b, i) => {
                  const barP = interpolate(
                    frame,
                    [fps * (1.5 + i * 0.15), fps * (2.3 + i * 0.15)],
                    [0, 1],
                    { extrapolateRight: 'clamp', easing: Easing.bezier(...SETTLE) }
                  );
                  return (
                    <MetricBar
                      key={i}
                      label={b.label}
                      value={b.value}
                      color={b.color}
                      progress={barP}
                      critical={b.value > 80}
                    />
                  );
                })}
              </div>
            </div>

            {/* "Otimizado" label after transition */}
            <div style={{
              opacity: labelOpacity,
              transform: `translateY(${(1 - labelOpacity) * 12}px)`,
              background: `${C.signal}14`,
              border: `1px solid ${C.signal}40`,
              borderRadius: 10,
              padding: '14px 18px',
              display: 'flex', gap: 12, alignItems: 'center',
            }}>
              <div style={{ fontSize: 22 }}>🚀</div>
              <div>
                <div style={{
                  fontFamily: font, fontSize: 15, fontWeight: 700,
                  color: C.signal,
                }}>Score melhorou após otimização</div>
                <div style={{
                  fontFamily: fontMono, fontSize: 13, color: C.inkMid, marginTop: 2,
                }}>
                  {scoreBefore} → {scoreAfter} pontos &nbsp;
                  <span style={{ color: C.ok }}>+{scoreAfter - scoreBefore} ▲</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
