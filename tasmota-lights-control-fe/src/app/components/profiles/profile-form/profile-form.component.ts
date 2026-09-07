import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  effect,
  input,
  output,
  signal,
  viewChild,
  viewChildren,
} from '@angular/core';
import { FormField, form, required, validate } from '@angular/forms/signals';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatRadioModule } from '@angular/material/radio';
import { ApiFailure } from '@model/api-error.model';
import { LightProfile, LightProfileInput } from '@model/profile.model';
import {
  validateIntegerRange,
  validateTrimmedRequired,
  validateUppercaseRgbHex,
} from '@functions/validators.function';
import { ModePreviewComponent } from '@components/shared/mode-preview/mode-preview.component';
import { RgbColorPickerComponent } from '@components/shared/rgb-color-picker/rgb-color-picker.component';
import { rgbcctHex } from '@functions/light-color.function';

type ProfileField = 'name' | 'dimmer' | 'mode' | 'rgbColor' | 'colorTemperatureKelvin';

@Component({
  selector: 'app-profile-form',
  standalone: true,
  imports: [
    FormField,
    MatButtonModule,
    MatFormFieldModule,
    MatInputModule,
    MatRadioModule,
    RgbColorPickerComponent,
    ModePreviewComponent,
  ],
  templateUrl: './profile-form.component.html',
  styleUrl: './profile-form.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ProfileFormComponent {
  readonly editing = input<LightProfile | null>(null);
  readonly pending = input(false);
  readonly failure = input<ApiFailure | null>(null);
  readonly save = output<LightProfileInput>();
  readonly cancel = output<void>();
  readonly fieldChanged = output<ProfileField>();
  readonly model = signal({
    name: '',
    dimmer: 100,
    mode: 'rgb' as 'rgb' | 'color_temperature' | 'mixed',
    rgbColor: '#88C0D0',
    colorTemperatureKelvin: 3000,
  });
  readonly controls = viewChildren<ElementRef<HTMLElement>>('formControl');
  private readonly colorPicker = viewChild(RgbColorPickerComponent);
  readonly profileForm = form(this.model, (p) => {
    required(p.name, { message: 'Name is required' });
    required(p.dimmer, { message: 'Dimmer is required' });
    validate(p.name, ({ value }) => this.serverValidation('name') ?? validateTrimmedRequired(value(), 80));
    validate(p.dimmer, ({ value }) => this.serverValidation('dimmer') ?? validateIntegerRange(value(), 1, 100));
    validate(p.mode, () => this.serverValidation('mode'));
    validate(p.rgbColor, ({ value }) =>
      this.serverValidation('rgbColor') ??
        (this.model().mode !== 'color_temperature' ? validateUppercaseRgbHex(value()) : undefined),
    );
    validate(p.colorTemperatureKelvin, ({ value }) =>
      this.serverValidation('colorTemperatureKelvin') ??
          (this.model().mode !== 'rgb'
          ? validateIntegerRange(value(), 2000, 6000)
          : undefined),
    );
  });
  constructor() {
    effect(() => {
      const profile = this.editing();
      this.profileForm().reset(
        profile
          ? {
              name: profile.name,
              dimmer: profile.dimmer,
              mode: profile.mode,
              rgbColor: profile.rgbColor ?? '#88C0D0',
              colorTemperatureKelvin: profile.colorTemperatureKelvin ?? 3000,
            }
          : {
              name: '',
              dimmer: 100,
              mode: 'rgb',
              rgbColor: '#88C0D0',
              colorTemperatureKelvin: 3000,
            },
      );
    });
  }
  protected onSubmit() {
    if (this.pending()) return;
    this.profileForm().markAsTouched();
    if (this.profileForm().invalid()) {
      queueMicrotask(() => this.focusFirstInvalid());
      return;
    }
    const v = this.model();
    this.save.emit(
      v.mode === 'rgb'
        ? {
            name: v.name.trim(),
            dimmer: v.dimmer,
            mode: 'rgb',
            rgbColor: v.rgbColor.toUpperCase(),
            colorTemperatureKelvin: null,
          }
        : v.mode === 'color_temperature'
          ? {
            name: v.name.trim(),
            dimmer: v.dimmer,
            mode: 'color_temperature',
            rgbColor: null,
            colorTemperatureKelvin: v.colorTemperatureKelvin,
            }
          : {
              name: v.name.trim(),
              dimmer: v.dimmer,
              mode: 'mixed',
              rgbColor: v.rgbColor.toUpperCase(),
              colorTemperatureKelvin: v.colorTemperatureKelvin,
            },
    );
  }
  protected updateRgbColor(value: string) {
    this.model.update((current) => ({ ...current, rgbColor: value }));
    this.profileForm.rgbColor().markAsTouched();
    this.changed('rgbColor');
  }
  protected colorPreview() {
    return this.model().mode === 'mixed'
      ? rgbcctHex(this.model().rgbColor, this.model().colorTemperatureKelvin)
      : this.model().rgbColor;
  }
  protected changed(field: ProfileField) {
    this.fieldChanged.emit(field);
  }
  protected clientError(field: ProfileField) {
    return this.profileForm[field]().errors()[0]?.message ?? null;
  }
  protected showClientError(field: ProfileField) {
    const state = this.profileForm[field]();
    return state.invalid() && (state.touched() || !!this.serverValidation(field));
  }
  private serverValidation(field: ProfileField) {
    const fields = this.failure()?.fields ?? {};
    const message = fields[field] ?? fields[field === 'rgbColor' ? 'rgb_color' : field];
    return message ? { kind: 'server', message } : undefined;
  }
  private focusFirstInvalid() {
    const control = this.controls().find(
      (item) => item.nativeElement.getAttribute('aria-invalid') === 'true',
    );
    if (control) {
      control.nativeElement.focus();
      return;
    }
    if (this.model().mode !== 'color_temperature' && this.profileForm.rgbColor().invalid())
      this.colorPicker()?.focus();
  }
}
