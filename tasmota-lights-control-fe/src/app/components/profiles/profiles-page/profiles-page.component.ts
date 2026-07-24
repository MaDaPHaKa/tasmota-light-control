import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatDialog } from '@angular/material/dialog';
import { MatIconModule } from '@angular/material/icon';
import { ProfileFormComponent } from '../profile-form/profile-form.component';
import { ProfileListComponent } from '../profile-list/profile-list.component';
import { AppStoreService } from '@services/app-store.service';
import { ProfilesApiService } from '@services/profiles-api.service';
import { LightProfile, LightProfileInput } from '@model/profile.model';
import { ConfirmDialogComponent, ConfirmDialogData } from '@components/shared/confirm-dialog/confirm-dialog.component';
import { SnackbarService } from '@services/snackbar.service';

@Component({ selector: 'app-profiles-page', standalone: true, imports: [MatButtonModule, MatIconModule, ProfileFormComponent, ProfileListComponent], templateUrl: './profiles-page.component.html', styleUrl: './profiles-page.component.scss', changeDetection: ChangeDetectionStrategy.OnPush })
export class ProfilesPageComponent {
  private readonly store = inject(AppStoreService); private readonly api = inject(ProfilesApiService); private readonly dialog = inject(MatDialog); private readonly snackbar = inject(SnackbarService);
  protected readonly profiles = this.store.profiles; protected readonly showForm = signal(false); protected readonly editing = signal<LightProfile | null>(null);
  protected toggleForm(): void { this.editing.set(null); this.showForm.update((value) => !value); }
  protected edit(profile: LightProfile): void { this.editing.set(profile); this.showForm.set(true); }
  protected save(input: LightProfileInput): void { const request = this.editing() ? this.api.replace(this.editing()!.id, input) : this.api.create(input); request.subscribe({ next: (profile) => { this.store.upsertProfile(profile); this.showForm.set(false); this.snackbar.success(`${profile.name} saved.`); }, error: () => this.snackbar.error('Could not save profile.') }); }
  protected remove(profile: LightProfile): void { const data: ConfirmDialogData = { title: `Delete ${profile.name}?`, consequence: 'This profile will be permanently removed.', confirmLabel: 'Delete profile', destructive: true }; this.dialog.open(ConfirmDialogComponent, { data }).afterClosed().subscribe((confirmed) => { if (!confirmed) return; this.api.delete(profile.id).subscribe({ next: () => { this.store.removeProfile(profile.id); this.snackbar.success('Profile deleted.'); }, error: () => this.snackbar.error('Could not delete profile.') }); }); }
}
