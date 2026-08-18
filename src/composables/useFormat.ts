export function formatBytes(bytes: number | null | undefined, decimals = 2): string {
  if (bytes === null || bytes === undefined || isNaN(bytes)) return '0 B';
  if (bytes === 0) return '0 B';

  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];

  const i = Math.floor(Math.log(Math.abs(bytes)) / Math.log(k));
  const idx = Math.min(i, sizes.length - 1);
  const val = parseFloat((bytes / Math.pow(k, idx)).toFixed(dm));

  return `${val} ${sizes[idx]}`;
}

export function formatDelta(delta: number | null | undefined, decimals = 2): string {
  if (delta === null || delta === undefined || isNaN(delta)) return '0 B';
  if (delta === 0) return '0 B';

  const prefix = delta > 0 ? '+' : '';
  return `${prefix}${formatBytes(delta, decimals)}`;
}

export function formatPercent(percent: number | null | undefined): string {
  if (percent === null || percent === undefined || isNaN(percent)) return '0.0%';
  if (percent === 0) return '0.0%';

  const prefix = percent > 0 ? '+' : '';
  return `${prefix}${percent.toFixed(1)}%`;
}

export function formatNumber(num: number | null | undefined): string {
  if (num === null || num === undefined) return '0';
  return num.toLocaleString();
}
