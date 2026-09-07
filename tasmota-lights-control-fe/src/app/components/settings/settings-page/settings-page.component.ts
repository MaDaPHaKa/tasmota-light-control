import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  DestroyRef,
  inject,
  signal,
  viewChildren,
} from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { finalize } from 'rxjs';
import { FormField, form, required, validate } from '@angular/forms/signals';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { ApiFailure } from '@model/api-error.model';
import { ResetSettings } from '@model/settings.model';
import { validateIntegerRange, validateUppercaseRgbHex } from '@functions/validators.function';
import { SettingsApiService } from '@services/settings-api.service';
import { EditableLightConfiguration, LightConfigurationComponent, LightConfigurationField } from '@components/shared/light-configuration/light-configuration.component';
import { SnackbarService } from '@services/snackbar.service';
import { PageHeader } from '@components/shared/page-header/page-header.component';

type SettingsField = 'dimmer' | 'mode' | 'rgbColor' | 'colorTemperatureKelvin' | 'fade' | 'speed';

@Component({
  selector: 'app-settings-page',
  standalone: true,
  imports: [
    FormField,
    MatButtonModule,
    MatFormFieldModule,
    MatInputModule,
    LightConfigurationComponent,
    PageHeader,
  ],
  templateUrl: './settings-page.component.html',
  styleUrl: './settings-page.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SettingsPageComponent {
  private readonly api = inject(SettingsApiService);
  private readonly snackbar = inject(SnackbarService);
  private readonly destroyRef = inject(DestroyRef);
  protected readonly model = signal({
    dimmer: 100,
    mode: 'color_temperature' as 'rgb' | 'color_temperature' | 'mixed',
    rgbColor: '#88C0D0',
    colorTemperatureKelvin: 3000,
    fade: 1 as number | null,
    speed: 4 as number | null,
  });
  protected readonly pending = signal(false);
  protected readonly failure = signal<ApiFailure | null>(null);
  protected readonly controls = viewChildren<ElementRef<HTMLElement>>('formControl');
  protected readonly settingsForm = form(this.model, (p) => {
    required(p.dimmer, { message: 'Dimmer is required' });
    validate(p.dimmer, ({ value }) => this.serverValidation('dimmer') ?? validateIntegerRange(value(), 1, 100));
    validate(p.mode, () => this.serverValidation('mode'));
    validate(p.rgbColor, ({ value }) =>
      this.serverValidation('rgbColor') ??
        (this.model().mode === 'rgb' ? validateUppercaseRgbHex(value()) : undefined),
    );
    validate(p.colorTemperatureKelvin, ({ value }) =>
      this.serverValidation('colorTemperatureKelvin') ??
        (this.model().mode !== 'rgb'
          ? validateIntegerRange(value(), 2000, 6000)
          : undefined),
    );
    validate(p.fade, ({ value }) => {
      const current = value();
      return current === null || Number.isNaN(current) ? undefined : validateIntegerRange(current, 0, 1);
    });
    validate(p.speed, ({ value }) => {
      const current = value();
      return current === null || Number.isNaN(current) ? undefined : validateIntegerRange(current, 1, 40);
    });
  });
  constructor() {
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
    this.failure.update((f) =>
      f
        ? {
            ...f,
            fields: Object.fromEntries(Object.entries(f.fields).filter(([key]) => key !== field)),
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
    if (this.pending()) return;
    this.settingsForm().markAsTouched();
    if (this.settingsForm().invalid()) {
      queueMicrotask(() => this.focusFirstInvalid());
      return;
    }
    const v = this.model();
    const fade = Number.isInteger(v.fade) ? v.fade : null;
    const speed = Number.isInteger(v.speed) ? v.speed : null;
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
              rgbColor: v.rgbColor.toUpperCase(),
              colorTemperatureKelvin: v.colorTemperatureKelvin,
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
    this.settingsForm().reset({
      dimmer: v.dimmer,
      mode: v.mode,
      rgbColor: v.rgbColor ?? '#88C0D0',
      colorTemperatureKelvin: v.colorTemperatureKelvin ?? 3000,
      fade: v.fade ?? null,
      speed: v.speed ?? null,
    });
  }
  private focusFirstInvalid() {
    const control = this.controls().find(
      (item) => item.nativeElement.getAttribute('aria-invalid') === 'true',
    );
    if (control) {
      control.nativeElement.focus();
      return;
    }
    if (this.model().mode !== 'color_temperature' && this.settingsForm.rgbColor().invalid()) return;
  }
}
