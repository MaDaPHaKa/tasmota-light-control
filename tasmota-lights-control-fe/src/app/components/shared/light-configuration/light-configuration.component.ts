import { ChangeDetectionStrategy, Component, ElementRef, input, output, viewChild } from '@angular/core';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatRadioModule } from '@angular/material/radio';
import { LightMode } from '@model/bulb.model';
import { normalizeMixedColorHex, validateMixedColorHex } from '@functions/validators.function';
import { ModePreviewComponent } from '@components/shared/mode-preview/mode-preview.component';
import { RgbColorPickerComponent } from '@components/shared/rgb-color-picker/rgb-color-picker.component';

export const DEFAULT_MIXED_COLOR = '#88C0D040BF';

export interface LightConfigurationValue {
  dimmer: number;
  mode: LightMode;
  rgbColor: string;
  colorTemperatureKelvin: number;
  mixedColor: string;
}

export type EditableLightConfiguration = LightConfigurationValue;

export type LightConfigurationField = keyof LightConfigurationValue;

export interface LightConfigurationErrors {
  dimmer?: string | null;
  mode?: string | null;
  rgbColor?: string | null;
  colorTemperatureKelvin?: string | null;
  mixedColor?: string | null;
}

@Component({
  selector: 'app-light-configuration',
  standalone: true,
  imports: [MatFormFieldModule, MatInputModule, MatRadioModule, ModePreviewComponent, RgbColorPickerComponent],
  templateUrl: './light-configuration.component.html',
  styleUrl: './light-configuration.component.scss',
  host: { '[class.compact]': 'compact()' },
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class LightConfigurationComponent {
  readonly value = input.required<LightConfigurationValue>();
  readonly disabled = input(false);
  readonly compact = input(false);
  readonly errors = input<LightConfigurationErrors>({});
  readonly valueChange = output<LightConfigurationValue>();
  readonly fieldChange = output<LightConfigurationField>();
  private readonly colorPicker = viewChild(RgbColorPickerComponent);
  private readonly mixedColorInput = viewChild<ElementRef<HTMLInputElement>>('mixedColorInput');

  focusColor(): void {
    this.colorPicker()?.focus();
  }

  focusMixedColor(): void {
    this.mixedColorInput()?.nativeElement.focus();
  }

  protected update(field: LightConfigurationField, fieldValue: number | string | LightMode): void {
    this.valueChange.emit({ ...this.value(), [field]: fieldValue } as LightConfigurationValue);
    this.fieldChange.emit(field);
  }

  protected colorPreview(): string {
    const value = this.value();
    if (value.mode !== 'mixed') return value.rgbColor;
    return validateMixedColorHex(value.mixedColor) ? '' : normalizeMixedColorHex(value.mixedColor).slice(0, 7);
  }

  protected previewText(): string {
    const value = this.value();
    return value.mode === 'mixed'
      ? validateMixedColorHex(value.mixedColor) === undefined
        ? normalizeMixedColorHex(value.mixedColor)
        : value.mixedColor
      : this.colorPreview();
  }
}
