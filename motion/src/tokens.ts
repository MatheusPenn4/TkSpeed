export const C = {
  void: '#08090c',
  base: '#0d0f14',
  panel: '#13161d',
  raised: '#1a1e27',
  machined: '#222732',
  hairline: '#2a3040',
  signal: '#58f2d2',
  ion: '#39c7ff',
  signalGrad: 'linear-gradient(135deg, #58f2d2 0%, #39c7ff 100%)',
  ok: '#46c88a',
  warn: '#e8b84b',
  risk: '#ef6e6e',
  info: '#6e93ff',
  inkHi: '#f2f5fa',
  inkMid: '#9ba6b7',
  inkLow: '#5a6473',
  inkFaint: '#353c49',
  glowSignal: 'rgba(88,242,210,0.22)',
  glowIon: 'rgba(57,199,255,0.22)',
  glowOk: 'rgba(70,200,138,0.22)',
  glowWarn: 'rgba(232,184,75,0.18)',
  glowRisk: 'rgba(239,110,110,0.22)',
};

export const ease = (t: number) => {
  // cubic-bezier(0.16, 1, 0.3, 1) — Apex settle
  return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
};

export const SETTLE = [0.16, 1, 0.3, 1] as const;

export const scoreColor = (score: number) => {
  if (score >= 900) return C.signal;
  if (score >= 700) return C.signal;
  if (score >= 450) return C.ion;
  if (score >= 200) return C.warn;
  return C.risk;
};

export const scoreLabel = (score: number) => {
  if (score >= 900) return 'Elite';
  if (score >= 700) return 'Excelente';
  if (score >= 450) return 'Bom';
  if (score >= 200) return 'Regular';
  return 'Crítico';
};
