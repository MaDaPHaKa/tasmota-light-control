import { DatePipe } from '@angular/common';
import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  effect,
  input,
  output,
  signal,
  viewChildren,
} from '@angular/core';
import { FormField, form, required, validate } from '@angular/forms/signals';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { Bulb, BulbInput, BulbTestResult } from '@model/bulb.model';
import { ApiFailure } from '@model/api-error.model';
import {
  validateCanonicalIpv4,
  validateIntegerRange,
  validateTrimmedRequired,
} from '@functions/validators.function';

@Component({
  selector: 'app-bulb-form',
  standalone: true,
  imports: [DatePipe, FormField, MatButtonModule, MatFormFieldModule, MatInputModule],
  templateUrl: './bulb-form.component.html',
  styleUrl: './bulb-form.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BulbFormComponent {
  readonly editing = input<Bulb | null>(null);
  readonly pending = input(false);
  readonly testPending = input(false);
  readonly failure = input<ApiFailure | null>(null);
  readonly lastTestResult = input<BulbTestResult | null>(null);
  readonly save = output<BulbInput>();
  readonly testEndpoint = output<{ ipAddress: string; port: number }>();
  readonly cancel = output<void>();
  readonly fieldChanged = output<keyof BulbInput>();
  readonly endpointChanged = output<void>();
  readonly model = signal({ name: '', ipAddress: '', port: 80 });
  readonly controls = viewChildren<ElementRef<HTMLInputElement>>('formControl');
  readonly bulbForm = form(this.model, (p) => {
    required(p.name, { message: 'Name is required' });
    required(p.ipAddress, { message: 'IP address is required' });
    validate(p.name, ({ value }) => validateTrimmedRequired(value(), 80));
    validate(p.ipAddress, ({ value }) => validateCanonicalIpv4(value()));
    validate(p.port, ({ value }) => validateIntegerRange(value(), 1, 65535));
  });
  constructor() {
    effect(() => {
      const b = this.editing();
      this.model.set(
        b
          ? { name: b.name, ipAddress: b.ipAddress, port: b.port }
          : { name: '', ipAddress: '', port: 80 },
      );
    });
  }
  protected onSubmit() {
    this.bulbForm().markAsTouched();
    if (this.bulbForm().invalid()) {
      queueMicrotask(() => this.focusFirstInvalid());
      return;
    }
    const value = this.model();
    this.save.emit({ ...value, name: value.name.trim() });
  }
  protected test() {
    if (this.bulbForm.ipAddress().invalid() || this.bulbForm.port().invalid()) return;
    this.testEndpoint.emit({ ipAddress: this.model().ipAddress, port: this.model().port });
  }
  protected changed(field: keyof BulbInput) {
    this.fieldChanged.emit(field);
    if (field === 'ipAddress' || field === 'port') this.endpointChanged.emit();
  }
  protected showError(field: keyof BulbInput) {
    const control = this.bulbForm[field]();
    return control.touched() && control.invalid();
  }
  protected clientError(field: keyof BulbInput) {
    return this.bulbForm[field]().errors()[0]?.message;
  }
  protected serverError(field: keyof BulbInput) {
    return this.failure()?.fields[field] ?? null;
  }
  private focusFirstInvalid() {
    this.controls()
      .find((control) => control.nativeElement.getAttribute('aria-invalid') === 'true')
      ?.nativeElement.focus();
  }
}
