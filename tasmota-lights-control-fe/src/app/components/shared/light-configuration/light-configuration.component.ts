import { ChangeDetectionStrategy, Component, input, output, viewChild } from '@angular/core';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatRadioModule } from '@angular/material/radio';
import { LightMode } from '@model/bulb.model';
import { rgbcctHex } from '@functions/light-color.function';
import { ModePreviewComponent } from '@components/shared/mode-preview/mode-preview.component';
import { RgbColorPickerComponent } from '@components/shared/rgb-color-picker/rgb-color-picker.component';

export interface LightConfigurationValue {
  dimmer: number;
  mode: LightMode;
  rgbColor: string;
  colorTemperatureKelvin: number;
}

export type EditableLightConfiguration = Pick<LightConfigurationValue, 'dimmer' | 'mode' | 'rgbColor' | 'colorTemperatureKelvin'>;

export type LightConfigurationField = keyof LightConfigurationValue;

export interface LightConfigurationErrors {
  dimmer?: string | null;
  mode?: string | null;
  rgbColor?: string | null;
  colorTemperatureKelvin?: string | null;
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

  focusColor(): void {
    this.colorPicker()?.focus();
  }

  protected update(field: LightConfigurationField, fieldValue: number | string | LightMode): void {
    this.valueChange.emit({ ...this.value(), [field]: fieldValue } as LightConfigurationValue);
    this.fieldChange.emit(field);
  }

  protected colorPreview(): string {
    const value = this.value();
    return value.mode === 'mixed' ? rgbcctHex(value.rgbColor, value.colorTemperatureKelvin) : value.rgbColor;
  }
}
