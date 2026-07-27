export type ResetSettings = RgbResetSettings | ColorTemperatureResetSettings;
export interface RgbResetSettings {
  dimmer: number;
  mode: 'rgb';
  rgbColor: string;
  colorTemperatureKelvin: null;
}
export interface ColorTemperatureResetSettings {
  dimmer: number;
  mode: 'color_temperature';
  rgbColor: null;
  colorTemperatureKelvin: number;
}
