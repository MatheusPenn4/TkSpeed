import React from 'react';
import {
  AbsoluteFill, useCurrentFrame, useVideoConfig,
  interpolate, Easing,
} from 'remotion';
import { C, SETTLE } from '../tokens';
import { font, fontMono } from '../fonts';
import { Background } from '../components/Background';
import { EvidenceCard } from '../components/EvidenceCard';

const STEPS = [
  { label: 'Criando ponto de restauração', status: 'done', time: '0.3s' },
  { label: 'Benchmark de referência', status: 'done', time: '2.1s' },
  { label: 'Aplicando: Plano de energia alto desempenho', status: 'done', time: '0.1s' },
  { label: 'Aplicando: Desabilitando Game Bar', status: 'done', time: '0.0s' },
  { label: 'Aplicando: Limpeza de arquivos temporários', status: 'done', time: '1.8s' },
  { label: 'Benchmark pós-otimização', status: 'done', time: '2.2s' },
  { label: 'Comparando resultados', status: 'done', time: '0.2s' },
];

const beforeTrace = [8200, 8150, 8320, 8100, 8280, 8190, 8240, 8160, 8310, 8210, 8170, 8290, 8220, 8140, 8260, 8190, 8310, 8180, 8250, 8220];
const afterTrace = [9100, 9250, 9180, 9320, 9270, 9150, 9410, 9360, 9290, 9440, 9380, 9310, 9420, 9350, 9280, 9460, 9390, 9320, 9410, 9480];

// Scene: 600 frames = 20s
export const Scene05Optimization: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const fadeIn = interpolate(frame, [0, fps * 0.5], [0, 1], { extrapolateRight: 'clamp' });
  const fadeOut = interpolate(frame, [fps * 18.5, fps * 20], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  // Phase 1: 0–6s — catalog intro
  // Phase 2: 6–13s — auto pilot execution
  // Phase 3: 13–20s — evidence card result

  const PHASE2_START = fps * 6;
  const PHASE3_START = fps * 13;

  const showPhase1 = interpolate(frame, [0, PHASE2_START - fps * 0.5, PHASE2_START], [1, 1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  const showPhase2 = interpolate(frame, [PHASE2_START - fps * 0.3, PHASE2_START + fps * 0.5], [0, 1], {
    extrapolateRight: 'clamp',
  });

  const showPhase3 = interpolate(frame, [PHASE3_START, PHASE3_START + fps * 0.6], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  const executionProgress = interpolate(frame, [PHASE2_START + fps * 0.5, PHASE3_START - fps * 0.5], [0, 1], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
    easing: Easing.bezier(0.1, 0.3, 0.9, 1),
  });

  const evidenceProgress = interpolate(frame, [PHASE3_START + fps * 0.5, PHASE3_START + fps * 2.5], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  return (
    <AbsoluteFill style={{ opacity: fadeIn * fadeOut }}>
      <Background />

      {/* ——————————————————————————————————————————
          PHASE 1: Catalog / CTA
      —————————————————————————————————————————— */}
      <AbsoluteFill style={{
        opacity: showPhase1,
        display: 'flex', flexDirection: 'column',
        padding: '60px 100px', gap: 28,
      }}>
        {(() => {
          const p = interpolate(frame, [fps * 0.2, fps * 1.2], [0, 1], {
            extrapolateRight: 'clamp',
            easing: Easing.bezier(...SETTLE),
          });
          return (
            <div style={{ opacity: p, transform: `translateY(${(1 - p) * 16}px)` }}>
              <div style={{
                fontFamily: font, fontSize: 13, fontWeight: 600,
                color: C.signal, letterSpacing: '0.1em',
                textTransform: 'uppercase', marginBottom: 8,
              }}>
                Central de Otimizações
              </div>
              <div style={{
                fontFamily: font, fontSize: 40, fontWeight: 700,
                color: C.inkHi, letterSpacing: '-0.025em', lineHeight: 1.1,
              }}>
                Otimize com confiança.
              </div>
              <div style={{
                fontFamily: font, fontSize: 40, fontWeight: 700,
                color: C.signal, letterSpacing: '-0.025em', lineHeight: 1.1,
              }}>
                Reverta em um clique.
              </div>
              <div style={{
                fontFamily: font, fontSize: 18, color: C.inkMid,
                marginTop: 12, lineHeight: 1.5, maxWidth: 560,
              }}>
                Cada otimização é medida antes e depois.
                Revertida automaticamente se não houver ganho real.
              </div>
            </div>
          );
        })()}

        {/* Catalog rows */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 10, maxWidth: 700 }}>
          {[
            { name: 'Plano de energia — alto desempenho', category: 'Moderada', gain: '+8–15%', critical: true },
            { name: 'Desabilitar Game Bar & Xbox DVR', category: 'Segura', gain: '+3–6%', critical: false },
            { name: 'Limpeza de arquivos temporários', category: 'Segura', gain: '2–8 GB', critical: false },
            { name: 'Otimizar inicialização', category: 'Segura', gain: '+12% boot', critical: false },
          ].map((opt, i) => {
            const p = interpolate(
              frame,
              [fps * (1 + i * 0.2), fps * (1.8 + i * 0.2)],
              [0, 1],
              { extrapolateRight: 'clamp', easing: Easing.bezier(...SETTLE) }
            );
            return (
              <div key={i} style={{
                background: C.panel,
                border: `1px solid ${opt.critical ? C.signal + '33' : C.hairline}`,
                borderRadius: 10, padding: '12px 18px',
                display: 'flex', alignItems: 'center', gap: 16,
                opacity: p,
                transform: `translateX(${(1 - p) * -18}px)`,
              }}>
                <div style={{ flex: 1 }}>
                  <div style={{ fontFamily: font, fontSize: 14, fontWeight: 600, color: C.inkHi }}>
                    {opt.name}
                  </div>
                  <div style={{ fontFamily: font, fontSize: 12, color: C.inkLow, marginTop: 2 }}>
                    Ganho típico: <span style={{ color: C.ion }}>{opt.gain}</span>
                  </div>
                </div>
                <div style={{
                  background: opt.critical ? `${C.signal}1a` : `${C.ok}1a`,
                  border: `1px solid ${opt.critical ? C.signal + '44' : C.ok + '44'}`,
                  borderRadius: 6, padding: '3px 8px',
                  fontFamily: font, fontSize: 11, fontWeight: 600,
                  color: opt.critical ? C.signal : C.ok,
                }}>
                  {opt.category}
                </div>
                <div style={{
                  background: `${C.ion}1a`,
                  border: `1px solid ${C.ion}44`,
                  borderRadius: 6, padding: '5px 12px',
                  fontFamily: font, fontSize: 12, fontWeight: 700,
                  color: C.ion, cursor: 'pointer',
                }}>
                  Aplicar
                </div>
              </div>
            );
          })}
        </div>

        {/* CTA Auto Pilot */}
        {(() => {
          const p = interpolate(frame, [fps * 2.5, fps * 3.5], [0, 1], {
            extrapolateRight: 'clamp',
            easing: Easing.bezier(...SETTLE),
          });
          const glow = interpolate(frame % (fps * 2), [0, fps, fps * 2], [0.5, 1, 0.5]);
          return (
            <div style={{
              opacity: p,
              transform: `translateY(${(1 - p) * 12}px)`,
              marginTop: 8,
              display: 'flex', gap: 16, alignItems: 'center',
            }}>
              <div style={{
                background: `linear-gradient(135deg, ${C.signal}, ${C.ion})`,
                borderRadius: 10, padding: '14px 28px',
                fontFamily: font, fontSize: 15, fontWeight: 700,
                color: C.void, cursor: 'pointer',
                boxShadow: `0 0 ${20 * glow}px ${C.signal}60`,
              }}>
                ⚡ Otimizar Agora — Auto Pilot
              </div>
              <div style={{
                fontFamily: font, fontSize: 13, color: C.inkMid,
              }}>
                4 otimizações encontradas · 100% reversível
              </div>
            </div>
          );
        })()}
      </AbsoluteFill>

      {/* ——————————————————————————————————————————
          PHASE 2: Auto Pilot Execution
      —————————————————————————————————————————— */}
      <AbsoluteFill style={{
        opacity: Math.min(showPhase2, showPhase3 < 0.5 ? 1 : 1 - (showPhase3 - 0.5) * 2),
        display: 'flex', alignItems: 'center', justifyContent: 'center',
      }}>
        <div style={{
          background: C.panel,
          border: `1px solid ${C.hairline}`,
          borderRadius: 16, padding: '32px 36px',
          width: 520,
        }}>
          {/* Header */}
          <div style={{ display: 'flex', gap: 12, alignItems: 'center', marginBottom: 24 }}>
            <div style={{ fontSize: 24 }}>⚡</div>
            <div>
              <div style={{
                fontFamily: font, fontSize: 18, fontWeight: 700, color: C.inkHi,
              }}>Auto Pilot</div>
              <div style={{
                fontFamily: font, fontSize: 13, color: C.inkMid,
              }}>Executando otimizações seguras</div>
            </div>
            {/* Spinner */}
            <div style={{
              marginLeft: 'auto',
              width: 22, height: 22,
              borderRadius: '50%',
              border: `2px solid ${C.raised}`,
              borderTop: `2px solid ${C.signal}`,
              transform: `rotate(${frame * 8}deg)`,
            }} />
          </div>

          {/* Steps */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            {STEPS.map((step, i) => {
              const stepDone = executionProgress > (i + 1) / STEPS.length;
              const stepActive = executionProgress > i / STEPS.length && !stepDone;
              const stepOpacity = executionProgress > (i / STEPS.length - 0.1) ? 1 : 0.3;

              return (
                <div key={i} style={{
                  display: 'flex', gap: 12, alignItems: 'center',
                  opacity: stepOpacity,
                }}>
                  <div style={{
                    width: 20, height: 20, borderRadius: '50%',
                    border: `2px solid ${stepDone ? C.ok : stepActive ? C.signal : C.hairline}`,
                    display: 'flex', alignItems: 'center', justifyContent: 'center',
                    flexShrink: 0,
                    boxShadow: stepActive ? `0 0 8px ${C.signal}` : 'none',
                  }}>
                    {stepDone && (
                      <div style={{
                        width: 8, height: 8, borderRadius: '50%',
                        background: C.ok,
                      }} />
                    )}
                    {stepActive && (
                      <div style={{
                        width: 8, height: 8, borderRadius: '50%',
                        background: C.signal,
                        boxShadow: `0 0 6px ${C.signal}`,
                      }} />
                    )}
                  </div>
                  <span style={{
                    fontFamily: font, fontSize: 13,
                    color: stepDone ? C.ok : stepActive ? C.inkHi : C.inkLow,
                    flex: 1,
                  }}>
                    {step.label}
                  </span>
                  {stepDone && (
                    <span style={{ fontFamily: fontMono, fontSize: 11, color: C.inkLow }}>
                      {step.time}
                    </span>
                  )}
                </div>
              );
            })}
          </div>

          {/* Overall progress */}
          <div style={{ marginTop: 20 }}>
            <div style={{ height: 4, background: C.raised, borderRadius: 2, overflow: 'hidden' }}>
              <div style={{
                height: '100%',
                width: `${executionProgress * 100}%`,
                background: `linear-gradient(90deg, ${C.signal}, ${C.ion})`,
                borderRadius: 2,
              }} />
            </div>
            <div style={{
              fontFamily: fontMono, fontSize: 12, color: C.inkLow,
              marginTop: 6, textAlign: 'right',
            }}>
              {Math.round(executionProgress * 100)}% concluído
            </div>
          </div>
        </div>
      </AbsoluteFill>

      {/* ——————————————————————————————————————————
          PHASE 3: Evidence Card
      —————————————————————————————————————————— */}
      <AbsoluteFill style={{
        opacity: showPhase3,
        display: 'flex', flexDirection: 'column',
        padding: '60px 100px', gap: 32,
      }}>
        <div style={{
          fontFamily: font, fontSize: 13, fontWeight: 600,
          color: C.ok, letterSpacing: '0.1em',
          textTransform: 'uppercase',
        }}>
          Resultado — Ganho Verificado
        </div>

        <div style={{
          fontFamily: font, fontSize: 38, fontWeight: 700,
          color: C.inkHi, letterSpacing: '-0.02em', lineHeight: 1.1,
        }}>
          Prova por medição.
          <span style={{ color: C.signal }}> Não por promessa.</span>
        </div>

        <div style={{ display: 'flex', gap: 24, flexWrap: 'wrap' }}>
          <EvidenceCard
            metric="CPU Multi-thread"
            before={beforeTrace}
            after={afterTrace}
            unit="pts"
            delta={14.8}
            progress={evidenceProgress}
            width={360}
          />
          <EvidenceCard
            metric="RAM Bandwidth"
            before={beforeTrace.map(v => v * 0.0046)}
            after={afterTrace.map(v => v * 0.0046)}
            unit="GB/s"
            delta={11.5}
            progress={Math.max(0, evidenceProgress - 0.2)}
            width={360}
          />
        </div>

        {/* Final badge */}
        {(() => {
          const p = interpolate(frame, [fps * 15.5, fps * 16.5], [0, 1], {
            extrapolateRight: 'clamp',
            easing: Easing.bezier(...SETTLE),
          });
          return (
            <div style={{
              opacity: p,
              transform: `translateY(${(1 - p) * 14}px)`,
              display: 'flex', gap: 20, alignItems: 'center',
            }}>
              <div style={{
                background: `${C.ok}18`,
                border: `1px solid ${C.ok}44`,
                borderRadius: 10, padding: '12px 20px',
                display: 'flex', gap: 12, alignItems: 'center',
              }}>
                <span style={{ fontFamily: fontMono, fontSize: 22, fontWeight: 800, color: C.ok }}>
                  ✓ 4 otimizações mantidas
                </span>
              </div>
              <div style={{
                fontFamily: font, fontSize: 13, color: C.inkMid,
              }}>
                Revertidas automaticamente se sem ganho · snapshots preservados
              </div>
            </div>
          );
        })()}
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
