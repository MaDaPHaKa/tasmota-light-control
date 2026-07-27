import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  input,
  model,
  viewChild,
} from '@angular/core';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { ColorPicker } from '@acrodata/color-picker';

@Component({
  selector: 'app-rgb-color-picker',
  standalone: true,
  imports: [ColorPicker, MatFormFieldModule, MatInputModule],
  templateUrl: './rgb-color-picker.component.html',
  styleUrl: './rgb-color-picker.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class RgbColorPickerComponent {
  readonly value = model.required<string>();
  readonly disabled = input(false);
  readonly hideSwatch = input(false);
  readonly errorMessage = input<string | null>(null);
  private readonly hexInput = viewChild.required<ElementRef<HTMLInputElement>>('hexInput');

  focus(): void {
    this.hexInput().nativeElement.focus();
  }

  protected updateFromPicker(value: string): void {
    const normalized = this.normalizeOpaqueHex(value);
    if (normalized) this.value.set(normalized);
  }

  protected updateFromText(value: string): void {
    const normalized = this.normalizeOpaqueHex(value);
    this.value.set(normalized ?? value.toUpperCase());
  }

  private normalizeOpaqueHex(value: string): string | null {
    return /^#[0-9A-F]{6}$/i.test(value) ? value.toUpperCase() : null;
  }
}
