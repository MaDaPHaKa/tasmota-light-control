import { Routes } from '@angular/router';

export const routes: Routes = [
  { path: '', redirectTo: 'bulbs', pathMatch: 'full' },
  {
    path: 'bulbs',
    loadComponent: () =>
      import('./components/bulbs/bulbs-page/bulbs-page.component').then(
        (m) => m.BulbsPageComponent,
      ),
  },
  {
    path: 'profiles',
    loadComponent: () =>
      import('./components/profiles/profiles-page/profiles-page.component').then(
        (m) => m.ProfilesPageComponent,
      ),
  },
  {
    path: 'control',
    loadComponent: () =>
      import('./components/control/control-page/control-page.component').then(
        (m) => m.ControlPageComponent,
      ),
  },
  {
    path: 'settings',
    loadComponent: () =>
      import('./components/settings/settings-page/settings-page.component').then(
        (m) => m.SettingsPageComponent,
      ),
  },
  {
    path: '**',
    loadComponent: () =>
      import('./components/not-found/not-found-page/not-found-page.component').then(
        (m) => m.NotFoundPageComponent,
      ),
  },
];
