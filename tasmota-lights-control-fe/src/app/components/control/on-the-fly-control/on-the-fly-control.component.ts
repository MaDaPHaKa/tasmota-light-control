import { ChangeDetectionStrategy, Component, computed, input, output, signal } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { LightMode } from '@model/bulb.model';
import { SetPropertiesRequest } from '@model/control.model';
import {
  normalizeMixedColorHex,
  validateIntegerRange,
  validateMixedColorHex,
  validateUppercaseRgbHex,
} from '@functions/validators.function';
import { DEFAULT_MIXED_COLOR, LightConfigurationComponent, EditableLightConfiguration } from '@components/shared/light-configuration/light-configuration.component';

@Component({
  selector: 'app-on-the-fly-control',
  standalone: true,
  imports: [MatButtonModule, MatIconModule, LightConfigurationComponent],
  templateUrl: './on-the-fly-control.component.html',
  styleUrl: './on-the-fly-control.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class OnTheFlyControlComponent {
  readonly selectedBulbCount = input(0);
  readonly pending = input(false);
  readonly setProperties = output<Omit<SetPropertiesRequest, 'bulbIds'>>();

  protected readonly dimmer = signal(100);
  protected readonly mode = signal<LightMode>('rgb');
  protected readonly rgbColor = signal('#88C0D0');
  protected readonly colorTemperatureKelvin = signal(3000);
  protected readonly mixedColor = signal(DEFAULT_MIXED_COLOR);
  protected readonly valid = computed(() => {
    if (this.pending() || !this.selectedBulbCount()) return false;
    if (validateIntegerRange(this.dimmer(), 1, 100) !== undefined) return false;
    if (this.mode() === 'rgb') return validateUppercaseRgbHex(this.rgbColor()) === undefined;
    if (this.mode() === 'color_temperature')
      return validateIntegerRange(this.colorTemperatureKelvin(), 2000, 6000) === undefined;
    return validateMixedColorHex(this.mixedColor()) === undefined;
  });
  protected readonly configurationErrors = computed(() => ({
    mixedColor: validateMixedColorHex(this.mixedColor())?.message ?? null,
  }));
  protected readonly configuration = computed<EditableLightConfiguration>(() => ({
    dimmer: this.dimmer(),
    mode: this.mode(),
    rgbColor: this.rgbColor(),
    colorTemperatureKelvin: this.colorTemperatureKelvin(),
    mixedColor: this.mixedColor(),
  }));
  protected updateConfiguration(value: EditableLightConfiguration) {
    this.dimmer.set(value.dimmer);
    this.mode.set(value.mode);
    this.rgbColor.set(value.rgbColor);
    this.colorTemperatureKelvin.set(value.colorTemperatureKelvin);
    this.mixedColor.set(value.mixedColor);
  }

  protected submit() {
    if (!this.valid()) return;
    const request: Omit<SetPropertiesRequest, 'bulbIds'> = { dimmer: this.dimmer() };
    if (this.mode() === 'rgb') request.rgbColor = this.rgbColor().toUpperCase();
    if (this.mode() === 'color_temperature') request.colorTemperatureKelvin = this.colorTemperatureKelvin();
    if (this.mode() === 'mixed') request.mixedColor = normalizeMixedColorHex(this.mixedColor());
    this.setProperties.emit(request);
  }

  protected applyDimmer() {
    if (!this.valid()) return;
    const request: Omit<SetPropertiesRequest, 'bulbIds'> = { dimmer: this.dimmer() };
    this.setProperties.emit(request);
  }
}
