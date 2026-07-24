import { Injectable, inject } from '@angular/core';
import { MatSnackBar } from '@angular/material/snack-bar';
import { ApiFailure } from '@model/api-error.model';

@Injectable({ providedIn: 'root' })
export class SnackbarService {
  private readonly snackBar = inject(MatSnackBar);

  success(message: string): void { this.open(message, 'success'); }
  info(message: string): void { this.open(message, 'info'); }
  error(message: string): void { this.open(message, 'error', 6000); }
  failure(failure: ApiFailure, fallback = 'Operation failed.'): void { this.error(failure.message || fallback); }
  private open(message: string, panelClass: string, duration = 3500): void { this.snackBar.open(message, 'Dismiss', { duration, panelClass: [`snackbar-${panelClass}`], horizontalPosition: 'right', verticalPosition: 'bottom' }); }
}
