export type Uuid = string;
export type Timestamp = string;
export type TasmotaResultCode =
  'success' | 'timeout' | 'unreachable' | 'invalid_response' | 'device_error' | 'http_error';
export type Reachability =
  'reachable' | 'unreachable' | 'timeout' | 'invalid_response' | 'device_error';
export type LightMode = 'rgb' | 'color_temperature' | 'mixed';
export interface Bulb {
  id: Uuid;
  name: string;
  ipAddress: string;
  port: number;
  createdAt: Timestamp;
  updatedAt: Timestamp;
}
export interface BulbInput {
  name: string;
  ipAddress: string;
  port: number;
}
export interface BulbEndpointInput {
  ipAddress: string;
  port: number;
}
export interface BulbTestResult {
  status: TasmotaResultCode;
  compatible: boolean | null;
  message: string;
  checkedAt: Timestamp;
}
export interface LiveBulbState {
  bulbId: Uuid;
  reachability: Reachability;
  power: 'on' | 'off' | null;
  dimmer: number | null;
  mode: LightMode | null;
  rgbColor: string | null;
  colorTemperatureKelvin: number | null;
  appliedProfileId: Uuid | null;
  checkedAt: Timestamp;
}
