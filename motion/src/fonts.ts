import { loadFont as loadInter } from '@remotion/google-fonts/Inter';
import { loadFont as loadJetBrains } from '@remotion/google-fonts/JetBrainsMono';

const { fontFamily: interFamily } = loadInter();
const { fontFamily: monoFamily } = loadJetBrains();

export const font = interFamily;
export const fontMono = monoFamily;
