import { ChangeDetectionStrategy, Component, computed, input, output, signal } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { LightMode } from '@model/bulb.model';
import { SetPropertiesRequest } from '@model/control.model';
import { validateIntegerRange, validateUppercaseRgbHex } from '@functions/validators.function';
import { LightConfigurationComponent, EditableLightConfiguration } from '@components/shared/light-configuration/light-configuration.component';

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
  protected readonly valid = computed(() => {
    if (this.pending() || !this.selectedBulbCount()) return false;
    if (this.dimmer() < 1 || this.dimmer() > 100) return false;
    if (this.mode() !== 'color_temperature' && validateUppercaseRgbHex(this.rgbColor()) !== undefined) return false;
    return this.mode() === 'rgb' || validateIntegerRange(this.colorTemperatureKelvin(), 2000, 6000) === undefined;
  });
  protected readonly configuration = computed<EditableLightConfiguration>(() => ({
    dimmer: this.dimmer(), mode: this.mode(), rgbColor: this.rgbColor(), colorTemperatureKelvin: this.colorTemperatureKelvin(),
  }));
  protected updateConfiguration(value: EditableLightConfiguration) {
    this.dimmer.set(value.dimmer); this.mode.set(value.mode); this.rgbColor.set(value.rgbColor); this.colorTemperatureKelvin.set(value.colorTemperatureKelvin);
  }

  protected submit() {
    if (!this.valid()) return;
    const request: Omit<SetPropertiesRequest, 'bulbIds'> = { dimmer: this.dimmer() };
    if (this.mode() !== 'color_temperature') request.rgbColor = this.rgbColor().toUpperCase();
    if (this.mode() !== 'rgb') request.colorTemperatureKelvin = this.colorTemperatureKelvin();
    this.setProperties.emit(request);
  }
}
