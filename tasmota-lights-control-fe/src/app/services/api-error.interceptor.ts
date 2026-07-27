import { HttpErrorResponse, HttpInterceptorFn } from '@angular/common/http';
import { catchError, throwError } from 'rxjs';
import { ApiErrorResponse, ApiFailure } from '@model/api-error.model';
export const apiErrorInterceptor: HttpInterceptorFn = (request, next) =>
  next(request).pipe(
    catchError((error: unknown) => {
      const http = error instanceof HttpErrorResponse ? error : null;
      const body = http?.error as Partial<ApiErrorResponse> | null;
      const failure: ApiFailure = {
        status: http?.status ?? 0,
        code: body?.error?.code ?? 'network_error',
        message: body?.error?.message ?? 'Request could not be completed.',
        fields: body?.error?.fields ?? {},
        requestId: body?.error?.requestId ?? null,
        retryAfter: Number(http?.headers.get('Retry-After')) || null,
      };
      return throwError(() => failure);
    }),
  );
