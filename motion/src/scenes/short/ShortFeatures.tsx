import React from 'react';
import {
  AbsoluteFill, useCurrentFrame, useVideoConfig,
  interpolate, Easing,
} from 'remotion';
import { C, SETTLE } from '../../tokens';
import { font, fontMono } from '../../fonts';
import { Background } from '../../components/Background';

const FEATURES = [
  {
    icon: '🎮',
    title: 'Game Center',
    desc: 'Perfil automático ao iniciar. Competitivo, Streaming, Power Save.',
    color: C.warn,
    metric: '4 perfis',
  },
  {
    icon: '🛡',
    title: 'Rollback',
    desc: 'Snapshot antes de cada mudança. Reverta qualquer tweak em 1 clique.',
    color: C.signal,
    metric: '100% reversível',
  },
  {
    icon: '📈',
    title: 'Monitor ao Vivo',
    desc: 'CPU, GPU, RAM e temperatura em tempo real. FPS durante gameplay.',
    color: C.ok,
    metric: 'Tempo real',
  },
  {
    icon: '🔋',
    title: 'RAM Flush',
    desc: 'Recupere GBs de RAM travada instantaneamente. Sem fechar nada.',
    color: C.ion,
    metric: '+2 GB livres',
  },
];

// Each card stays "active" for ~40 frames, slides in at 20-frame intervals
const INTERVAL = 40;

// Scene 06 — 195 frames = 6.5s
// FEATURES: Rapid-fire showcase of key modules.
export const ShortFeatures: React.FC = () => {
  const frame = useCurrentFrame();
  useVideoConfig();

  const fadeIn = interpolate(frame, [0, 8], [0, 1], { extrapolateRight: 'clamp' });
  const fadeOut = interpolate(frame, [180, 195], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  // Header
  const headerP = interpolate(frame, [5, 24], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  // Stats row at end
  const statsP = interpolate(frame, [168, 184], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  return (
    <AbsoluteFill style={{ opacity: fadeIn * fadeOut }}>
      <Background />

      <AbsoluteFill style={{
        display: 'flex',
        flexDirection: 'column',
        padding: '192px 88px 288px',
        gap: 24,
      }}>

        {/* Header */}
        <div style={{
          opacity: headerP,
          transform: `translateY(${(1 - headerP) * 14}px)`,
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
            Plataforma completa
          </div>
          <div style={{
            fontFamily: font,
            fontSize: 56,
            fontWeight: 700,
            color: C.inkHi,
            letterSpacing: '-0.025em',
            lineHeight: 1.05,
          }}>
            Tudo integrado.
            <span style={{ color: C.ion }}> Uma app.</span>
          </div>
        </div>

        {/* Feature cards — sequential animated reveal */}
        <div style={{
          flex: 1,
          display: 'flex',
          flexDirection: 'column',
          gap: 18,
        }}>
          {FEATURES.map((f, i) => {
            const startFrame = 24 + i * INTERVAL;
            const cardP = interpolate(
              frame,
              [startFrame, startFrame + 20],
              [0, 1],
              { extrapolateRight: 'clamp', easing: Easing.bezier(...SETTLE) }
            );

            // "Active" window — highlight the card while it's the newest
            const isActive = frame >= startFrame + 20 && frame < startFrame + INTERVAL + 10;
            const brightness = isActive ? 1.0 : 0.55;

            return (
              <div key={i} style={{
                opacity: cardP * brightness,
                transform: `translateX(${(1 - cardP) * -30}px) scale(${isActive ? 1 : 0.975})`,
                background: isActive ? `${f.color}10` : C.panel,
                border: `1.5px solid ${isActive ? f.color + '55' : C.hairline}`,
                borderRadius: 18,
                padding: '22px 24px',
                display: 'flex',
                gap: 20,
                alignItems: 'center',
                // Active left accent
                borderLeft: isActive ? `3px solid ${f.color}` : `1.5px solid ${C.hairline}`,
              }}>
                {/* Icon */}
                <div style={{
                  fontSize: 42,
                  lineHeight: 1,
                  filter: isActive ? `drop-shadow(0 0 14px ${f.color})` : 'none',
                }}>
                  {f.icon}
                </div>

                {/* Text */}
                <div style={{ flex: 1 }}>
                  <div style={{
                    fontFamily: font,
                    fontSize: 26,
                    fontWeight: 700,
                    color: isActive ? f.color : C.inkHi,
                    marginBottom: 4,
                    lineHeight: 1,
                  }}>
                    {f.title}
                  </div>
                  <div style={{
                    fontFamily: font,
                    fontSize: 20,
                    color: C.inkMid,
                    lineHeight: 1.35,
                  }}>
                    {f.desc}
                  </div>
                </div>

                {/* Metric badge */}
                <div style={{
                  background: isActive ? `${f.color}1a` : `${C.raised}`,
                  border: `1px solid ${isActive ? f.color + '44' : C.hairline}`,
                  borderRadius: 10,
                  padding: '8px 14px',
                  fontFamily: font,
                  fontSize: 18,
                  fontWeight: 700,
                  color: isActive ? f.color : C.inkLow,
                  whiteSpace: 'nowrap',
                }}>
                  {f.metric}
                </div>
              </div>
            );
          })}
        </div>

        {/* Bottom stats bar */}
        <div style={{
          opacity: statsP,
          transform: `translateY(${(1 - statsP) * 14}px)`,
          display: 'flex',
          background: C.panel,
          border: `1px solid ${C.hairline}`,
          borderRadius: 16,
          overflow: 'hidden',
        }}>
          {[
            { value: '40+',   label: 'Otimizações',   color: C.signal },
            { value: '100%',  label: 'Reversível',    color: C.ok     },
            { value: '0 B',   label: 'Dados enviados', color: C.ion   },
          ].map((stat, i) => (
            <div key={i} style={{
              flex: 1,
              padding: '18px 0',
              textAlign: 'center',
              borderRight: i < 2 ? `1px solid ${C.hairline}` : 'none',
            }}>
              <div style={{
                fontFamily: fontMono,
                fontSize: 32,
                fontWeight: 700,
                color: stat.color,
                textShadow: `0 0 18px ${stat.color}50`,
                lineHeight: 1,
              }}>
                {stat.value}
              </div>
              <div style={{
                fontFamily: font,
                fontSize: 17,
                color: C.inkMid,
                marginTop: 5,
              }}>
                {stat.label}
              </div>
            </div>
          ))}
        </div>

      </AbsoluteFill>
    </AbsoluteFill>
  );
};
