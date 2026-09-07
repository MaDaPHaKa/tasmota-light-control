import { Routes } from '@angular/router';

export const routes: Routes = [
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
    path: 'direct-link',
    loadComponent: () =>
      import('./components/control/direct-link/direct-link.component').then(
        (m) => m.DirectLinkComponent,
      ),
  },
  {
    path: 'settings',
    loadComponent: () =>
      import('./components/settings/settings-page/settings-page.component').then(
        (m) => m.SettingsPageComponent,
      ),
  }, 
  { path: '', redirectTo: 'bulbs', pathMatch: 'full' },
];
