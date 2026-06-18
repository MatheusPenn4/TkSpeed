import React from 'react';
import {
  AbsoluteFill, useCurrentFrame, useVideoConfig,
  interpolate, Easing,
} from 'remotion';
import { C, SETTLE } from '../tokens';
import { font, fontMono } from '../fonts';
import { Background } from '../components/Background';

const BENCHMARKS = [
  { suite: 'CPU Single-thread', before: 2240, after: 2580, unit: 'pts', color: C.ion },
  { suite: 'CPU Multi-thread', before: 12400, after: 14100, unit: 'pts', color: C.ion },
  { suite: 'RAM Bandwidth', before: 38.2, after: 42.6, unit: 'GB/s', color: C.signal },
  { suite: 'RAM Latência', before: 82, after: 68, unit: 'ns', color: C.warn, lowerBetter: true },
  { suite: 'Disco Seq. Leitura', before: 3200, after: 3200, unit: 'MB/s', color: C.inkMid },
];

// Scene: 450 frames = 15s
export const Scene04PerfLab: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const fadeIn = interpolate(frame, [0, fps * 0.5], [0, 1], { extrapolateRight: 'clamp' });
  const fadeOut = interpolate(frame, [fps * 13.5, fps * 15], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  // Benchmark "running" progress
  const runProgress = interpolate(frame, [fps * 1, fps * 4.5], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0.6, 0.8, 1),
  });

  // Show results after run
  const showResults = interpolate(frame, [fps * 5, fps * 5.8], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  return (
    <AbsoluteFill style={{ opacity: fadeIn * fadeOut }}>
      <Background />

      <AbsoluteFill style={{
        display: 'flex', flexDirection: 'column',
        padding: '60px 100px', gap: 28,
      }}>
        {/* Header */}
        {(() => {
          const p = interpolate(frame, [fps * 0.2, fps * 1], [0, 1], {
            extrapolateRight: 'clamp',
            easing: Easing.bezier(...SETTLE),
          });
          return (
            <div style={{ opacity: p, transform: `translateY(${(1 - p) * 16}px)` }}>
              <div style={{
                fontFamily: font, fontSize: 13, fontWeight: 600,
                color: C.ion, letterSpacing: '0.1em',
                textTransform: 'uppercase', marginBottom: 8,
              }}>
                Laboratório de Performance
              </div>
              <div style={{
                fontFamily: font, fontSize: 38, fontWeight: 700,
                color: C.inkHi, letterSpacing: '-0.02em', lineHeight: 1.1,
              }}>
                Benchmark mensurável.
              </div>
              <div style={{
                fontFamily: font, fontSize: 38, fontWeight: 700,
                color: C.ion, letterSpacing: '-0.02em', lineHeight: 1.1,
              }}>
                Ganho com prova.
              </div>
            </div>
          );
        })()}

        <div style={{ display: 'flex', gap: 48, flex: 1 }}>
          {/* LEFT: running benchmark visualization */}
          <div style={{
            width: 380, background: C.panel,
            border: `1px solid ${C.hairline}`,
            borderRadius: 12, padding: '20px 24px',
            display: 'flex', flexDirection: 'column', gap: 16,
          }}>
            <div style={{
              fontFamily: font, fontSize: 11, fontWeight: 600,
              color: C.inkLow, letterSpacing: '0.08em',
              textTransform: 'uppercase',
            }}>
              Suite Completa — em execução
            </div>

            {/* Animated progress bar */}
            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 6 }}>
                <span style={{ fontFamily: font, fontSize: 13, color: C.inkMid }}>
                  {runProgress < 1 ? 'Executando testes…' : '✓ Concluído'}
                </span>
                <span style={{ fontFamily: fontMono, fontSize: 13, color: C.ion }}>
                  {Math.round(runProgress * 100)}%
                </span>
              </div>
              <div style={{ height: 6, background: C.raised, borderRadius: 3, overflow: 'hidden' }}>
                <div style={{
                  height: '100%',
                  width: `${runProgress * 100}%`,
                  background: `linear-gradient(90deg, ${C.ion}, ${C.signal})`,
                  borderRadius: 3,
                  boxShadow: runProgress < 1 ? `0 0 12px ${C.ion}` : 'none',
                }} />
              </div>
            </div>

            {/* Suite items */}
            {BENCHMARKS.map((b, i) => {
              const itemProgress = interpolate(
                frame,
                [fps * (1 + i * 0.6), fps * (1.8 + i * 0.6)],
                [0, 1],
                { extrapolateRight: 'clamp', easing: Easing.bezier(...SETTLE) }
              );
              const done = runProgress > (i + 1) / BENCHMARKS.length;
              return (
                <div key={i} style={{
                  display: 'flex', alignItems: 'center', gap: 10,
                  opacity: itemProgress,
                  transform: `translateX(${(1 - itemProgress) * -12}px)`,
                }}>
                  <div style={{
                    width: 8, height: 8, borderRadius: '50%',
                    background: done ? C.ok : C.ion,
                    boxShadow: `0 0 6px ${done ? C.ok : C.ion}`,
                    flexShrink: 0,
                  }} />
                  <span style={{ fontFamily: font, fontSize: 13, color: C.inkMid, flex: 1 }}>
                    {b.suite}
                  </span>
                  {done && (
                    <span style={{ fontFamily: fontMono, fontSize: 12, color: C.ok }}>
                      {b.after}{b.unit}
                    </span>
                  )}
                </div>
              );
            })}
          </div>

          {/* RIGHT: comparison results */}
          <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 14 }}>
            <div style={{
              fontFamily: font, fontSize: 16, fontWeight: 600,
              color: C.inkHi, marginBottom: 4,
              opacity: showResults,
            }}>
              Comparação Antes → Depois
            </div>

            {BENCHMARKS.map((b, i) => {
              const cardP = interpolate(
                frame,
                [fps * (5 + i * 0.2), fps * (5.8 + i * 0.2)],
                [0, 1],
                { extrapolateRight: 'clamp', easing: Easing.bezier(...SETTLE) }
              );
              const pct = b.lowerBetter
                ? ((b.before - b.after) / b.before) * 100
                : ((b.after - b.before) / b.before) * 100;
              const isGain = pct > 0.5;
              const verdictColor = pct > 0.5 ? C.ok : pct < -0.5 ? C.risk : C.inkLow;

              return (
                <div key={i} style={{
                  background: C.panel,
                  border: `1px solid ${C.hairline}`,
                  borderRadius: 10, padding: '12px 16px',
                  opacity: cardP,
                  transform: `translateY(${(1 - cardP) * 12}px)`,
                  display: 'flex', alignItems: 'center', gap: 16,
                }}>
                  <div style={{ flex: 1 }}>
                    <div style={{
                      fontFamily: font, fontSize: 12, color: C.inkLow,
                      marginBottom: 3,
                    }}>{b.suite}</div>
                    <div style={{ display: 'flex', gap: 12, alignItems: 'center' }}>
                      <span style={{ fontFamily: fontMono, fontSize: 14, color: C.inkLow }}>
                        {b.before}{b.unit}
                      </span>
                      <span style={{ color: C.inkFaint, fontSize: 12 }}>→</span>
                      <span style={{
                        fontFamily: fontMono, fontSize: 16, fontWeight: 700,
                        color: b.color,
                        textShadow: `0 0 12px ${b.color}`,
                      }}>
                        {b.after}{b.unit}
                      </span>
                    </div>
                  </div>
                  <div style={{
                    fontFamily: fontMono, fontSize: 16, fontWeight: 700,
                    color: verdictColor,
                    minWidth: 70, textAlign: 'right',
                  }}>
                    {isGain ? '▲' : pct < -0.5 ? '▼' : '◦'} {Math.abs(pct).toFixed(1)}%
                  </div>
                </div>
              );
            })}

            {/* Overall */}
            {(() => {
              const p = interpolate(frame, [fps * 7, fps * 8], [0, 1], {
                extrapolateRight: 'clamp',
                easing: Easing.bezier(...SETTLE),
              });
              return (
                <div style={{
                  opacity: p,
                  transform: `translateY(${(1 - p) * 12}px)`,
                  background: `${C.signal}14`,
                  border: `1px solid ${C.signal}44`,
                  borderRadius: 10, padding: '14px 18px',
                  display: 'flex', gap: 14, alignItems: 'center',
                  marginTop: 4,
                }}>
                  <div style={{ fontSize: 20 }}>📊</div>
                  <div>
                    <div style={{
                      fontFamily: font, fontSize: 14, fontWeight: 700,
                      color: C.signal,
                    }}>
                      Ganho médio confirmado: +12.4%
                    </div>
                    <div style={{
                      fontFamily: fontMono, fontSize: 12, color: C.inkMid, marginTop: 2,
                    }}>
                      Confiança 94% · 5 sessões comparadas
                    </div>
                  </div>
                </div>
              );
            })()}
          </div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
