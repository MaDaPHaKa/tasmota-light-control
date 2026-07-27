import { Injectable, inject } from '@angular/core';
import { MatSnackBar } from '@angular/material/snack-bar';
import { ApiFailure } from '@model/api-error.model';

@Injectable({ providedIn: 'root' })
export class SnackbarService {
  private readonly snackBar = inject(MatSnackBar);

  success(message: string): void {
    this.open(message, 'success');
  }
  info(message: string): void {
    this.open(message, 'info');
  }
  error(message: string): void {
    this.open(message, 'error');
  }
  failure(failure: ApiFailure, fallback = 'Operation failed.'): void {
    const fieldMessages = Object.entries(failure.fields).map(([field, message]) => `${field}: ${message}`);
    const message = fieldMessages.length
      ? `${failure.message || fallback} ${fieldMessages.join(' ')} `
      : failure.message || fallback;
    this.open(failure.requestId ? `${message} Request ID: ${failure.requestId}` : message, 'error');
  }
  private open(message: string, panelClass: string): void {
    this.snackBar.open(message, 'Close', {
      duration: 10_000,
      panelClass: [`snackbar-${panelClass}`],
      horizontalPosition: 'center',
      verticalPosition: 'bottom',
    });
  }
}
