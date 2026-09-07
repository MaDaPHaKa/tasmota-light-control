export interface MixedResetSettings {
  dimmer: number;
  mode: 'mixed';
  rgbColor: string;
  colorTemperatureKelvin: number;
  fade: number | null;
  speed: number | null;
}
export type ResetSettings = RgbResetSettings | ColorTemperatureResetSettings | MixedResetSettings;
export interface RgbResetSettings {
  dimmer: number;
  mode: 'rgb';
  rgbColor: string;
  colorTemperatureKelvin: null;
  fade: number | null;
  speed: number | null;
}
export interface ColorTemperatureResetSettings {
  dimmer: number;
  mode: 'color_temperature';
  rgbColor: null;
  colorTemperatureKelvin: number;
  fade: number | null;
  speed: number | null;
}
