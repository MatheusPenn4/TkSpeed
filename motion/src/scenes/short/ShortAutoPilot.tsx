import React from 'react';
import {
  AbsoluteFill, useCurrentFrame, useVideoConfig,
  interpolate, Easing,
} from 'remotion';
import { C, SETTLE } from '../../tokens';
import { font, fontMono } from '../../fonts';
import { Background } from '../../components/Background';

const STEPS = [
  { label: 'Criando ponto de restauração', time: '0.3s' },
  { label: 'Benchmark de referência',       time: '2.1s' },
  { label: 'Plano de energia — alto desemp.', time: '0.1s' },
  { label: 'Desabilitando Xbox DVR',         time: '0.0s' },
  { label: 'Benchmark pós-otimização',       time: '2.2s' },
];

// Scene 04 — 180 frames = 6s
// AUTO PILOT: Optimization sequence executes. Proof of automation.
export const ShortAutoPilot: React.FC = () => {
  const frame = useCurrentFrame();
  useVideoConfig();

  const fadeIn = interpolate(frame, [0, 8], [0, 1], { extrapolateRight: 'clamp' });
  const fadeOut = interpolate(frame, [165, 180], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  // Label + card entrance
  const cardP = interpolate(frame, [5, 26], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // Execution progress (frames 26-150)
  const exP = interpolate(frame, [26, 150], [0, 1], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
    easing: Easing.bezier(0.1, 0.3, 0.9, 1),
  });

  // "Done" state
  const doneP = interpolate(frame, [150, 165], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // Spinner rotation
  const spin = frame * 9;

  return (
    <AbsoluteFill style={{ opacity: fadeIn * fadeOut }}>
      <Background />

      <AbsoluteFill style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '192px 88px 288px',
        gap: 24,
      }}>

        {/* Section label */}
        <div style={{
          opacity: cardP,
          transform: `translateY(${(1 - cardP) * 14}px)`,
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
            Central de Otimizações
          </div>
          <div style={{
            fontFamily: font,
            fontSize: 46,
            fontWeight: 700,
            color: C.inkHi,
            letterSpacing: '-0.025em',
            lineHeight: 1.1,
          }}>
            Otimize com confiança.
          </div>
          <div style={{
            fontFamily: font,
            fontSize: 46,
            fontWeight: 700,
            color: C.signal,
            letterSpacing: '-0.025em',
            lineHeight: 1.1,
          }}>
            Reverta em 1 clique.
          </div>
        </div>

        {/* Card */}
        <div style={{
          width: '100%',
          opacity: cardP,
          transform: `translateY(${(1 - cardP) * 22}px)`,
          background: C.panel,
          border: `1px solid ${C.hairline}`,
          borderRadius: 20,
          padding: '28px 28px',
        }}>
          {/* Card header */}
          <div style={{
            display: 'flex',
            gap: 16,
            alignItems: 'center',
            marginBottom: 26,
          }}>
            <div style={{ fontSize: 36 }}>⚡</div>
            <div style={{ flex: 1 }}>
              <div style={{
                fontFamily: font,
                fontSize: 28,
                fontWeight: 700,
                color: C.inkHi,
              }}>
                Auto Pilot
              </div>
              <div style={{
                fontFamily: font,
                fontSize: 20,
                color: C.inkMid,
                marginTop: 2,
              }}>
                Aplicando otimizações seguras
              </div>
            </div>
            {/* Spinner / Done */}
            {doneP < 0.5 ? (
              <div style={{
                width: 30, height: 30,
                borderRadius: '50%',
                border: `3px solid ${C.raised}`,
                borderTop: `3px solid ${C.signal}`,
                transform: `rotate(${spin}deg)`,
                flexShrink: 0,
              }} />
            ) : (
              <div style={{
                width: 30, height: 30,
                borderRadius: '50%',
                background: `${C.ok}22`,
                border: `2px solid ${C.ok}`,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                flexShrink: 0,
                fontFamily: fontMono,
                fontSize: 16,
                color: C.ok,
              }}>
                ✓
              </div>
            )}
          </div>

          {/* Steps */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            {STEPS.map((step, i) => {
              const done   = exP > (i + 1) / STEPS.length;
              const active = exP > i / STEPS.length && !done;
              const vis    = exP > (i / STEPS.length - 0.06) ? 1 : 0.22;
              return (
                <div key={i} style={{
                  display: 'flex',
                  gap: 14,
                  alignItems: 'center',
                  opacity: vis,
                }}>
                  {/* Status dot */}
                  <div style={{
                    width: 22, height: 22,
                    borderRadius: '50%',
                    border: `2px solid ${done ? C.ok : active ? C.signal : C.hairline}`,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    flexShrink: 0,
                    background: done ? `${C.ok}1a` : 'transparent',
                    boxShadow: active ? `0 0 10px ${C.signal}` : 'none',
                  }}>
                    {done && (
                      <span style={{ fontSize: 11, color: C.ok, lineHeight: 1 }}>✓</span>
                    )}
                    {active && (
                      <div style={{
                        width: 7, height: 7,
                        borderRadius: '50%',
                        background: C.signal,
                        boxShadow: `0 0 6px ${C.signal}`,
                      }} />
                    )}
                  </div>
                  <span style={{
                    fontFamily: font,
                    fontSize: 22,
                    flex: 1,
                    color: done ? C.ok : active ? C.inkHi : C.inkLow,
                  }}>
                    {step.label}
                  </span>
                  {done && (
                    <span style={{
                      fontFamily: fontMono,
                      fontSize: 18,
                      color: C.inkLow,
                    }}>
                      {step.time}
                    </span>
                  )}
                </div>
              );
            })}
          </div>

          {/* Progress bar */}
          <div style={{ marginTop: 22 }}>
            <div style={{
              height: 6,
              background: C.raised,
              borderRadius: 3,
              overflow: 'hidden',
            }}>
              <div style={{
                height: '100%',
                width: `${exP * 100}%`,
                background: `linear-gradient(90deg, ${C.signal}, ${C.ion})`,
                borderRadius: 3,
                boxShadow: exP < 1 ? `0 0 12px ${C.signal}` : 'none',
              }} />
            </div>
            <div style={{
              fontFamily: fontMono,
              fontSize: 20,
              color: C.inkLow,
              marginTop: 8,
              textAlign: 'right',
            }}>
              {Math.round(exP * 100)}% concluído
            </div>
          </div>
        </div>

        {/* Done badge */}
        <div style={{
          opacity: doneP,
          transform: `translateY(${(1 - doneP) * 18}px) scale(${0.88 + doneP * 0.12})`,
          display: 'flex',
          gap: 14,
          alignItems: 'center',
          background: `${C.ok}16`,
          border: `1.5px solid ${C.ok}50`,
          borderRadius: 16,
          padding: '20px 28px',
          width: '100%',
        }}>
          <div style={{ fontSize: 30 }}>✅</div>
          <div>
            <div style={{
              fontFamily: font,
              fontSize: 26,
              fontWeight: 700,
              color: C.ok,
            }}>
              4 otimizações mantidas
            </div>
            <div style={{
              fontFamily: font,
              fontSize: 18,
              color: C.inkMid,
              marginTop: 3,
            }}>
              Snapshots preservados · reversível a qualquer hora
            </div>
          </div>
        </div>

      </AbsoluteFill>
    </AbsoluteFill>
  );
};
