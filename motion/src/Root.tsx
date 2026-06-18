import React from 'react';
import { Composition } from "remotion";
import { TkSpeedVideo, TOTAL_FRAMES } from "./TkSpeedVideo";
import { TkSpeedShort, SHORT_FRAMES } from "./TkSpeedShort";

export const RemotionRoot: React.FC = () => {
  return (
    <>
      {/* Landscape 16:9 — full 90s version */}
      <Composition
        id="TkSpeedMotion"
        component={TkSpeedVideo}
        durationInFrames={TOTAL_FRAMES}
        fps={30}
        width={1920}
        height={1080}
      />

      {/* Vertical 9:16 — 35s social short (Reels / TikTok / Shorts) */}
      <Composition
        id="TkSpeedShort"
        component={TkSpeedShort}
        durationInFrames={SHORT_FRAMES}
        fps={30}
        width={1080}
        height={1920}
      />
    </>
  );
};
