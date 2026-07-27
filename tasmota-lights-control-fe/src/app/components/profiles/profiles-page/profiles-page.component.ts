import { ChangeDetectionStrategy, Component, DestroyRef, inject, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { finalize } from 'rxjs';
import { MatButtonModule } from '@angular/material/button';
import { MatDialog } from '@angular/material/dialog';
import { MatIconModule } from '@angular/material/icon';
import { ProfileFormComponent } from '../profile-form/profile-form.component';
import { ProfileListComponent } from '../profile-list/profile-list.component';
import { AppStoreService } from '@services/app-store.service';
import { ProfilesApiService } from '@services/profiles-api.service';
import { LightProfile, LightProfileInput } from '@model/profile.model';
import {
  ConfirmDialogComponent,
  ConfirmDialogData,
} from '@components/shared/confirm-dialog/confirm-dialog.component';
import { SnackbarService } from '@services/snackbar.service';
import { ApiFailure } from '@model/api-error.model';
import { PageHeader } from '@components/shared/page-header/page-header.component';

@Component({
  selector: 'app-profiles-page',
  standalone: true,
  imports: [MatButtonModule, MatIconModule, ProfileFormComponent, ProfileListComponent, PageHeader],
  templateUrl: './profiles-page.component.html',
  styleUrl: './profiles-page.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ProfilesPageComponent {
  private readonly store = inject(AppStoreService);
  private readonly api = inject(ProfilesApiService);
  private readonly dialog = inject(MatDialog);
  private readonly snackbar = inject(SnackbarService);
  private readonly destroyRef = inject(DestroyRef);
  protected readonly profiles = this.store.profiles;
  protected readonly showForm = signal(false);
  protected readonly editing = signal<LightProfile | null>(null);
  protected readonly pending = signal(false);
  protected readonly failure = signal<ApiFailure | null>(null);
  protected toggleForm(): void {
    this.editing.set(null);
    this.showForm.update((value) => !value);
  }
  protected edit(profile: LightProfile): void {
    this.editing.set(profile);
    this.showForm.set(true);
  }
  protected save(input: LightProfileInput): void {
    if (this.pending()) return;
    this.pending.set(true);
    this.failure.set(null);
    const request = this.editing()
      ? this.api.replace(this.editing()!.id, input)
      : this.api.create(input);
    request
      .pipe(
        finalize(() => this.pending.set(false)),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe({
        next: (profile) => {
          this.store.upsertProfile(profile);
          this.showForm.set(false);
          this.snackbar.success(`${profile.name} saved.`);
        },
        error: (failure: ApiFailure) => {
          this.failure.set(failure);
          this.snackbar.failure(failure, 'Could not save profile.');
        },
      });
  }
  protected clearField(field: string) {
    this.failure.update((f) =>
      f
        ? {
            ...f,
            fields: Object.fromEntries(Object.entries(f.fields).filter(([key]) => key !== field)),
          }
        : null,
    );
  }
  protected remove(profile: LightProfile): void {
    const data: ConfirmDialogData = {
      title: `Delete ${profile.name}?`,
      consequence: 'This profile will be permanently removed.',
      confirmLabel: 'Delete profile',
      destructive: true,
    };
    this.dialog
      .open(ConfirmDialogComponent, { data, autoFocus: 'first-tabbable' })
      .afterClosed()
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((confirmed) => {
        if (confirmed !== 'confirmed') return;
        this.api
          .delete(profile.id)
          .pipe(takeUntilDestroyed(this.destroyRef))
          .subscribe({
            next: () => {
              this.store.removeProfile(profile.id);
              this.snackbar.success('Profile deleted.');
            },
            error: (failure: ApiFailure) =>
              this.snackbar.failure(failure, 'Could not delete profile.'),
          });
      });
  }
}
