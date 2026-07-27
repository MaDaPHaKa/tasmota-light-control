import { HttpErrorResponse, HttpInterceptorFn } from '@angular/common/http';
import { catchError, throwError } from 'rxjs';
import { ApiErrorResponse, ApiFailure } from '@model/api-error.model';
export const apiErrorInterceptor: HttpInterceptorFn = (request, next) =>
  next(request).pipe(
    catchError((error: unknown) => {
      const http = error instanceof HttpErrorResponse ? error : null;
      const response = http?.error as Partial<ApiErrorResponse> | null;
      const body = (response?.error ?? response) as Partial<ApiErrorResponse['error']> | null;
      const failure: ApiFailure = {
        status: http?.status ?? 0,
        code: body?.code ?? (http?.status === 422 ? 'validation_failed' : 'network_error'),
        message: body?.message ?? 'Request could not be completed.',
        fields: body?.fields ?? {},
        requestId: body?.requestId ?? null,
        retryAfter: Number(http?.headers.get('Retry-After')) || null,
      };
      return throwError(() => failure);
    }),
  );
