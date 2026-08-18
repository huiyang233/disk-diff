import type { ColorTheme, DiffStatus } from '../types';

export function getDiffColor(
  deltaPercent: number,
  status: DiffStatus,
  theme: ColorTheme = 'stock_cn'
): { bg: string; border: string; text: string; badgeBg: string } {
  const isCn = theme === 'stock_cn';

  // 1. Deleted/Removed items or -100% reduction: Strongest Green (in CN stock mode)
  if (status === 'removed' || deltaPercent <= -99.9) {
    return isCn
      ? {
          bg: 'rgba(16, 185, 129, 0.82)',
          border: 'rgba(52, 211, 153, 0.9)',
          text: '#ffffff',
          badgeBg: 'rgba(5, 150, 105, 0.9)',
        }
      : {
          bg: 'rgba(220, 38, 38, 0.82)',
          border: 'rgba(248, 113, 113, 0.9)',
          text: '#ffffff',
          badgeBg: 'rgba(185, 28, 28, 0.9)',
        };
  }

  // 2. Newly added items or +100% increase: Strongest Red (in CN stock mode)
  if (status === 'added' || deltaPercent >= 99.9) {
    return isCn
      ? {
          bg: 'rgba(220, 38, 38, 0.82)',
          border: 'rgba(248, 113, 113, 0.9)',
          text: '#ffffff',
          badgeBg: 'rgba(185, 28, 28, 0.9)',
        }
      : {
          bg: 'rgba(16, 185, 129, 0.82)',
          border: 'rgba(52, 211, 153, 0.9)',
          text: '#ffffff',
          badgeBg: 'rgba(5, 150, 105, 0.9)',
        };
  }

  // 3. Unchanged or negligible change
  if (Math.abs(deltaPercent) < 0.05) {
    return {
      bg: 'rgba(30, 41, 59, 0.65)',
      border: 'rgba(51, 65, 85, 0.6)',
      text: '#cbd5e1',
      badgeBg: 'rgba(51, 65, 85, 0.4)',
    };
  }

  // 4. Calculate intensity based on percentage change (cap at 100%)
  const absPct = Math.min(Math.abs(deltaPercent), 100);
  const ratio = Math.sqrt(absPct / 100); // Non-linear curve for better visual contrast

  const isIncrease = deltaPercent > 0;

  // In CN stock style: Increase is Red, Decrease is Green
  const useRed = (isIncrease && isCn) || (!isIncrease && !isCn);

  if (useRed) {
    // Red gradient (Increase in CN)
    const alpha = 0.35 + ratio * 0.47;
    return {
      bg: `rgba(220, 38, 38, ${alpha.toFixed(2)})`,
      border: `rgba(248, 113, 113, ${(alpha + 0.1).toFixed(2)})`,
      text: '#ffffff',
      badgeBg: 'rgba(185, 28, 28, 0.85)',
    };
  } else {
    // Green gradient (Decrease in CN)
    const alpha = 0.35 + ratio * 0.47;
    return {
      bg: `rgba(16, 185, 129, ${alpha.toFixed(2)})`,
      border: `rgba(52, 211, 153, ${(alpha + 0.1).toFixed(2)})`,
      text: '#ffffff',
      badgeBg: 'rgba(5, 150, 105, 0.85)',
    };
  }
}

export function getScanColor(index: number, isDir: boolean): { bg: string; border: string; text: string } {
  if (!isDir) {
    return {
      bg: 'rgba(39, 47, 63, 0.65)',
      border: 'rgba(55, 65, 81, 0.5)',
      text: '#cbd5e1',
    };
  }

  // Harmonious modern macOS palette (Cobalt, Royal Indigo, Deep Teal, Pure Azure, Purple, Warm Amber, Crimson Rose, Emerald)
  const palettes = [
    { bg: 'rgba(2, 132, 199, 0.72)', border: 'rgba(56, 189, 248, 0.75)' },  // Electric Sky
    { bg: 'rgba(79, 70, 229, 0.72)', border: 'rgba(129, 140, 248, 0.75)' }, // Royal Indigo
    { bg: 'rgba(13, 148, 136, 0.72)', border: 'rgba(45, 212, 191, 0.75)' },  // Deep Teal
    { bg: 'rgba(37, 99, 235, 0.72)', border: 'rgba(96, 165, 250, 0.75)' },  // Pure Blue
    { bg: 'rgba(147, 51, 234, 0.72)', border: 'rgba(192, 132, 252, 0.75)' }, // Purple
    { bg: 'rgba(180, 83, 9, 0.72)', border: 'rgba(251, 191, 36, 0.75)' },   // Warm Amber
    { bg: 'rgba(225, 29, 72, 0.72)', border: 'rgba(251, 113, 133, 0.75)' },  // Crimson Rose
    { bg: 'rgba(5, 150, 105, 0.72)', border: 'rgba(52, 211, 153, 0.75)' },  // Emerald
  ];

  const choice = palettes[index % palettes.length];
  return {
    ...choice,
    text: '#ffffff',
  };
}
