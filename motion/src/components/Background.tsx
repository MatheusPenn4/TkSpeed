import { AbsoluteFill, useCurrentFrame, interpolate } from 'remotion';
import { C } from '../tokens';

export const Background: React.FC<{ pulse?: boolean }> = ({ pulse = false }) => {
  const frame = useCurrentFrame();

  const glowOpacity = pulse
    ? interpolate(Math.sin(frame / 45), [-1, 1], [0.03, 0.09])
    : 0.05;

  return (
    <AbsoluteFill style={{ background: C.void, overflow: 'hidden' }}>
      {/* CSS grid — resolution-agnostic */}
      <AbsoluteFill style={{
        opacity: 0.018,
        backgroundImage: `
          linear-gradient(${C.signal} 0.5px, transparent 0.5px),
          linear-gradient(90deg, ${C.signal} 0.5px, transparent 0.5px)
        `,
        backgroundSize: '80px 80px',
      }} />

      {/* Center radial glow */}
      <div style={{
        position: 'absolute',
        top: '50%',
        left: '50%',
        transform: 'translate(-50%, -50%)',
        width: 1000,
        height: 1000,
        borderRadius: '50%',
        background: `radial-gradient(ellipse, rgba(88,242,210,${glowOpacity}) 0%, transparent 70%)`,
        pointerEvents: 'none',
      }} />

      {/* Top-right accent glow */}
      <div style={{
        position: 'absolute',
        top: -200,
        right: -200,
        width: 700,
        height: 700,
        borderRadius: '50%',
        background: `radial-gradient(ellipse, rgba(57,199,255,0.04) 0%, transparent 70%)`,
        pointerEvents: 'none',
      }} />
    </AbsoluteFill>
  );
};
