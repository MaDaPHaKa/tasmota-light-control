import { LightMode, Timestamp, Uuid } from './bulb.model';
export interface ResourceFields { id: Uuid; createdAt: Timestamp; updatedAt: Timestamp; }
export interface RgbProfileInput { name: string; dimmer: number; mode: 'rgb'; rgbColor: string; colorTemperatureKelvin: null; }
export interface ColorTemperatureProfileInput { name: string; dimmer: number; mode: 'color_temperature'; rgbColor: null; colorTemperatureKelvin: number; }
export type LightProfileInput = RgbProfileInput | ColorTemperatureProfileInput;
export type RgbLightProfile = RgbProfileInput & ResourceFields;
export type ColorTemperatureLightProfile = ColorTemperatureProfileInput & ResourceFields;
export type LightProfile = RgbLightProfile | ColorTemperatureLightProfile;
export type ProfileMode = LightMode;
