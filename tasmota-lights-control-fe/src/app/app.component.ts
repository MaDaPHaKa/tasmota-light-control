import {
  ChangeDetectionStrategy,
  Component,
  DestroyRef,
  computed,
  effect,
  inject,
  signal,
} from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { NavigationEnd, Router, RouterOutlet } from '@angular/router';
import { filter } from 'rxjs';
import { AppStoreService } from '@services/app-store.service';
import { SnackbarService } from '@services/snackbar.service';
import { SidebarComponent } from '@components/sidebar/sidebar.component';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet, SidebarComponent],
  templateUrl: './app.component.html',
  styleUrl: './app.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class AppComponent {
  private readonly router = inject(Router);
  private readonly store = inject(AppStoreService);
  private readonly snackbar = inject(SnackbarService);
  private readonly destroyRef = inject(DestroyRef);
  protected readonly bulbs = this.store.bulbs;
  protected readonly startupError = this.store.startupError;
  protected readonly routePath = signal(this.router.url);
  protected readonly routeTitle = computed(
    () =>
      ({ bulbs: 'Bulbs', profiles: 'Profiles', control: 'Control', 'direct-link': 'Direct link', settings: 'Settings' })[
        this.routePath().split('/')[1]
      ] ?? 'Not found',
  );

  constructor() {
    this.store.initialize();
    effect(() => {
      const startupError = this.startupError();
      if (startupError) this.snackbar.error(startupError);
    });
    this.router.events
      .pipe(
        filter((event) => event instanceof NavigationEnd),
        takeUntilDestroyed(this.destroyRef),
      )
      .subscribe((event) => this.routePath.set(event.urlAfterRedirects));
  }
}
