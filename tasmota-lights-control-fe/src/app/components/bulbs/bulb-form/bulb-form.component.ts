import { ChangeDetectionStrategy, Component, effect, input, output, signal } from '@angular/core';
import { FormField, form, required, validate } from '@angular/forms/signals';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { Bulb, BulbInput, BulbTestResult } from '@model/bulb.model';
import { validateCanonicalIpv4, validateIntegerRange, validateTrimmedRequired } from '@functions/validators.function';
@Component({
  selector: 'app-bulb-form',
  standalone: true,
  imports: [FormField, MatButtonModule, MatFormFieldModule, MatInputModule],
  templateUrl: './bulb-form.component.html',
  styleUrl: './bulb-form.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BulbFormComponent {
  readonly editing = input<Bulb | null>(null);
  readonly pending = input(false);
  readonly save = output<BulbInput>();
  readonly testEndpoint = output<{ ipAddress: string; port: number }>();
  readonly cancel = output<void>();
  readonly model = signal({ name: '', ipAddress: '', port: 80 });
  readonly bulbForm = form(this.model, (p) => {
    required(p.name, { message: 'Name is required' });
    required(p.ipAddress, { message: 'IP address is required' });
    validate(p.name, ({ value }) => validateTrimmedRequired(value(), 80));
    validate(p.ipAddress, ({ value }) => validateCanonicalIpv4(value()));
    validate(p.port, ({ value }) => validateIntegerRange(value(), 1, 65535));
  });
  constructor(){effect(()=>{const bulb=this.editing();if(bulb)this.model.set({name:bulb.name,ipAddress:bulb.ipAddress,port:bulb.port});});}
  protected submit() {
    if (this.bulbForm().valid())
      this.save.emit({ ...this.model(), name: this.model().name.trim() });
  }
  protected test() {
    this.testEndpoint.emit({ ipAddress: this.model().ipAddress, port: this.model().port });
  }
}
