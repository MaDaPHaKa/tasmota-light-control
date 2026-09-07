import {
  ChangeDetectionStrategy,
  Component,
  DestroyRef,
  inject,
  signal,
} from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { finalize, Observable } from 'rxjs';
import { MatButtonModule } from '@angular/material/button';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatDialog } from '@angular/material/dialog';
import { MatIconModule } from '@angular/material/icon';
import { AppStoreService } from '@services/app-store.service';
import { ControlApiService } from '@services/control-api.service';
import { SnackbarService } from '@services/snackbar.service';
import { ControlOperationResult, SetPropertiesRequest } from '@model/control.model';
import { ApiFailure } from '@model/api-error.model';
import { Uuid } from '@model/bulb.model';
import {
  ConfirmDialogComponent,
  ConfirmDialogData,
  ConfirmDialogResult,
} from '@components/shared/confirm-dialog/confirm-dialog.component';
import { PageHeader } from '@components/shared/page-header/page-header.component';
import { ProfileControlComponent } from '@components/control/profile-control/profile-control.component';
import { OnTheFlyControlComponent } from '@components/control/on-the-fly-control/on-the-fly-control.component';
import { ResetBulbsComponent } from '@components/control/reset-bulbs/reset-bulbs.component';

@Component({
  selector: 'app-control-page',
  standalone: true,
  imports: [
    MatButtonModule,
    MatCheckboxModule,
    MatIconModule,
    ProfileControlComponent,
    OnTheFlyControlComponent,
    ResetBulbsComponent,
    PageHeader,
  ],
  templateUrl: './control-page.component.html',
  styleUrl: './control-page.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ControlPageComponent {
  private readonly store = inject(AppStoreService);
  private readonly api = inject(ControlApiService);
  private readonly dialog = inject(MatDialog);
  private readonly snackbar = inject(SnackbarService);
  private readonly destroyRef = inject(DestroyRef);
  protected readonly bulbs = this.store.bulbs;
  protected readonly profiles = this.store.profiles;
  protected readonly selectedProfile = signal<Uuid>('');
  protected readonly selectedBulbs = signal(new Set<Uuid>());
  protected readonly pending = signal(false);
  protected chooseProfile(id: Uuid) {
    this.selectedProfile.set(id);
  }
  protected toggleBulb(id: Uuid) {
    this.selectedBulbs.update((set) => {
      const next = new Set(set);
      next.has(id) ? next.delete(id) : next.add(id);
      return next;
    });
  }
  protected selectAll() {
    this.selectedBulbs.set(new Set(this.bulbs().map((b) => b.id)));
  }
  protected clearAll() {
    this.selectedBulbs.set(new Set());
  }
  protected apply() {
    const profileId = this.selectedProfile(),
      bulbIds = [...this.selectedBulbs()];
    if (!profileId || !bulbIds.length || this.pending()) return;
    this.execute(this.api.applyProfile({ profileId, bulbIds }));
  }
  protected reset() {
    const bulbIds = [...this.selectedBulbs()];
    if (!bulbIds.length || this.pending()) return;
    this.execute(this.api.reset({ bulbIds }));
  }
  protected setProperties(properties: Omit<SetPropertiesRequest, 'bulbIds'>) {
    const bulbIds = [...this.selectedBulbs()];
    if (!bulbIds.length || this.pending()) return;
    this.execute(this.api.setProperties({ bulbIds, ...properties }));
  }
  protected resetAll() {
    if (!this.bulbs().length || this.pending()) return;
    const data: ConfirmDialogData = {
      title: 'Reset all bulbs?',
      consequence: `Reset all ${this.bulbs().length} registered bulbs to configured defaults.`,
      confirmLabel: 'Reset all bulbs',
      destructive: true,
    };
    this.dialog
      .open<ConfirmDialogComponent, ConfirmDialogData, ConfirmDialogResult>(
        ConfirmDialogComponent,
        { data, autoFocus: 'first-tabbable' },
      )
      .afterClosed()
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((value) => {
        if (value === 'confirmed') this.execute(this.api.resetAll());
      });
  }
  private execute(request: Observable<ControlOperationResult>) {
    if (this.pending()) return;
    this.pending.set(true);
    request
      .pipe(
        finalize(() => this.pending.set(false)),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe({
        next: (r) => {
          r.results
            .filter((x) => x.status === 'success')
            .forEach((x) => this.store.refreshStatus(x.bulbId));
          const message = `Operation completed: ${r.summary.succeeded} succeeded, ${r.summary.failed} failed, ${r.summary.total} total.`;
          if (!r.summary.failed) this.snackbar.success(message);
          else if (r.summary.succeeded) this.snackbar.info(message);
          else this.snackbar.error(message);
        },
        error: (failure: ApiFailure) => this.snackbar.failure(failure),
      });
  }
}
