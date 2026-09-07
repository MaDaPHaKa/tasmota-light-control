export function rgbcctHex(rgb: string, kelvin: number): string {
  const clamped = Math.min(6000, Math.max(2000, kelvin));
  const cold = Math.round(((6000 - clamped) * 255) / 4000);
  const warm = 255 - cold;
  return `${rgb.toUpperCase()}${cold.toString(16).padStart(2, '0')}${warm.toString(16).padStart(2, '0')}`.toUpperCase();
}
