import { ChangeDetectionStrategy, Component, input, output } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatSelectModule } from '@angular/material/select';
import { LightProfile } from '@model/profile.model';
import { Uuid } from '@model/bulb.model';

@Component({
  selector: 'app-profile-control',
  standalone: true,
  imports: [MatButtonModule, MatFormFieldModule, MatIconModule, MatSelectModule],
  templateUrl: './profile-control.component.html',
  styleUrl: './profile-control.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ProfileControlComponent {
  readonly profiles = input<readonly LightProfile[]>([]);
  readonly selectedProfile = input<Uuid>('');
  readonly selectedBulbCount = input(0);
  readonly pending = input(false);
  readonly profileSelected = output<Uuid>();
  readonly applyProfile = output<void>();
}
