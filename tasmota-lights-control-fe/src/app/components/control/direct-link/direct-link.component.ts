import {
  ChangeDetectionStrategy,
  Component,
  DestroyRef,
  signal,
  inject,
} from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { finalize } from 'rxjs';
import { Clipboard } from '@angular/cdk/clipboard';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import { Uuid } from '@model/bulb.model';
import { ApiFailure } from '@model/api-error.model';
import { ControlApiService } from '@services/control-api.service';
import { SnackbarService } from '@services/snackbar.service';
import { AppStoreService } from '@services/app-store.service';
@Component({
  selector: 'app-direct-link',
  standalone: true,
  imports: [MatButtonModule, MatFormFieldModule, MatInputModule, MatSelectModule],
  templateUrl: './direct-link.component.html',
  styleUrl: './direct-link.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DirectLinkComponent {
  private readonly store = inject(AppStoreService);
  readonly bulbs = this.store.bulbs;
  readonly profiles = this.store.profiles;
  private readonly api = inject(ControlApiService);
  private readonly clipboard = inject(Clipboard);
  private readonly snackbar = inject(SnackbarService);
  private readonly destroyRef = inject(DestroyRef);
  protected readonly bulbId = signal<Uuid>('');
  protected readonly profileId = signal<Uuid>('');
  protected readonly url = signal('');
  protected readonly pending = signal(false);
  protected selectBulb(value: Uuid) {
    this.bulbId.set(value);
    this.clearResult();
  }
  protected selectProfile(value: Uuid) {
    this.profileId.set(value);
    this.clearResult();
  }
  protected generate() {
    if (!this.bulbId() || !this.profileId() || this.pending()) return;
    this.pending.set(true);
    this.api
      .generateDirectLink({ bulbId: this.bulbId(), profileId: this.profileId() })
      .pipe(
        finalize(() => this.pending.set(false)),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe({
        next: (v) => {
          this.url.set(v);
          this.snackbar.success('Direct link generated.');
        },
        error: (failure: ApiFailure) =>
          this.snackbar.failure(failure, 'Could not generate direct link.'),
      });
  }
  protected copy() {
    if (this.url() && this.clipboard.copy(this.url())) this.snackbar.success('Direct link copied.');
    else this.snackbar.error('Could not copy direct link.');
  }
  private clearResult() {
    this.url.set('');
  }
}
