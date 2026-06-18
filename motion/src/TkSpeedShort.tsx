import React from 'react';
import { AbsoluteFill, Sequence } from 'remotion';
import { ShortHook } from './scenes/short/ShortHook';
import { ShortIntro } from './scenes/short/ShortIntro';
import { ShortDashboard } from './scenes/short/ShortDashboard';
import { ShortAutoPilot } from './scenes/short/ShortAutoPilot';
import { ShortProof } from './scenes/short/ShortProof';
import { ShortFeatures } from './scenes/short/ShortFeatures';
import { ShortOutro } from './scenes/short/ShortOutro';

// Scene durations at 30fps — total 35s
const S01 = 90;   // 3.0s  — Hook: BAIXO FPS?
const S02 = 105;  // 3.5s  — Intro: Logo reveal
const S03 = 195;  // 6.5s  — Dashboard: Score arc + bottlenecks
const S04 = 180;  // 6.0s  — Auto Pilot: Optimization execution
const S05 = 165;  // 5.5s  — Proof: Before/after numbers
const S06 = 195;  // 6.5s  — Features: Rapid-fire showcase
const S07 = 120;  // 4.0s  — Outro: Logo + CTA

const t01 = 0;
const t02 = t01 + S01;
const t03 = t02 + S02;
const t04 = t03 + S03;
const t05 = t04 + S04;
const t06 = t05 + S05;
const t07 = t06 + S06;

export const SHORT_FRAMES = t07 + S07; // 1050 = 35s

export const TkSpeedShort: React.FC = () => {
  return (
    <AbsoluteFill>
      <Sequence from={t01} durationInFrames={S01}>
        <ShortHook />
      </Sequence>
      <Sequence from={t02} durationInFrames={S02}>
        <ShortIntro />
      </Sequence>
      <Sequence from={t03} durationInFrames={S03}>
        <ShortDashboard />
      </Sequence>
      <Sequence from={t04} durationInFrames={S04}>
        <ShortAutoPilot />
      </Sequence>
      <Sequence from={t05} durationInFrames={S05}>
        <ShortProof />
      </Sequence>
      <Sequence from={t06} durationInFrames={S06}>
        <ShortFeatures />
      </Sequence>
      <Sequence from={t07} durationInFrames={S07}>
        <ShortOutro />
      </Sequence>
    </AbsoluteFill>
  );
};
