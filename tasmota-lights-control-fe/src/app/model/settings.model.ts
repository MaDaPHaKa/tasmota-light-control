export type ResetSettings = RgbResetSettings | ColorTemperatureResetSettings;
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
