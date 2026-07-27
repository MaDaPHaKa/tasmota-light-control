import { ChangeDetectionStrategy, Component, input } from '@angular/core';
@Component({
  selector: 'app-mode-preview',
  standalone: true,
  templateUrl: './mode-preview.component.html',
  styleUrl: './mode-preview.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ModePreviewComponent {
  readonly color = input('#88C0D0');
  readonly text = input('#88C0D0');
  readonly dimmer = input(100);
}
