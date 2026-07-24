import { TasmotaResultCode, Uuid, Timestamp } from './bulb.model';
export interface ApplyProfileRequest { profileId: Uuid; bulbIds: Uuid[]; }
export interface ResetBulbsRequest { bulbIds: Uuid[]; }
export interface DirectLinkRequest { bulbId: Uuid; profileId: Uuid; }
export type ControlOperation = 'apply_profile' | 'reset' | 'reset_all';
export interface TargetControlResult { bulbId: Uuid; bulbName: string; status: TasmotaResultCode; message: string; }
export interface ControlOperationResult { operation: ControlOperation; profileId: Uuid | null; startedAt: Timestamp; completedAt: Timestamp; summary: { total: number; succeeded: number; failed: number }; results: TargetControlResult[]; }
