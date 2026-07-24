import { LiveBulbState, Uuid } from './bulb.model';
export type LoadState = 'idle' | 'loading' | 'loaded' | 'error';
export type ActionState = 'idle' | 'pending' | 'success' | 'partial_failure' | 'failure';
export type BulbStatusViewState = { kind: 'idle' } | { kind: 'loading' } | { kind: 'loaded'; value: LiveBulbState } | { kind: 'error'; message: string };
export type StatusMap = ReadonlyMap<Uuid, BulbStatusViewState>;
