export type ErrorCode = 'bad_request' | 'not_found' | 'conflict' | 'validation_failed' | 'too_many_requests' | 'internal_error' | 'service_unavailable';
export interface ApiErrorResponse { error: { code: ErrorCode; message: string; fields?: Record<string, string>; requestId: string; }; }
export interface ApiFailure { status: number; code: ErrorCode | 'network_error'; message: string; fields: Record<string, string>; requestId: string | null; retryAfter: number | null; }
