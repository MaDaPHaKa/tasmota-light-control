import { LightMode, Timestamp, Uuid } from './bulb.model';
export interface ResourceFields {
  id: Uuid;
  createdAt: Timestamp;
  updatedAt: Timestamp;
}
export interface RgbProfileInput {
  name: string;
  dimmer: number;
  mode: 'rgb';
  rgbColor: string;
  colorTemperatureKelvin: null;
}
export interface ColorTemperatureProfileInput {
  name: string;
  dimmer: number;
  mode: 'color_temperature';
  rgbColor: null;
  colorTemperatureKelvin: number;
}
export interface MixedProfileInput {
  name: string;
  dimmer: number;
  mode: 'mixed';
  rgbColor: string;
  colorTemperatureKelvin: number;
}
export type LightProfileInput = RgbProfileInput | ColorTemperatureProfileInput | MixedProfileInput;
export type RgbLightProfile = RgbProfileInput & ResourceFields;
export type ColorTemperatureLightProfile = ColorTemperatureProfileInput & ResourceFields;
export type MixedLightProfile = MixedProfileInput & ResourceFields;
export type LightProfile = RgbLightProfile | ColorTemperatureLightProfile | MixedLightProfile;
export type ProfileMode = LightMode;
