import {
  ChangeDetectionStrategy,
  Component,
  DestroyRef,
  computed,
  inject,
  signal,
} from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { finalize, Observable } from 'rxjs';
import { MatButtonModule } from '@angular/material/button';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatDialog } from '@angular/material/dialog';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { MatRadioModule } from '@angular/material/radio';
import { MatSelectModule } from '@angular/material/select';
import { AppStoreService } from '@services/app-store.service';
import { ControlApiService } from '@services/control-api.service';
import { SnackbarService } from '@services/snackbar.service';
import { ControlOperationResult, SetPropertiesRequest } from '@model/control.model';
import { ApiFailure } from '@model/api-error.model';
import { LightMode, Uuid } from '@model/bulb.model';
import {
  ConfirmDialogComponent,
  ConfirmDialogData,
  ConfirmDialogResult,
} from '@components/shared/confirm-dialog/confirm-dialog.component';
import { PageHeader } from '@components/shared/page-header/page-header.component';
import { RgbColorPickerComponent } from '@components/shared/rgb-color-picker/rgb-color-picker.component';
import { ModePreviewComponent } from '@components/shared/mode-preview/mode-preview.component';
import { rgbcctHex } from '@functions/light-color.function';
import { validateIntegerRange, validateUppercaseRgbHex } from '@functions/validators.function';

@Component({
  selector: 'app-control-page',
  standalone: true,
  imports: [
    MatButtonModule,
    MatCheckboxModule,
    MatFormFieldModule,
    MatIconModule,
    MatInputModule,
    MatRadioModule,
    MatSelectModule,
    RgbColorPickerComponent,
    ModePreviewComponent,
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
  protected readonly dimmer = signal(100);
  protected readonly mode = signal<LightMode>('rgb');
  protected readonly rgbColor = signal('#88C0D0');
  protected readonly colorTemperatureKelvin = signal(3000);
  protected readonly applyValid = computed(() => {
    if (this.pending() || !this.selectedBulbs().size) return false;
    if (this.dimmer() < 1 || this.dimmer() > 100) return false;
    if (
      this.mode() !== 'color_temperature' &&
      validateUppercaseRgbHex(this.rgbColor()) !== undefined
    )
      return false;
    if (
      this.mode() !== 'rgb' &&
      validateIntegerRange(this.colorTemperatureKelvin(), 2000, 6000) !== undefined
    )
      return false;
    return true;
  });
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
  protected setProperties() {
    const bulbIds = [...this.selectedBulbs()];
    if (!bulbIds.length || this.pending()) return;
    const request: SetPropertiesRequest = { bulbIds, dimmer: this.dimmer() };
    if (this.mode() !== 'color_temperature') request.rgbColor = this.rgbColor().toUpperCase();
    if (this.mode() !== 'rgb') request.colorTemperatureKelvin = this.colorTemperatureKelvin();
    this.execute(this.api.setProperties(request));
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
  protected colorPreview() {
    return this.mode() === 'mixed'
      ? rgbcctHex(this.rgbColor(), this.colorTemperatureKelvin())
      : this.rgbColor();
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
