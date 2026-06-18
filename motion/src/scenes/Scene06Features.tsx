import React from 'react';
import {
  AbsoluteFill, useCurrentFrame, useVideoConfig,
  interpolate, Easing,
} from 'remotion';
import { C, SETTLE } from '../tokens';
import { font, fontMono } from '../fonts';
import { Background } from '../components/Background';
import { FeatureCard } from '../components/FeatureCard';


const FEATURES = [
  {
    icon: '🚀',
    title: 'Gerenciador de Inicialização',
    desc: 'Controle o que inicia com o Windows. Nomes amigáveis, impacto estimado, RAM liberada por app.',
    color: C.ion,
  },
  {
    icon: '🛡',
    title: 'Central de Restauração',
    desc: 'Snapshot antes de cada mudança. Reverta qualquer otimização em um clique, com histórico completo.',
    color: C.signal,
  },
  {
    icon: '🎮',
    title: 'Game Center',
    desc: 'Perfis por jogo: Competitivo, Balanceado, Streaming. Game Boost ativa ao iniciar e reverte ao fechar.',
    color: C.warn,
  },
  {
    icon: '📈',
    title: 'Monitor em Tempo Real',
    desc: 'CPU, GPU, RAM, temperatura ao vivo. Detecção de processos pesados. FPS durante sessões de jogo.',
    color: C.ok,
  },
  {
    icon: '🔬',
    title: 'Gerenciador de Memória',
    desc: 'Memory Arc visual. Flush de cache, histórico de comprometimento, correlação com FPS.',
    color: C.info,
  },
  {
    icon: '📊',
    title: 'Histórico & Evolução',
    desc: 'TkSpeed Score ao longo do tempo. Digital Twin — rastreaamento completo da sua máquina.',
    color: C.signal,
  },
];

// Scene: 600 frames = 20s
export const Scene06Features: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const fadeIn = interpolate(frame, [0, fps * 0.5], [0, 1], { extrapolateRight: 'clamp' });
  const fadeOut = interpolate(frame, [fps * 18.5, fps * 20], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  const headerP = interpolate(frame, [fps * 0.2, fps * 1.2], [0, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  return (
    <AbsoluteFill style={{ opacity: fadeIn * fadeOut }}>
      <Background />

      <AbsoluteFill style={{
        display: 'flex', flexDirection: 'column',
        padding: '55px 90px', gap: 32,
      }}>
        {/* Header */}
        <div style={{
          opacity: headerP,
          transform: `translateY(${(1 - headerP) * 16}px)`,
        }}>
          <div style={{
            fontFamily: font, fontSize: 13, fontWeight: 600,
            color: C.signal, letterSpacing: '0.1em',
            textTransform: 'uppercase', marginBottom: 8,
          }}>
            TkSpeed — Plataforma Completa
          </div>
          <div style={{
            fontFamily: font, fontSize: 38, fontWeight: 700,
            color: C.inkHi, letterSpacing: '-0.02em', lineHeight: 1.1,
          }}>
            Controle total do seu PC.
            <span style={{ color: C.ion }}> Tudo integrado.</span>
          </div>
        </div>

        {/* Feature grid */}
        <div style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(3, 1fr)',
          gap: 16,
          flex: 1,
        }}>
          {FEATURES.map((f, i) => {
            const delay = i * 0.12;
            const cardP = interpolate(
              frame,
              [fps * (0.8 + delay), fps * (1.7 + delay)],
              [0, 1],
              { extrapolateRight: 'clamp', easing: Easing.bezier(...SETTLE) }
            );
            return (
              <FeatureCard
                key={i}
                icon={f.icon}
                title={f.title}
                desc={f.desc}
                accentColor={f.color}
                progress={cardP}
              />
            );
          })}
        </div>

        {/* Bottom stats */}
        {(() => {
          const statsP = interpolate(frame, [fps * 3.5, fps * 4.5], [0, 1], {
            extrapolateRight: 'clamp',
            easing: Easing.bezier(...SETTLE),
          });
          return (
            <div style={{
              opacity: statsP,
              transform: `translateY(${(1 - statsP) * 12}px)`,
              display: 'flex', gap: 32,
              background: C.panel,
              border: `1px solid ${C.hairline}`,
              borderRadius: 12, padding: '18px 28px',
              alignItems: 'center',
            }}>
              {[
                { value: '40+', label: 'Otimizações no catálogo', color: C.signal },
                { value: '100%', label: 'Reversível por design', color: C.ok },
                { value: '0', label: 'Dados enviados para servidores', color: C.ion },
                { value: 'Rust', label: 'Backend de alta performance', color: C.warn },
              ].map((stat, i) => (
                <div key={i} style={{
                  display: 'flex', flexDirection: 'column', gap: 3,
                  borderRight: i < 3 ? `1px solid ${C.hairline}` : 'none',
                  paddingRight: i < 3 ? 32 : 0,
                }}>
                  <div style={{
                    fontFamily: fontMono, fontSize: 26, fontWeight: 700,
                    color: stat.color,
                    textShadow: `0 0 20px ${stat.color}`,
                  }}>
                    {stat.value}
                  </div>
                  <div style={{
                    fontFamily: font, fontSize: 12, color: C.inkMid,
                    maxWidth: 140,
                  }}>
                    {stat.label}
                  </div>
                </div>
              ))}
            </div>
          );
        })()}
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
