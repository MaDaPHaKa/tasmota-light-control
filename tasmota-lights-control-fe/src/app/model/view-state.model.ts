import { LiveBulbState, Uuid } from './bulb.model';
export type LoadState = 'idle' | 'loading' | 'loaded' | 'error';
export type BulbStatusViewState =
  | { kind: 'idle' }
  | { kind: 'loading' }
  | { kind: 'loaded'; value: LiveBulbState }
  | { kind: 'error'; message: string };
export type StatusMap = ReadonlyMap<Uuid, BulbStatusViewState>;
