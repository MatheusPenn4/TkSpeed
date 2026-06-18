import React from 'react';
import {
  AbsoluteFill, useCurrentFrame, useVideoConfig,
  interpolate, Easing, Img, staticFile,
} from 'remotion';
import { C, SETTLE } from '../tokens';
import { font, fontMono } from '../fonts';
import { Background } from '../components/Background';

// Scene duration: 150 frames = 5s
export const Scene01Intro: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const symbolScale = interpolate(frame, [0, fps * 1.2], [0.4, 1], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  const symbolOpacity = interpolate(frame, [0, fps * 0.6], [0, 1], {
    extrapolateRight: 'clamp',
  });

  const ringScale = interpolate(frame, [fps * 0.3, fps * 1.5], [0.6, 1.15], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  const ringOpacity = interpolate(frame, [fps * 0.3, fps * 0.9, fps * 1.4, fps * 1.7], [0, 0.35, 0.35, 0], {
    extrapolateRight: 'clamp',
  });

  const ring2Scale = interpolate(frame, [fps * 0.6, fps * 2], [0.6, 1.4], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(0.2, 0.8, 0.4, 1),
  });

  const ring2Opacity = interpolate(frame, [fps * 0.6, fps * 1.2, fps * 1.8, fps * 2.2], [0, 0.2, 0.2, 0], {
    extrapolateRight: 'clamp',
  });

  const wordmarkY = interpolate(frame, [fps * 0.7, fps * 1.4], [20, 0], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  const wordmarkOpacity = interpolate(frame, [fps * 0.7, fps * 1.3], [0, 1], {
    extrapolateRight: 'clamp',
  });

  const taglineY = interpolate(frame, [fps * 1.1, fps * 1.8], [14, 0], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  const taglineOpacity = interpolate(frame, [fps * 1.1, fps * 1.7], [0, 1], {
    extrapolateRight: 'clamp',
  });

  const barWidth = interpolate(frame, [fps * 1.5, fps * 2.5], [0, 160], {
    extrapolateRight: 'clamp',
    easing: Easing.bezier(...SETTLE),
  });

  const barOpacity = interpolate(frame, [fps * 1.5, fps * 1.9], [0, 1], {
    extrapolateRight: 'clamp',
  });

  // Fade out at end
  const sceneOpacity = interpolate(frame, [fps * 4.2, fps * 5], [1, 0], {
    extrapolateLeft: 'clamp', extrapolateRight: 'clamp',
  });

  return (
    <AbsoluteFill style={{ opacity: sceneOpacity }}>
      <Background pulse />

      <AbsoluteFill style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', flexDirection: 'column', gap: 0 }}>

        {/* Outer ring */}
        <div style={{
          position: 'absolute',
          width: 320, height: 320,
          borderRadius: '50%',
          border: `1.5px solid ${C.signal}`,
          transform: `scale(${ring2Scale})`,
          opacity: ring2Opacity,
          left: '50%', top: '50%',
          marginLeft: -160, marginTop: -160,
        }} />

        {/* Inner ring */}
        <div style={{
          position: 'absolute',
          width: 220, height: 220,
          borderRadius: '50%',
          border: `1px solid ${C.signal}`,
          transform: `scale(${ringScale})`,
          opacity: ringOpacity,
          left: '50%', top: '50%',
          marginLeft: -110, marginTop: -200,
        }} />

        {/* Symbol */}
        <div style={{
          transform: `scale(${symbolScale})`,
          opacity: symbolOpacity,
          marginBottom: 28,
          filter: `drop-shadow(0 0 28px ${C.signal})`,
        }}>
          <Img
            src={staticFile('symbol.png')}
            style={{ width: 76, height: 'auto', objectFit: 'contain' }}
          />
        </div>

        {/* Wordmark */}
        <div style={{
          transform: `translateY(${wordmarkY}px)`,
          opacity: wordmarkOpacity,
          display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 6,
        }}>
          <div style={{
            fontFamily: font, fontSize: 42, fontWeight: 700,
            color: C.inkHi, letterSpacing: '-0.02em', lineHeight: 1,
          }}>
            TkSpeed
          </div>

          {/* Tagline */}
          <div style={{
            transform: `translateY(${taglineY}px)`,
            opacity: taglineOpacity,
            fontFamily: font, fontSize: 15, fontWeight: 500,
            color: C.inkMid, letterSpacing: '0.12em',
            textTransform: 'uppercase',
          }}>
            Engenharia de Performance
          </div>
        </div>

        {/* Progress bar */}
        <div style={{
          marginTop: 48,
          opacity: barOpacity,
          width: 160, height: 2,
          borderRadius: 1,
          background: C.raised,
          overflow: 'hidden',
        }}>
          <div style={{
            height: '100%',
            width: barWidth,
            background: `linear-gradient(90deg, ${C.signal}, ${C.ion})`,
            borderRadius: 1,
          }} />
        </div>

        {/* Status */}
        <div style={{
          marginTop: 12,
          opacity: barOpacity * 0.6,
          fontFamily: fontMono, fontSize: 11,
          color: C.inkLow, letterSpacing: '0.08em',
        }}>
          inicializando
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
