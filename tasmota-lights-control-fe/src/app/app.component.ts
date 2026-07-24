import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { AppShellComponent } from './components/shell/app-shell/app-shell.component';
import { AppStoreService } from '@services/app-store.service';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [AppShellComponent],
  templateUrl: './app.component.html',
  styleUrl: './app.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class AppComponent {
  private readonly store = inject(AppStoreService);
  constructor() {
    this.store.initialize();
  }
}
