import { DestroyRef, Injectable, inject, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { catchError, forkJoin, of, tap } from 'rxjs';
import { Bulb, LiveBulbState, Uuid } from '@model/bulb.model';
import { LightProfile } from '@model/profile.model';
import { BulbStatusViewState, LoadState } from '@model/view-state.model';
import { BulbsApiService } from '@services/bulbs-api.service';
import { ProfilesApiService } from '@services/profiles-api.service';

@Injectable({ providedIn: 'root' })
export class AppStoreService {
  private readonly bulbsApi = inject(BulbsApiService);
  private readonly profilesApi = inject(ProfilesApiService);
  private readonly destroyRef = inject(DestroyRef);
  private initialized = false;
  readonly bulbs = signal<readonly Bulb[]>([]);
  readonly profiles = signal<readonly LightProfile[]>([]);
  readonly bulbsLoadState = signal<LoadState>('idle');
  readonly profilesLoadState = signal<LoadState>('idle');
  readonly statusByBulbId = signal<ReadonlyMap<Uuid, BulbStatusViewState>>(new Map());
  readonly startupError = signal<string | null>(null);
  initialize() {
    if (this.initialized) return;
    this.initialized = true;
    this.bulbsLoadState.set('loading');
    this.profilesLoadState.set('loading');
    forkJoin({
      bulbs: this.bulbsApi.list().pipe(
        tap((v) => this.bulbs.set(this.sorted(v))),
        tap(() => this.bulbsLoadState.set('loaded')),
        catchError(() => {
          this.bulbsLoadState.set('error');
          this.startupError.set('Could not load bulbs.');
          return of([] as Bulb[]);
        }),
      ),
      profiles: this.profilesApi.list().pipe(
        tap((v) => this.profiles.set(this.sorted(v))),
        tap(() => this.profilesLoadState.set('loaded')),
        catchError(() => {
          this.profilesLoadState.set('error');
          this.startupError.set('Could not load profiles.');
          return of([] as LightProfile[]);
        }),
      ),
    })
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(({ bulbs }) => {
        if (bulbs.length) this.refreshAllStatuses();
      });
  }
  reloadBulbs() {
    this.bulbsLoadState.set('loading');
    return this.bulbsApi.list().pipe(
      tap((v) => {
        this.bulbs.set(this.sorted(v));
        this.bulbsLoadState.set('loaded');
      }),
    );
  }
  reloadProfiles() {
    this.profilesLoadState.set('loading');
    return this.profilesApi.list().pipe(
      tap((v) => {
        this.profiles.set(this.sorted(v));
        this.profilesLoadState.set('loaded');
      }),
    );
  }
  refreshAllStatuses() {
    this.bulbs().forEach((b) =>
      this.statusByBulbId.update((m) => new Map(m).set(b.id, { kind: 'loading' })),
    );
    this.bulbsApi
      .getAllStatuses()
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: (states) => {
          const returned = new Set(states.map((s) => s.bulbId));
          states.forEach((s) => this.setStatus(s));
          this.bulbs()
            .filter((b) => !returned.has(b.id))
            .forEach((b) => this.setStatusError(b.id));
        },
        error: () => this.bulbs().forEach((b) => this.setStatusError(b.id)),
      });
  }
  refreshStatus(id: Uuid) {
    this.statusByBulbId.update((m) => new Map(m).set(id, { kind: 'loading' }));
    this.bulbsApi
      .getStatus(id)
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({ next: (s) => this.setStatus(s), error: () => this.setStatusError(id) });
  }
  private setStatus(s: LiveBulbState) {
    this.statusByBulbId.update((m) => new Map(m).set(s.bulbId, { kind: 'loaded', value: s }));
  }
  private setStatusError(id: Uuid) {
    this.statusByBulbId.update((m) =>
      new Map(m).set(id, { kind: 'error', message: 'Status unavailable.' }),
    );
  }
  upsertBulb(b: Bulb) {
    this.bulbs.update((v) =>
      this.sorted(v.some((x) => x.id === b.id) ? v.map((x) => (x.id === b.id ? b : x)) : [...v, b]),
    );
  }
  removeBulb(id: Uuid) {
    this.bulbs.update((v) => this.sorted(v.filter((x) => x.id !== id)));
    this.statusByBulbId.update((current) => {
      const next = new Map(current);
      next.delete(id);
      return next;
    });
  }
  upsertProfile(p: LightProfile) {
    this.profiles.update((v) =>
      this.sorted(v.some((x) => x.id === p.id) ? v.map((x) => (x.id === p.id ? p : x)) : [...v, p]),
    );
    this.refreshStatusesAfterProfileChange();
  }
  removeProfile(id: Uuid) {
    this.profiles.update((v) => this.sorted(v.filter((x) => x.id !== id)));
    this.refreshStatusesAfterProfileChange();
  }
  private refreshStatusesAfterProfileChange() {
    if (this.bulbs().length) this.refreshAllStatuses();
  }
  private sorted<T extends { name: string }>(values: readonly T[]): T[] {
    return [...values].sort((a, b) =>
      a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }),
    );
  }
}
