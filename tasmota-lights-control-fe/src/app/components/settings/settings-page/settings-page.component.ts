import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  DestroyRef,
  effect,
  inject,
  signal,
  viewChild,
  viewChildren,
} from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { finalize, Subscription } from 'rxjs';
import { FormField, disabled, form, required, validate } from '@angular/forms/signals';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import { ApiFailure } from '@model/api-error.model';
import { BulbSettings, ResetSettings } from '@model/settings.model';
import { Uuid } from '@model/bulb.model';
import {
  normalizeMixedColorHex,
  validateIntegerRange,
  validateMixedColorHex,
  validateUppercaseRgbHex,
} from '@functions/validators.function';
import { SettingsApiService } from '@services/settings-api.service';
import { DEFAULT_MIXED_COLOR, EditableLightConfiguration, LightConfigurationComponent, LightConfigurationField } from '@components/shared/light-configuration/light-configuration.component';
import { SnackbarService } from '@services/snackbar.service';
import { PageHeader } from '@components/shared/page-header/page-header.component';
import { AppStoreService } from '@services/app-store.service';

type SettingsField = 'dimmer' | 'mode' | 'rgbColor' | 'colorTemperatureKelvin' | 'mixedColor';
type BulbSettingsLoadState = 'idle' | 'loading' | 'loaded' | 'error';

@Component({
  selector: 'app-settings-page',
  standalone: true,
  imports: [
    FormField,
    MatButtonModule,
    MatFormFieldModule,
    MatInputModule,
    MatSelectModule,
    LightConfigurationComponent,
    PageHeader,
  ],
  templateUrl: './settings-page.component.html',
  styleUrl: './settings-page.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SettingsPageComponent {
  private readonly api = inject(SettingsApiService);
  private readonly store = inject(AppStoreService);
  private readonly snackbar = inject(SnackbarService);
  private readonly destroyRef = inject(DestroyRef);
  protected readonly model = signal({
    dimmer: 100,
    mode: 'color_temperature' as 'rgb' | 'color_temperature' | 'mixed',
    rgbColor: '#88C0D0',
    colorTemperatureKelvin: 3000,
    mixedColor: DEFAULT_MIXED_COLOR,
  });
  protected readonly bulbSettingsModel = signal({ fade: 1 as 0 | 1, speed: 4 });
  protected readonly pending = signal(false);
  protected readonly failure = signal<ApiFailure | null>(null);
  protected readonly resetSettings = signal<ResetSettings | null>(null);
  protected readonly bulbs = this.store.bulbs;
  protected readonly bulbsLoadState = this.store.bulbsLoadState;
  protected readonly selectedBulbId = signal<Uuid | null>(null);
  protected readonly bulbSettingsLoadState = signal<BulbSettingsLoadState>('idle');
  protected readonly bulbSettingsSaving = signal(false);
  protected readonly bulbSettingsFailure = signal<ApiFailure | null>(null);
  protected readonly controls = viewChildren<ElementRef<HTMLElement>>('formControl');
  private readonly lightConfiguration = viewChild(LightConfigurationComponent);
  private bulbSettingsRequest?: Subscription;
  protected readonly settingsForm = form(this.model, (p) => {
    required(p.dimmer, { message: 'Dimmer is required' });
    validate(p.dimmer, ({ value }) => this.serverValidation('dimmer') ?? validateIntegerRange(value(), 1, 100));
    validate(p.mode, () => this.serverValidation('mode'));
    validate(p.rgbColor, ({ value }) =>
      this.model().mode === 'rgb' ? this.serverValidation('rgbColor') ?? validateUppercaseRgbHex(value()) : undefined,
    );
    validate(p.colorTemperatureKelvin, ({ value }) =>
      this.model().mode === 'color_temperature'
        ? this.serverValidation('colorTemperatureKelvin') ?? validateIntegerRange(value(), 2000, 6000)
        : undefined,
    );
    validate(p.mixedColor, ({ value }) =>
      this.model().mode === 'mixed' ? this.serverValidation('mixedColor') ?? validateMixedColorHex(value()) : undefined,
    );
  });
  protected readonly bulbSettingsForm = form(this.bulbSettingsModel, (p) => {
    disabled(p.fade, () => this.bulbSettingsSaving());
    disabled(p.speed, () => this.bulbSettingsSaving());
    required(p.fade, { message: 'Fade is required' });
    validate(p.fade, ({ value }) => validateIntegerRange(value(), 0, 1));
    required(p.speed, { message: 'Speed is required' });
    validate(p.speed, ({ value }) => validateIntegerRange(value(), 1, 40));
  });
  constructor() {
    effect(() => {
      const bulbs = this.bulbs();
      const selectedId = this.selectedBulbId();
      if (!selectedId && bulbs.length) this.selectBulb(bulbs[0].id);
      else if (selectedId && !bulbs.some((bulb) => bulb.id === selectedId)) {
        this.selectBulb(bulbs[0]?.id ?? null);
      }
    });
    this.pending.set(true);
    this.api
      .getResetSettings()
      .pipe(
        finalize(() => this.pending.set(false)),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe({
        next: (v) => {
          this.setSettings(v);
          this.snackbar.success('Settings loaded.');
        },
        error: (f: ApiFailure) => {
          this.failure.set(f);
          this.snackbar.failure(f, 'Could not load settings.');
        },
      });
  }
  protected updateConfiguration(value: EditableLightConfiguration) { this.model.update((current) => ({ ...current, ...value })); }
  protected configurationChanged(field: LightConfigurationField) {
    this.settingsForm[field]().markAsTouched();
    this.changed(field);
  }
  protected changed(field: SettingsField) {
    const apiField = field === 'rgbColor' ? 'rgb_color' : field;
    this.failure.update((f) =>
      f
        ? {
            ...f,
            fields: Object.fromEntries(Object.entries(f.fields).filter(([key]) => key !== field && key !== apiField)),
          }
        : null,
    );
  }
  protected clientError(field: SettingsField) {
    return this.settingsForm[field]().errors()[0]?.message ?? null;
  }
  protected showClientError(field: SettingsField) {
    const state = this.settingsForm[field]();
    return state.invalid() && (state.touched() || !!this.serverValidation(field));
  }
  private serverValidation(field: SettingsField) {
    const fields = this.failure()?.fields ?? {};
    const message = fields[field] ?? fields[field === 'rgbColor' ? 'rgb_color' : field];
    return message ? { kind: 'server', message } : undefined;
  }
  protected save() {
    const resetSettings = this.resetSettings();
    if (this.pending() || !resetSettings) return;
    this.settingsForm().markAsTouched();
    if (this.settingsForm().invalid()) {
      queueMicrotask(() => this.focusFirstInvalid());
      return;
    }
    const v = this.model();
    const { fade, speed } = resetSettings;
    const settings: ResetSettings =
      v.mode === 'rgb'
        ? {
            dimmer: v.dimmer,
            mode: 'rgb',
            rgbColor: v.rgbColor.toUpperCase(),
             colorTemperatureKelvin: null,
             fade,
             speed,
          }
        : v.mode === 'color_temperature'
          ? {
            dimmer: v.dimmer,
            mode: 'color_temperature',
            rgbColor: null,
            colorTemperatureKelvin: v.colorTemperatureKelvin,
            fade,
            speed,
            }
          : {
              dimmer: v.dimmer,
              mode: 'mixed',
              mixedColor: normalizeMixedColorHex(v.mixedColor),
              fade,
              speed,
            };
    this.pending.set(true);
    this.failure.set(null);
    this.api
      .replaceResetSettings(settings)
      .pipe(
        finalize(() => this.pending.set(false)),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe({
        next: (result) => {
          this.setSettings(result);
          this.snackbar.success('Settings saved.');
        },
        error: (failure: ApiFailure) => {
          this.failure.set(failure);
          this.snackbar.failure(failure, 'Could not save settings.');
        },
      });
  }
  private setSettings(v: ResetSettings) {
    this.resetSettings.set(v);
    this.settingsForm().reset({
      dimmer: v.dimmer,
      mode: v.mode,
      rgbColor: v.mode === 'rgb' ? v.rgbColor : '#88C0D0',
      colorTemperatureKelvin: v.mode === 'color_temperature' ? v.colorTemperatureKelvin : 3000,
      mixedColor: v.mode === 'mixed' ? v.mixedColor : DEFAULT_MIXED_COLOR,
    });
  }
  protected selectBulb(id: Uuid | null) {
    if (id === this.selectedBulbId()) return;
    this.selectedBulbId.set(id);
    if (!id) {
      this.bulbSettingsRequest?.unsubscribe();
      this.bulbSettingsLoadState.set('idle');
      this.bulbSettingsFailure.set(null);
      return;
    }
    this.loadBulbSettings(id);
  }
  private loadBulbSettings(id: Uuid) {
    this.bulbSettingsRequest?.unsubscribe();
    this.bulbSettingsFailure.set(null);
    this.bulbSettingsLoadState.set('loading');
    this.bulbSettingsRequest = this.api
      .getBulbSettings(id)
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: (settings) => {
          if (this.selectedBulbId() !== id) return;
          this.setBulbSettings(settings);
          this.bulbSettingsLoadState.set('loaded');
        },
        error: (failure: ApiFailure) => {
          if (this.selectedBulbId() !== id) return;
          this.bulbSettingsFailure.set(failure);
          this.bulbSettingsLoadState.set('error');
          this.snackbar.failure(failure, 'Could not load settings from this bulb.');
        },
      });
  }
  protected reloadBulbSettings() {
    const id = this.selectedBulbId();
    if (id) this.loadBulbSettings(id);
  }
  protected saveBulbSettings() {
    const id = this.selectedBulbId();
    if (!id || this.bulbSettingsSaving() || this.bulbSettingsLoadState() !== 'loaded') return;
    this.bulbSettingsForm().markAsTouched();
    if (this.bulbSettingsForm().invalid()) return;
    const settings: BulbSettings = {
      fade: this.bulbSettingsModel().fade,
      speed: this.bulbSettingsModel().speed,
    };
    const bulbName = this.bulbs().find((bulb) => bulb.id === id)?.name ?? 'Bulb';
    this.bulbSettingsSaving.set(true);
    this.bulbSettingsFailure.set(null);
    this.api
      .replaceBulbSettings(id, settings)
      .pipe(
        finalize(() => this.bulbSettingsSaving.set(false)),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe({
        next: (confirmed) => {
          if (this.selectedBulbId() === id) {
            this.setBulbSettings(confirmed);
            this.bulbSettingsLoadState.set('loaded');
          }
          this.snackbar.success(`Settings saved to ${bulbName}.`);
        },
        error: (failure: ApiFailure) => {
          if (this.selectedBulbId() === id) {
            this.bulbSettingsFailure.set(failure);
            this.bulbSettingsLoadState.set('error');
          }
          this.snackbar.failure(failure, `Could not save settings to ${bulbName}.`);
        },
      });
  }
  private setBulbSettings(settings: BulbSettings) {
    this.bulbSettingsForm().reset({ fade: settings.fade, speed: settings.speed });
  }
  private focusFirstInvalid() {
    const control = this.controls().find(
      (item) => item.nativeElement.getAttribute('aria-invalid') === 'true',
    );
    if (control) {
      control.nativeElement.focus();
      return;
    }
    if (this.model().mode === 'mixed' && this.settingsForm.mixedColor().invalid()) {
      this.lightConfiguration()?.focusMixedColor();
    } else if (this.model().mode === 'rgb' && this.settingsForm.rgbColor().invalid()) {
      this.lightConfiguration()?.focusColor();
    }
  }
}
