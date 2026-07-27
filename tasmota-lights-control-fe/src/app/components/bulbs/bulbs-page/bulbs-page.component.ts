import { ChangeDetectionStrategy, Component, DestroyRef, inject, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { finalize } from 'rxjs';
import { MatButtonModule } from '@angular/material/button';
import { MatDialog } from '@angular/material/dialog';
import { MatIconModule } from '@angular/material/icon';
import { AppStoreService } from '@services/app-store.service';
import { BulbsApiService } from '@services/bulbs-api.service';
import { ControlApiService } from '@services/control-api.service';
import { Bulb, BulbInput, BulbTestResult, Uuid } from '@model/bulb.model';
import { ApiFailure } from '@model/api-error.model';
import { SnackbarService } from '@services/snackbar.service';
import { BulbFormComponent } from '../bulb-form/bulb-form.component';
import { BulbListComponent } from '../bulb-list/bulb-list.component';
import {
  ConfirmDialogComponent,
  ConfirmDialogData,
  ConfirmDialogResult,
} from '@components/shared/confirm-dialog/confirm-dialog.component';
import { PageHeader } from '@components/shared/page-header/page-header.component';

@Component({
  selector: 'app-bulbs-page',
  standalone: true,
  imports: [MatButtonModule, MatIconModule, BulbFormComponent, BulbListComponent, PageHeader],
  templateUrl: './bulbs-page.component.html',
  styleUrl: './bulbs-page.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BulbsPageComponent {
  private readonly store = inject(AppStoreService);
  private readonly api = inject(BulbsApiService);
  private readonly controlApi = inject(ControlApiService);
  private readonly dialog = inject(MatDialog);
  private readonly snackbar = inject(SnackbarService);
  private readonly destroyRef = inject(DestroyRef);
  protected readonly bulbs = this.store.bulbs;
  protected readonly profiles = this.store.profiles;
  protected readonly statuses = this.store.statusByBulbId;
  protected readonly showForm = signal(false);
  protected readonly editing = signal<Bulb | null>(null);
  protected readonly pending = signal(false);
  protected readonly testPending = signal(false);
  protected readonly failure = signal<ApiFailure | null>(null);
  protected readonly lastTestResult = signal<BulbTestResult | null>(null);
  protected readonly pendingActionIds = signal(new Set<Uuid>());
  protected toggleForm() {
    this.editing.set(null);
    this.failure.set(null);
    this.lastTestResult.set(null);
    this.showForm.update((v) => !v);
  }
  protected edit(b: Bulb) {
    this.editing.set(b);
    this.failure.set(null);
    this.lastTestResult.set(null);
    this.showForm.set(true);
  }
  protected save(input: BulbInput) {
    if (this.pending()) return;
    const test = this.lastTestResult();
    if (test && (test.status !== 'success' || test.compatible !== true)) {
      const data: ConfirmDialogData = {
        title: 'Save unverified bulb?',
        consequence: `Latest endpoint test returned ${test.status}${test.compatible === false ? ' and reported an incompatible device' : ''}. Save these values anyway?`,
        confirmLabel: 'Save anyway',
        destructive: false,
      };
      this.dialog
        .open<ConfirmDialogComponent, ConfirmDialogData, ConfirmDialogResult>(
          ConfirmDialogComponent,
          { data, autoFocus: 'first-tabbable' },
        )
        .afterClosed()
        .pipe(takeUntilDestroyed(this.destroyRef))
        .subscribe((result) => {
          if (result === 'confirmed') this.performSave(input);
        });
      return;
    }
    this.performSave(input);
  }
  private performSave(input: BulbInput) {
    this.pending.set(true);
    this.failure.set(null);
    const editing = this.editing();
    (editing ? this.api.replace(editing.id, input) : this.api.create(input))
      .pipe(
        finalize(() => this.pending.set(false)),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe({
        next: (b) => {
          this.store.upsertBulb(b);
          this.showForm.set(false);
          this.snackbar.success(`${b.name} saved.`);
        },
        error: (failure: ApiFailure) => {
          this.failure.set(failure);
          this.snackbar.failure(failure, 'Could not save bulb.');
        },
      });
  }
  protected test(bulb: Bulb) {
    if (this.pendingActionIds().has(bulb.id)) return;
    this.markPending(bulb.id, true);
    this.api
      .testSaved(bulb.id)
      .pipe(
        finalize(() => this.markPending(bulb.id, false)),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe({
        next: (r) => this.snackbar.info(`${bulb.name} test: ${r.status} — ${r.message}`),
        error: (f: ApiFailure) => this.snackbar.failure(f),
      });
  }
  protected testEndpoint(endpoint: { ipAddress: string; port: number }) {
    if (this.testPending()) return;
    this.testPending.set(true);
    this.api
      .testUnsaved(endpoint)
      .pipe(
        finalize(() => this.testPending.set(false)),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe({
        next: (r) => this.lastTestResult.set(r),
        error: (failure: ApiFailure) => {
          this.failure.set(failure);
          this.snackbar.failure(failure, 'Could not test bulb endpoint.');
        },
      });
  }
  protected clearTest() {
    this.lastTestResult.set(null);
    this.failure.set(null);
  }
  protected clearField(field: keyof BulbInput) {
    this.failure.update((f) =>
      f
        ? {
            ...f,
            fields: Object.fromEntries(Object.entries(f.fields).filter(([key]) => key !== field)),
          }
        : null,
    );
  }
  protected refresh(id: Uuid) {
    this.store.refreshStatus(id);
  }
  protected reset(id: Uuid) {
    if (this.pendingActionIds().has(id)) return;
    this.markPending(id, true);
    this.controlApi
      .reset({ bulbIds: [id] })
      .pipe(
        finalize(() => this.markPending(id, false)),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe({
        next: (r) => {
          if (r.summary.succeeded) {
            this.store.refreshStatus(id);
            this.snackbar.success('1 of 1 bulb reset.');
          } else this.snackbar.error('Bulb reset failed: 0 succeeded, 1 failed.');
        },
        error: (f: ApiFailure) => this.snackbar.failure(f),
      });
  }
  protected remove(b: Bulb) {
    const data: ConfirmDialogData = {
      title: `Delete ${b.name}?`,
      consequence: 'This bulb registration will be permanently removed.',
      confirmLabel: 'Delete bulb',
      destructive: true,
    };
    this.dialog
      .open<ConfirmDialogComponent, ConfirmDialogData, ConfirmDialogResult>(
        ConfirmDialogComponent,
        { data, autoFocus: 'first-tabbable' },
      )
      .afterClosed()
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((ok) => {
        if (ok !== 'confirmed') return;
        this.api
          .delete(b.id)
          .pipe(takeUntilDestroyed(this.destroyRef))
          .subscribe({
            next: () => {
              this.store.removeBulb(b.id);
              this.snackbar.success(`${b.name} deleted.`);
            },
            error: (f: ApiFailure) => this.snackbar.failure(f),
          });
      });
  }
  private markPending(id: Uuid, pending: boolean) {
    this.pendingActionIds.update((current) => {
      const next = new Set(current);
      pending ? next.add(id) : next.delete(id);
      return next;
    });
  }
}
