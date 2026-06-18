import React from 'react';
import { AbsoluteFill, Sequence } from 'remotion';
import { Scene01Intro } from './scenes/Scene01Intro';
import { Scene02Problem } from './scenes/Scene02Problem';
import { Scene03MissionControl } from './scenes/Scene03MissionControl';
import { Scene04PerfLab } from './scenes/Scene04PerfLab';
import { Scene05Optimization } from './scenes/Scene05Optimization';
import { Scene06Features } from './scenes/Scene06Features';
import { Scene07Outro } from './scenes/Scene07Outro';

// Scene durations at 30fps
const S01 = 150;  // 5s  — Intro Splash
const S02 = 240;  // 8s  — Problem Statement
const S03 = 450;  // 15s — Mission Control
const S04 = 450;  // 15s — Performance Lab
const S05 = 600;  // 20s — Optimization
const S06 = 600;  // 20s — Features
const S07 = 210;  // 7s  — Outro / CTA

const t01 = 0;
const t02 = t01 + S01;
const t03 = t02 + S02;
const t04 = t03 + S03;
const t05 = t04 + S04;
const t06 = t05 + S05;
const t07 = t06 + S06;

export const TOTAL_FRAMES = t07 + S07; // 2700 = 90s

export const TkSpeedVideo: React.FC = () => {
  return (
    <AbsoluteFill>
      <Sequence from={t01} durationInFrames={S01}>
        <Scene01Intro />
      </Sequence>

      <Sequence from={t02} durationInFrames={S02}>
        <Scene02Problem />
      </Sequence>

      <Sequence from={t03} durationInFrames={S03}>
        <Scene03MissionControl />
      </Sequence>

      <Sequence from={t04} durationInFrames={S04}>
        <Scene04PerfLab />
      </Sequence>

      <Sequence from={t05} durationInFrames={S05}>
        <Scene05Optimization />
      </Sequence>

      <Sequence from={t06} durationInFrames={S06}>
        <Scene06Features />
      </Sequence>

      <Sequence from={t07} durationInFrames={S07}>
        <Scene07Outro />
      </Sequence>
    </AbsoluteFill>
  );
};
