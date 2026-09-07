import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';

@Component({
  selector: 'app-reset-bulbs',
  standalone: true,
  imports: [MatButtonModule],
  templateUrl: './reset-bulbs.component.html',
  styleUrl: './reset-bulbs.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ResetBulbsComponent {
  readonly bulbCount = input(0);
  readonly selectedBulbCount = input(0);
  readonly pending = input(false);
  readonly resetSelected = output<void>();
  readonly resetAll = output<void>();
}
